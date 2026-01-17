//! Workflow API handlers

use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use std::collections::HashMap;
use crate::api::state::AppState;
use crate::workflow::{Workflow, WorkflowNode, WorkflowExecution, ExecutionStatus, ExecutionEngine, StateStore, ToolRegistry};
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<crate::workflow::WorkflowEdge>,
}

#[derive(Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub variables: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize)]
pub struct WorkflowListItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_count: usize,
    pub enabled: bool,
}

pub async fn list_workflows(State(state): State<Arc<AppState>>) -> Json<Vec<WorkflowListItem>> {
    // Try to get from database first
    if let Some(ref pool) = state.db_pool {
        if let Ok(rows) = sqlx::query_as::<_, (String, String, Option<String>, String)>(
            "SELECT id, name, description, status FROM workflows ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await {
            let items: Vec<WorkflowListItem> = rows.into_iter().map(|(id, name, desc, status)| {
                WorkflowListItem {
                    id,
                    name,
                    description: desc.unwrap_or_default(),
                    node_count: 0,
                    enabled: status == "active",
                }
            }).collect();
            return Json(items);
        }
    }
    
    // Fallback to in-memory store
    let workflows = state.workflow_store.list_workflows().await;
    Json(workflows.into_iter().map(|w| WorkflowListItem {
        id: w.id, name: w.name, description: w.description, node_count: w.nodes.len(), enabled: w.enabled
    }).collect())
}

pub async fn get_workflow(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Result<Json<Workflow>> {
    match state.workflow_store.get_workflow(&id).await? {
        Some(w) => Ok(Json(w)),
        None => Err(crate::error::SynapseError::Validation("Workflow not found".into())),
    }
}

pub async fn create_workflow(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<(StatusCode, Json<Workflow>)> {
    let mut workflow = Workflow::new(&req.name, &req.description, "api");
    workflow.nodes = req.nodes;
    workflow.edges = req.edges;
    state.workflow_store.save_workflow(workflow.clone()).await?;
    Ok((StatusCode::CREATED, Json(workflow)))
}

pub async fn delete_workflow(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> StatusCode {
    match state.workflow_store.delete_workflow(&id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        _ => StatusCode::NOT_FOUND,
    }
}

pub async fn execute_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteWorkflowRequest>,
) -> Result<Json<WorkflowExecution>> {
    let workflow = state.workflow_store.get_workflow(&id).await?
        .ok_or_else(|| crate::error::SynapseError::Validation("Workflow not found".into()))?;
    
    let engine = ExecutionEngine::new(state.workflow_store.clone(), state.tool_registry.clone())
        .with_router(state.router.clone());
    
    let variables = req.variables.unwrap_or_default();
    let execution = engine.execute(&workflow, variables, "api").await?;
    Ok(Json(execution))
}

pub async fn get_execution(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Result<Json<WorkflowExecution>> {
    match state.workflow_store.get_execution(&id).await? {
        Some(e) => Ok(Json(e)),
        None => Err(crate::error::SynapseError::Validation("Execution not found".into())),
    }
}

pub async fn list_executions(State(state): State<Arc<AppState>>) -> Json<Vec<WorkflowExecution>> {
    Json(state.workflow_store.list_executions(None).await)
}
