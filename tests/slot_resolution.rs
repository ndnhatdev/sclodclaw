//! Slot/dependency resolution tests.

use redclaw::core::lifecycle::DependencyGraph;

#[test]
fn activation_order_is_dependency_first() {
    let mut graph = DependencyGraph::new();
    graph.add_module("runtime-native", vec![]);
    graph.add_module("channel-cli", vec!["runtime-native".to_string()]);
    graph.add_module(
        "provider-openai-compatible",
        vec!["runtime-native".to_string()],
    );

    let order = graph.activation_order().expect("activation order");
    let runtime_index = order.iter().position(|m| m == "runtime-native").unwrap();
    let channel_index = order.iter().position(|m| m == "channel-cli").unwrap();
    let provider_index = order
        .iter()
        .position(|m| m == "provider-openai-compatible")
        .unwrap();

    assert!(runtime_index < channel_index);
    assert!(runtime_index < provider_index);
}

#[test]
fn dependency_cycle_blocks_activation() {
    let mut graph = DependencyGraph::new();
    graph.add_module("a", vec!["b".to_string()]);
    graph.add_module("b", vec!["a".to_string()]);

    let err = graph.validate_dag().expect_err("cycle should fail");
    assert!(err.contains("Cycle detected"));
}
