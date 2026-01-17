//! Workflow execution engine

use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;

use crate::workflow::{
    Workflow, WorkflowExecution, NodeExecution, ExecutionStatus, NodeType,
    DAGValidator, ToolRegistry, StateStore,
};
use crate::error::{Result, SynapseError};
use crate::router::SmartRouter;
use crate::{ChatRequest, Message};

pub struct ExecutionEngine {
    state_store: Arc<StateStore>,
    tool_registry: Arc<ToolRegistry>,
    router: Option<Arc<SmartRouter>>,
}

impl ExecutionEngine {
    pub fn new(state_store: Arc<StateStore>, tool_registry: Arc<ToolRegistry>) -> Self {
        Self { state_store, tool_registry, router: None }
    }

    pub fn with_router(mut self, router: Arc<SmartRouter>) -> Self {
        self.router = Some(router);
        self
    }

    pub async fn execute(&self, workflow: &Workflow, variables: HashMap<String, serde_json::Value>, user_id: &str) -> Result<WorkflowExecution> {
        // Validate
        let validation = DAGValidator::validate(workflow);
        if !validation.valid {
            return Err(SynapseError::Validation(validation.errors.join(", ")));
        }

        // Create execution
        let mut execution = WorkflowExecution::new(&workflow.id, user_id).with_variables(variables);
        execution.status = ExecutionStatus::Running;
        self.state_store.save_execution(execution.clone()).await?;

        // Get topological order
        let order = DAGValidator::topological_sort(workflow)
            .map_err(|e| SynapseError::Validation(e))?;

        // Execute nodes
        for node_id in order {
            let node = workflow.nodes.iter().find(|n| n.id == node_id)
                .ok_or_else(|| SynapseError::Internal("Node not found".into()))?;

            let mut node_exec = NodeExecution::new(&node_id);
            node_exec.status = ExecutionStatus::Running;
            self.state_store.save_node_execution(&execution.id, node_exec.clone()).await?;

            match self.execute_node(node, &execution).await {
                Ok(output) => {
                    execution.variables.insert(node_id.clone(), output.clone());
                    node_exec = node_exec.complete(output);
                }
                Err(e) => {
                    node_exec = node_exec.fail(e.to_string());
                    execution.status = ExecutionStatus::Failed;
                    execution.error = Some(e.to_string());
                    self.state_store.save_node_execution(&execution.id, node_exec).await?;
                    break;
                }
            }
            
            execution.nodes_execution.insert(node_id, node_exec.clone());
            self.state_store.save_node_execution(&execution.id, node_exec).await?;
        }

        if execution.status == ExecutionStatus::Running {
            execution.status = ExecutionStatus::Completed;
        }
        execution.end_time = Some(Utc::now());
        self.state_store.save_execution(execution.clone()).await?;

        Ok(execution)
    }

    async fn execute_node(&self, node: &crate::workflow::WorkflowNode, exec: &WorkflowExecution) -> Result<serde_json::Value> {
        match &node.node_type {
            NodeType::LLM { model, temperature, max_tokens, system_prompt } => {
                self.execute_llm(model, *temperature, *max_tokens, system_prompt.as_deref(), exec).await
            }
            NodeType::HTTP { method, url, headers, body_template } => {
                self.execute_http(method, url, headers, body_template.as_deref(), exec).await
            }
            NodeType::Condition { expression } => {
                self.evaluate_condition(expression, exec).await
            }
            NodeType::Transform { expression } => {
                self.transform_data(expression, exec).await
            }
            NodeType::Delay { seconds } => {
                tokio::time::sleep(std::time::Duration::from_secs(*seconds as u64)).await;
                Ok(serde_json::json!({"delayed": seconds}))
            }
            NodeType::Tool { tool_name } => {
                let input = exec.variables.get("input").cloned().unwrap_or(serde_json::Value::Null);
                self.tool_registry.execute(tool_name, input).await
            }
            _ => Ok(serde_json::json!({"status": "executed"})),
        }
    }

    async fn execute_llm(&self, model: &str, temperature: f32, max_tokens: u32, system_prompt: Option<&str>, exec: &WorkflowExecution) -> Result<serde_json::Value> {
        let router = self.router.as_ref().ok_or_else(|| SynapseError::Internal("No LLM router".into()))?;
        
        let prompt = exec.variables.get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello");

        let mut messages = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(Message::system(sys));
        }
        messages.push(Message::user(prompt));

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            temperature,
            max_tokens,
            ..Default::default()
        };

        let response = router.route(&request).await?;
        let content = response.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(serde_json::json!({
            "response": content,
            "model": response.model,
            "tokens": response.usage.total_tokens,
        }))
    }

    async fn execute_http(&self, method: &str, url: &str, _headers: &HashMap<String, String>, _body: Option<&str>, _exec: &WorkflowExecution) -> Result<serde_json::Value> {
        self.tool_registry.execute("http", serde_json::json!({
            "method": method,
            "url": url,
        })).await
    }

    async fn evaluate_condition(&self, expression: &str, exec: &WorkflowExecution) -> Result<serde_json::Value> {
        // Simple expression evaluation
        let result = if expression.contains("==") {
            let parts: Vec<&str> = expression.split("==").collect();
            if parts.len() == 2 {
                let left = parts[0].trim();
                let right = parts[1].trim().trim_matches('"');
                let left_val = exec.variables.get(left).and_then(|v| v.as_str()).unwrap_or("");
                left_val == right
            } else { false }
        } else if expression == "true" { true }
        else { false };

        Ok(serde_json::json!({"result": result}))
    }

    async fn transform_data(&self, _expression: &str, exec: &WorkflowExecution) -> Result<serde_json::Value> {
        // Return variables as-is for now
        Ok(serde_json::json!(exec.variables))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::WorkflowNode;

    #[tokio::test]
    async fn test_execute_simple_workflow() {
        let store = Arc::new(StateStore::new());
        let tools = Arc::new(ToolRegistry::new());
        let engine = ExecutionEngine::new(store, tools);

        let mut workflow = Workflow::new("Test", "Test", "user1");
        workflow.add_node(WorkflowNode {
            id: "delay1".into(),
            name: "Wait".into(),
            node_type: NodeType::Delay { seconds: 0 },
            retry_count: 0,
            timeout_seconds: 10,
            config: serde_json::Value::Null,
        });

        let result = engine.execute(&workflow, HashMap::new(), "user1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, ExecutionStatus::Completed);
    }
}
