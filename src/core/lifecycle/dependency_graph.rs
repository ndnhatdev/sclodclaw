//! Dependency graph and DAG validation.

use std::collections::{HashMap, HashSet};

pub struct DependencyGraph {
    dependencies: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, module_id: &str, dependencies: Vec<String>) {
        self.dependencies
            .insert(module_id.to_string(), dependencies);
    }

    pub fn validate_dag(&self) -> Result<(), String> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        for module_id in self.dependencies.keys() {
            if !visited.contains(module_id)
                && self.has_cycle(module_id, &mut visited, &mut rec_stack)?
            {
                return Err(format!("Cycle detected: {}", module_id));
            }
        }
        Ok(())
    }

    pub fn activation_order(&self) -> Result<Vec<String>, String> {
        self.validate_dag()?;

        let mut ordered = Vec::new();
        let mut permanent = HashSet::new();
        let mut temporary = HashSet::new();

        let mut module_ids = self.dependencies.keys().cloned().collect::<Vec<_>>();
        module_ids.sort();

        for module_id in module_ids {
            self.visit_for_order(&module_id, &mut temporary, &mut permanent, &mut ordered)?;
        }

        Ok(ordered)
    }

    fn has_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Result<bool, String> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        if let Some(deps) = self.dependencies.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    if self.has_cycle(dep, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(dep) {
                    return Ok(true);
                }
            }
        }
        rec_stack.remove(node);
        Ok(false)
    }

    fn visit_for_order(
        &self,
        node: &str,
        temporary: &mut HashSet<String>,
        permanent: &mut HashSet<String>,
        ordered: &mut Vec<String>,
    ) -> Result<(), String> {
        if permanent.contains(node) {
            return Ok(());
        }
        if temporary.contains(node) {
            return Err(format!("Cycle detected while ordering: {node}"));
        }

        temporary.insert(node.to_string());

        if let Some(deps) = self.dependencies.get(node) {
            for dep in deps {
                if self.dependencies.contains_key(dep) {
                    self.visit_for_order(dep, temporary, permanent, ordered)?;
                }
            }
        }

        temporary.remove(node);
        permanent.insert(node.to_string());
        ordered.push(node.to_string());
        Ok(())
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::DependencyGraph;

    #[test]
    fn activation_order_places_dependencies_first() {
        let mut graph = DependencyGraph::new();
        graph.add_module("runtime-native", vec![]);
        graph.add_module("channel-cli", vec!["runtime-native".to_string()]);

        let order = graph.activation_order().expect("activation order");
        let runtime_index = order
            .iter()
            .position(|m| m == "runtime-native")
            .expect("runtime index");
        let channel_index = order
            .iter()
            .position(|m| m == "channel-cli")
            .expect("channel index");

        assert!(
            runtime_index < channel_index,
            "runtime-native must activate before channel-cli"
        );
    }
}
