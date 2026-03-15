//! Process module timeout tests.

use redclaw::core::runtime::{ProcessModuleRunner, ProcessRunnerConfig, TraceContext};

#[test]
fn process_module_timeout_returns_ipc_timeout() {
    let runner = ProcessModuleRunner::new(
        "tool-shell",
        "python",
        vec!["-c".to_string(), "import time; time.sleep(3)".to_string()],
        ProcessRunnerConfig {
            default_timeout_ms: 100,
            cancel_grace_ms: 50,
            auto_restart_on_crash: false,
        },
        TraceContext::default(),
    );

    let err = runner.activate().expect_err("activate should timeout");
    let msg = format!("{err:#}");
    assert!(msg.contains("ipc.timeout") || msg.contains("ipc.invalid_payload"));
}
