//! State persistence for workflow executions

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::workflow::{Workflow, WorkflowExecution, NodeExecution, ExecutionStatus};
use crate::error::{Result, SynapseError};

/// In-memory state store (for development; use PostgreSQL in production)
pub struct StateStore {
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
    executions: Arc<RwLock<HashMap<String, WorkflowExecution>>>,
    checkpoints: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Workflow CRUD
    pub async fn save_workflow(&self, workflow: Workflow) -> Result<()> {
        self.workflows.write().await.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    pub async fn get_workflow(&self, id: &str) -> Result<Option<Workflow>> {
        Ok(self.workflows.read().await.get(id).cloned())
    }

    pub async fn list_workflows(&self) -> Vec<Workflow> {
        self.workflows.read().await.values().cloned().collect()
    }

    pub async fn delete_workflow(&self, id: &str) -> Result<bool> {
        Ok(self.workflows.write().await.remove(id).is_some())
    }

    // Execution CRUD
    pub async fn save_execution(&self, execution: WorkflowExecution) -> Result<()> {
        self.executions.write().await.insert(execution.id.clone(), execution);
        Ok(())
    }

    pub async fn get_execution(&self, id: &str) -> Result<Option<WorkflowExecution>> {
        Ok(self.executions.read().await.get(id).cloned())
    }

    pub async fn list_executions(&self, workflow_id: Option<&str>) -> Vec<WorkflowExecution> {
        let execs = self.executions.read().await;
        match workflow_id {
            Some(wid) => execs.values().filter(|e| e.workflow_id == wid).cloned().collect(),
            None => execs.values().cloned().collect(),
        }
    }

    pub async fn update_execution_status(&self, id: &str, status: ExecutionStatus) -> Result<()> {
        if let Some(exec) = self.executions.write().await.get_mut(id) {
            exec.status = status;
            if matches!(status, ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled) {
                exec.end_time = Some(chrono::Utc::now());
            }
        }
        Ok(())
    }

    pub async fn save_node_execution(&self, execution_id: &str, node_exec: NodeExecution) -> Result<()> {
        if let Some(exec) = self.executions.write().await.get_mut(execution_id) {
            exec.nodes_execution.insert(node_exec.node_id.clone(), node_exec);
        }
        Ok(())
    }

    // Checkpoints
    pub async fn save_checkpoint(&self, execution_id: &str, node_id: &str, data: serde_json::Value) -> Result<()> {
        let key = format!("{}:{}", execution_id, node_id);
        self.checkpoints.write().await.insert(key, data);
        Ok(())
    }

    pub async fn get_checkpoint(&self, execution_id: &str, node_id: &str) -> Option<serde_json::Value> {
        let key = format!("{}:{}", execution_id, node_id);
        self.checkpoints.read().await.get(&key).cloned()
    }

    pub async fn clear_checkpoints(&self, execution_id: &str) -> Result<()> {
        let prefix = format!("{}:", execution_id);
        self.checkpoints.write().await.retain(|k, _| !k.starts_with(&prefix));
        Ok(())
    }
}

impl Default for StateStore {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_crud() {
        let store = StateStore::new();
        let workflow = Workflow::new("Test", "Test", "user1");
        let id = workflow.id.clone();
        
        store.save_workflow(workflow).await.unwrap();
        let loaded = store.get_workflow(&id).await.unwrap();
        assert!(loaded.is_some());
        
        store.delete_workflow(&id).await.unwrap();
        let deleted = store.get_workflow(&id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_execution_crud() {
        let store = StateStore::new();
        let exec = WorkflowExecution::new("workflow1", "user1");
        let id = exec.id.clone();
        
        store.save_execution(exec).await.unwrap();
        store.update_execution_status(&id, ExecutionStatus::Completed).await.unwrap();
        
        let loaded = store.get_execution(&id).await.unwrap().unwrap();
        assert_eq!(loaded.status, ExecutionStatus::Completed);
    }
}
