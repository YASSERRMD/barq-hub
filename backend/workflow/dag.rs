//! DAG validation for workflows

use std::collections::{HashMap, HashSet, VecDeque};
use crate::workflow::{Workflow, WorkflowEdge};

/// Validates workflow DAGs for cycles, orphans, and other issues
pub struct DAGValidator;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![msg.into()],
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
        self.valid = false;
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

impl DAGValidator {
    /// Validate a workflow DAG
    pub fn validate(workflow: &Workflow) -> ValidationResult {
        let mut result = ValidationResult::ok();

        // Check for empty workflow
        if workflow.nodes.is_empty() {
            result.add_warning("Workflow has no nodes");
            return result;
        }

        // Check for cycles
        if let Some(cycle) = Self::find_cycle(workflow) {
            result.add_error(format!("Workflow contains a cycle involving node: {}", cycle));
        }

        // Check for orphaned nodes
        if let Some(orphans) = Self::find_orphaned_nodes(workflow) {
            for orphan in orphans {
                result.add_warning(format!("Node '{}' is not connected to any edges", orphan));
            }
        }

        // Validate edge references
        if let Some(invalid_edges) = Self::validate_edges(workflow) {
            for (from, to) in invalid_edges {
                result.add_error(format!("Invalid edge: {} -> {} (node not found)", from, to));
            }
        }

        // Check for multiple entry points
        let entry_points = Self::find_entry_points(workflow);
        if entry_points.len() > 1 {
            result.add_warning(format!("Workflow has {} entry points: {:?}", entry_points.len(), entry_points));
        }

        // Check for no exit points
        let exit_points = Self::find_exit_points(workflow);
        if exit_points.is_empty() && !workflow.nodes.is_empty() {
            result.add_warning("Workflow has no exit points (may contain cycles)");
        }

        result
    }

    /// Check if the workflow has a cycle using DFS
    fn find_cycle(workflow: &Workflow) -> Option<String> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in &workflow.nodes {
            if !visited.contains(&node.id) {
                if let Some(cycle_node) = Self::dfs_cycle(&node.id, workflow, &mut visited, &mut rec_stack) {
                    return Some(cycle_node);
                }
            }
        }
        None
    }

    fn dfs_cycle(
        node_id: &str,
        workflow: &Workflow,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Option<String> {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());

        for edge in &workflow.edges {
            if edge.from == node_id {
                if !visited.contains(&edge.to) {
                    if let Some(cycle) = Self::dfs_cycle(&edge.to, workflow, visited, rec_stack) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&edge.to) {
                    return Some(edge.to.clone());
                }
            }
        }

        rec_stack.remove(node_id);
        None
    }

    /// Find nodes not connected to any edges
    fn find_orphaned_nodes(workflow: &Workflow) -> Option<Vec<String>> {
        if workflow.nodes.len() == 1 {
            return None; // Single node is valid
        }

        let node_ids: HashSet<_> = workflow.nodes.iter().map(|n| n.id.clone()).collect();
        let referenced_ids: HashSet<_> = workflow
            .edges
            .iter()
            .flat_map(|e| vec![e.from.clone(), e.to.clone()])
            .collect();

        let orphans: Vec<_> = node_ids.difference(&referenced_ids).cloned().collect();
        
        if orphans.is_empty() {
            None
        } else {
            Some(orphans)
        }
    }

    /// Validate that all edge references exist
    fn validate_edges(workflow: &Workflow) -> Option<Vec<(String, String)>> {
        let node_ids: HashSet<_> = workflow.nodes.iter().map(|n| n.id.clone()).collect();
        let mut invalid = Vec::new();

        for edge in &workflow.edges {
            if !node_ids.contains(&edge.from) || !node_ids.contains(&edge.to) {
                invalid.push((edge.from.clone(), edge.to.clone()));
            }
        }

        if invalid.is_empty() {
            None
        } else {
            Some(invalid)
        }
    }

    /// Find entry points (nodes with no incoming edges)
    fn find_entry_points(workflow: &Workflow) -> Vec<String> {
        let has_incoming: HashSet<_> = workflow.edges.iter().map(|e| e.to.clone()).collect();
        
        workflow.nodes
            .iter()
            .filter(|n| !has_incoming.contains(&n.id))
            .map(|n| n.id.clone())
            .collect()
    }

    /// Find exit points (nodes with no outgoing edges)
    fn find_exit_points(workflow: &Workflow) -> Vec<String> {
        let has_outgoing: HashSet<_> = workflow.edges.iter().map(|e| e.from.clone()).collect();
        
        workflow.nodes
            .iter()
            .filter(|n| !has_outgoing.contains(&n.id))
            .map(|n| n.id.clone())
            .collect()
    }

    /// Perform topological sort using Kahn's algorithm
    pub fn topological_sort(workflow: &Workflow) -> Result<Vec<String>, String> {
        let mut in_degree: HashMap<String, u32> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for node in &workflow.nodes {
            in_degree.insert(node.id.clone(), 0);
            adjacency.insert(node.id.clone(), Vec::new());
        }

        // Build graph
        for edge in &workflow.edges {
            if let Some(count) = in_degree.get_mut(&edge.to) {
                *count += 1;
            }
            if let Some(adj) = adjacency.get_mut(&edge.from) {
                adj.push(edge.to.clone());
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<_> = in_degree
            .iter()
            .filter(|(_, &count)| count == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            result.push(node_id.clone());

            if let Some(neighbors) = adjacency.get(&node_id) {
                for neighbor in neighbors {
                    if let Some(count) = in_degree.get_mut(neighbor) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        if result.len() == workflow.nodes.len() {
            Ok(result)
        } else {
            Err("Cycle detected in workflow".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::WorkflowNode;

    fn create_test_workflow() -> Workflow {
        let mut workflow = Workflow::new("Test", "Test workflow", "user1");
        workflow.add_node(WorkflowNode::llm("node1", "Start", "gpt-4"));
        workflow.add_node(WorkflowNode::llm("node2", "Process", "gpt-4"));
        workflow.add_node(WorkflowNode::llm("node3", "End", "gpt-4"));
        workflow.add_edge("node1", "node2");
        workflow.add_edge("node2", "node3");
        workflow
    }

    #[test]
    fn test_valid_workflow() {
        let workflow = create_test_workflow();
        let result = DAGValidator::validate(&workflow);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_cycle_detection() {
        let mut workflow = create_test_workflow();
        workflow.add_edge("node3", "node1"); // Create cycle
        
        let result = DAGValidator::validate(&workflow);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("cycle")));
    }

    #[test]
    fn test_topological_sort() {
        let workflow = create_test_workflow();
        let order = DAGValidator::topological_sort(&workflow).unwrap();
        
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], "node1");
        assert_eq!(order[1], "node2");
        assert_eq!(order[2], "node3");
    }

    #[test]
    fn test_invalid_edge() {
        let mut workflow = create_test_workflow();
        workflow.edges.push(WorkflowEdge {
            from: "node1".to_string(),
            to: "nonexistent".to_string(),
            condition: None,
        });
        
        let result = DAGValidator::validate(&workflow);
        assert!(!result.valid);
    }
}
