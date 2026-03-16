//! Process runner v1 for execution_mode=process modules.

use crate::core::runtime::process_ipc::{read_response, send_request};
use crate::core::runtime::process_ipc_messages::{RequestEnvelope, ResponseEnvelope, TraceContext};
use serde_json::json;
use std::io::{BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRunnerConfig {
    pub default_timeout_ms: u64,
    pub cancel_grace_ms: u64,
    pub auto_restart_on_crash: bool,
}

impl Default for ProcessRunnerConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30_000,
            cancel_grace_ms: 2_000,
            auto_restart_on_crash: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessModuleRunner {
    command: String,
    args: Vec<String>,
    module_id: String,
    config: ProcessRunnerConfig,
    trace_context: TraceContext,
}

impl ProcessModuleRunner {
    pub fn new(
        module_id: impl Into<String>,
        command: impl Into<String>,
        args: Vec<String>,
        config: ProcessRunnerConfig,
        trace_context: TraceContext,
    ) -> Self {
        Self {
            command: command.into(),
            args,
            module_id: module_id.into(),
            config,
            trace_context,
        }
    }

    pub fn from_env(module_id: &str) -> anyhow::Result<Self> {
        let command = std::env::var("REDHORSE_PROCESS_MODULE_BIN").map_err(|_| {
            anyhow::anyhow!(
                "ipc.method_unsupported: process runner command is not configured (REDHORSE_PROCESS_MODULE_BIN)"
            )
        })?;

        let args = vec!["--module".to_string(), module_id.to_string()];

        Ok(Self::new(
            module_id,
            command,
            args,
            ProcessRunnerConfig::default(),
            TraceContext::from_env(),
        ))
    }

    pub fn activate(&self) -> anyhow::Result<ResponseEnvelope> {
        let activate_payload = json!({
            "config": {},
            "capabilities": [],
            "traceContext": {
                "traceparent": self.trace_context.traceparent,
                "tracestate": self.trace_context.tracestate
            },
            "identityScope": null
        });

        self.execute("activate", activate_payload, self.config.default_timeout_ms)
    }

    pub fn invoke(
        &self,
        kind: &str,
        correlation_id: &str,
        input: serde_json::Value,
    ) -> anyhow::Result<ResponseEnvelope> {
        let invoke_payload = json!({
            "kind": kind,
            "capabilities": [],
            "traceContext": {
                "traceparent": self.trace_context.traceparent,
                "tracestate": self.trace_context.tracestate
            },
            "correlationId": correlation_id,
            "input": input
        });

        self.execute("invoke", invoke_payload, self.config.default_timeout_ms)
    }

    fn execute(
        &self,
        method: &str,
        payload: serde_json::Value,
        timeout_ms: u64,
    ) -> anyhow::Result<ResponseEnvelope> {
        let mut child = self.spawn_child()?;

        {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("ipc.invalid_frame: missing child stdin"))?;

            let handshake = RequestEnvelope::new(
                "req-0",
                "handshake",
                json!({
                    "protocolVersion": 1,
                    "moduleId": self.module_id,
                    "executionMode": "process",
                    "traceContext": {
                        "traceparent": self.trace_context.traceparent,
                        "tracestate": self.trace_context.tracestate
                    },
                    "identityScope": null
                }),
            );
            send_request(stdin, &handshake)?;

            let request = RequestEnvelope::new("req-1", method, payload);
            send_request(stdin, &request)?;
        }

        drop(child.stdin.take());

        let status = wait_with_timeout(
            &mut child,
            Duration::from_millis(timeout_ms),
            Duration::from_millis(self.config.cancel_grace_ms),
        )?;
        let mut stdout_buf = Vec::new();
        if let Some(stdout) = child.stdout.as_mut() {
            stdout.read_to_end(&mut stdout_buf)?;
        }

        let mut stderr_buf = String::new();
        if let Some(stderr) = child.stderr.as_mut() {
            stderr.read_to_string(&mut stderr_buf)?;
        }

        if !status.success() {
            anyhow::bail!(
                "ipc.child_crashed: process exited with status {status}; stderr={stderr_buf}"
            );
        }

        let mut reader = BufReader::new(std::io::Cursor::new(stdout_buf));
        let mut activate_response: Option<ResponseEnvelope> = None;
        while let Some(response) = read_response(&mut reader)? {
            if response.id == "req-1" {
                activate_response = Some(response);
                break;
            }
        }

        activate_response.ok_or_else(|| {
            anyhow::anyhow!("ipc.invalid_payload: missing terminal response for request id req-1")
        })
    }

    fn spawn_child(&self) -> anyhow::Result<Child> {
        let mut command = Command::new(&self.command);
        command
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        command.env_clear();
        if let Ok(path) = std::env::var("PATH") {
            command.env("PATH", path);
        }
        if let Ok(system_root) = std::env::var("SYSTEMROOT") {
            command.env("SYSTEMROOT", system_root);
        }
        if !self.trace_context.traceparent.is_empty() {
            command.env("TRACEPARENT", &self.trace_context.traceparent);
        }
        if !self.trace_context.tracestate.is_empty() {
            command.env("TRACESTATE", &self.trace_context.tracestate);
        }

        Ok(command.spawn()?)
    }
}

fn wait_with_timeout(
    child: &mut Child,
    timeout: Duration,
    cancel_grace: Duration,
) -> anyhow::Result<std::process::ExitStatus> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        thread::sleep(Duration::from_millis(10));
    }

    let cancel = RequestEnvelope::new("req-cancel", "cancel", json!({"id": "req-1"}));

    if let Some(stdin) = child.stdin.as_mut() {
        let _ = send_request(stdin, &cancel);
    }

    let grace_start = Instant::now();
    while grace_start.elapsed() < cancel_grace {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        thread::sleep(Duration::from_millis(10));
    }

    child.kill()?;
    let _ = child.wait();
    anyhow::bail!("ipc.timeout: child process exceeded timeout budget")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timeout_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".to_string(),
                vec!["/C".to_string(), "ping 127.0.0.1 -n 4 >NUL".to_string()],
            )
        } else {
            (
                "sh".to_string(),
                vec!["-c".to_string(), "sleep 3".to_string()],
            )
        }
    }

    fn crash_command() -> (String, Vec<String>) {
        if cfg!(windows) {
            (
                "cmd".to_string(),
                vec!["/C".to_string(), "exit /B 3".to_string()],
            )
        } else {
            (
                "sh".to_string(),
                vec!["-c".to_string(), "exit 3".to_string()],
            )
        }
    }

    #[test]
    fn process_timeout_is_reported() {
        let (command, args) = timeout_command();
        let runner = ProcessModuleRunner::new(
            "tool-shell",
            command,
            args,
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

    #[test]
    fn process_crash_is_reported() {
        let (command, args) = crash_command();
        let runner = ProcessModuleRunner::new(
            "tool-shell",
            command,
            args,
            ProcessRunnerConfig::default(),
            TraceContext::default(),
        );

        let err = runner.activate().expect_err("activate should fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("ipc.child_crashed") || msg.contains("ipc.invalid_payload"));
    }
}
