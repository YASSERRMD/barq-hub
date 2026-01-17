//! Workflow types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique workflow ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Workflow nodes
    pub nodes: Vec<WorkflowNode>,
    /// Edges connecting nodes
    pub edges: Vec<WorkflowEdge>,
    /// Version number
    pub version: u32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Owner user ID
    pub owner_id: String,
    /// Whether workflow is enabled
    pub enabled: bool,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Workflow {
    pub fn new(name: impl Into<String>, description: impl Into<String>, owner_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
            version: 1,
            created_at: now,
            updated_at: now,
            owner_id: owner_id.into(),
            enabled: true,
            tags: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: WorkflowNode) {
        self.nodes.push(node);
        self.updated_at = Utc::now();
    }

    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.edges.push(WorkflowEdge {
            from: from.into(),
            to: to.into(),
            condition: None,
        });
        self.updated_at = Utc::now();
    }
}

/// A single node in the workflow DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// Unique node ID within workflow
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Node type and configuration
    pub node_type: NodeType,
    /// Number of retries on failure
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
    /// Node-specific configuration
    #[serde(default)]
    pub config: serde_json::Value,
}

fn default_retry_count() -> u32 { 3 }
fn default_timeout() -> u32 { 300 }

impl WorkflowNode {
    pub fn llm(id: impl Into<String>, name: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: NodeType::LLM {
                model: model.into(),
                temperature: 0.7,
                max_tokens: 2048,
                system_prompt: None,
            },
            retry_count: 3,
            timeout_seconds: 120,
            config: serde_json::Value::Null,
        }
    }

    pub fn http(id: impl Into<String>, name: impl Into<String>, method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: NodeType::HTTP {
                method: method.into(),
                url: url.into(),
                headers: HashMap::new(),
                body_template: None,
            },
            retry_count: 3,
            timeout_seconds: 60,
            config: serde_json::Value::Null,
        }
    }

    pub fn condition(id: impl Into<String>, name: impl Into<String>, expression: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type: NodeType::Condition {
                expression: expression.into(),
            },
            retry_count: 0,
            timeout_seconds: 10,
            config: serde_json::Value::Null,
        }
    }
}

/// Types of nodes supported in workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeType {
    /// LLM inference node
    LLM {
        model: String,
        temperature: f32,
        max_tokens: u32,
        system_prompt: Option<String>,
    },
    /// HTTP request node
    HTTP {
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body_template: Option<String>,
    },
    /// SQL query node
    SQL {
        query: String,
        database: String,
    },
    /// Kafka message node
    Kafka {
        topic: String,
        action: String, // "publish" or "consume"
    },
    /// Conditional branching
    Condition {
        expression: String,
    },
    /// Loop over items
    Loop {
        items_var: String,
        max_iterations: u32,
    },
    /// Merge multiple branches
    Merge {
        merge_type: MergeType,
    },
    /// Fork into parallel branches
    Fork {
        branches: u32,
    },
    /// Join parallel branches
    Join {
        join_type: JoinType,
    },
    /// Transform data
    Transform {
        expression: String,
    },
    /// Delay execution
    Delay {
        seconds: u32,
    },
    /// Custom tool execution
    Tool {
        tool_name: String,
    },
    /// Sub-workflow
    SubWorkflow {
        workflow_id: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeType {
    Concat,
    Object,
    First,
    Last,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinType {
    All,
    Any,
    Race,
}

/// Edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Optional condition for edge traversal
    pub condition: Option<String>,
}

/// Status of a workflow execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
    Cancelled,
    TimedOut,
}

impl Default for ExecutionStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// A workflow execution instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// Unique execution ID
    pub id: String,
    /// Workflow ID being executed
    pub workflow_id: String,
    /// Current status
    pub status: ExecutionStatus,
    /// Node execution states
    pub nodes_execution: HashMap<String, NodeExecution>,
    /// Execution variables/context
    pub variables: HashMap<String, serde_json::Value>,
    /// Start timestamp
    pub start_time: DateTime<Utc>,
    /// End timestamp (if completed)
    pub end_time: Option<DateTime<Utc>>,
    /// User who triggered execution
    pub triggered_by: String,
    /// Error message if failed
    pub error: Option<String>,
}

impl WorkflowExecution {
    pub fn new(workflow_id: impl Into<String>, triggered_by: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workflow_id: workflow_id.into(),
            status: ExecutionStatus::Pending,
            nodes_execution: HashMap::new(),
            variables: HashMap::new(),
            start_time: Utc::now(),
            end_time: None,
            triggered_by: triggered_by.into(),
            error: None,
        }
    }

    pub fn with_variables(mut self, variables: HashMap<String, serde_json::Value>) -> Self {
        self.variables = variables;
        self
    }

    pub fn duration_ms(&self) -> Option<i64> {
        self.end_time.map(|end| {
            (end - self.start_time).num_milliseconds()
        })
    }
}

/// Execution state of a single node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecution {
    /// Node ID
    pub node_id: String,
    /// Current status
    pub status: ExecutionStatus,
    /// Input data
    pub input: serde_json::Value,
    /// Output data
    pub output: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Start timestamp
    pub start_time: DateTime<Utc>,
    /// End timestamp
    pub end_time: Option<DateTime<Utc>>,
    /// Current retry count
    pub retry_count: u32,
}

impl NodeExecution {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            status: ExecutionStatus::Pending,
            input: serde_json::Value::Null,
            output: None,
            error: None,
            start_time: Utc::now(),
            end_time: None,
            retry_count: 0,
        }
    }

    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = input;
        self
    }

    pub fn complete(mut self, output: serde_json::Value) -> Self {
        self.status = ExecutionStatus::Completed;
        self.output = Some(output);
        self.end_time = Some(Utc::now());
        self
    }

    pub fn fail(mut self, error: impl Into<String>) -> Self {
        self.status = ExecutionStatus::Failed;
        self.error = Some(error.into());
        self.end_time = Some(Utc::now());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let workflow = Workflow::new("Test Workflow", "A test", "user1");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.version, 1);
        assert!(workflow.enabled);
    }

    #[test]
    fn test_add_nodes() {
        let mut workflow = Workflow::new("Test", "Test", "user1");
        workflow.add_node(WorkflowNode::llm("llm1", "GPT-4 Call", "gpt-4"));
        workflow.add_node(WorkflowNode::http("http1", "API Call", "POST", "https://api.example.com"));
        workflow.add_edge("llm1", "http1");
        
        assert_eq!(workflow.nodes.len(), 2);
        assert_eq!(workflow.edges.len(), 1);
    }

    #[test]
    fn test_execution_creation() {
        let exec = WorkflowExecution::new("workflow1", "user1");
        assert_eq!(exec.status, ExecutionStatus::Pending);
        assert!(exec.variables.is_empty());
    }

    #[test]
    fn test_node_execution_lifecycle() {
        let node_exec = NodeExecution::new("node1")
            .with_input(serde_json::json!({"prompt": "Hello"}));
        
        assert_eq!(node_exec.status, ExecutionStatus::Pending);
        
        let completed = node_exec.complete(serde_json::json!({"response": "Hi!"}));
        assert_eq!(completed.status, ExecutionStatus::Completed);
        assert!(completed.output.is_some());
    }
}
