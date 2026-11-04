// PTY Multiplexer OpenRPC Unix Domain Socket Adapter.
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use trustkit_core::entities::{
    AgentSemanticState, BlockReasonKind, MultiplexedTerminalHandle, MultiplexedTerminalRequest,
};
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::TerminalMultiplexerPort;

static REQ_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_req_id() -> u64 {
    REQ_COUNTER.fetch_add(1, Ordering::SeqCst)
}

fn current_timestamp_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

fn fault(msg: impl Into<String>) -> DomainViolation {
    DomainViolation::InfrastructureBoundaryFault {
        subsystem: "pty_socket_adapter".to_string(),
        details: msg.into(),
    }
}

#[derive(Debug, Clone)]
pub struct PtySocketAdapter {
    pub socket_path: PathBuf,
}

impl PtySocketAdapter {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    pub fn discover_default() -> Option<Self> {
        if let Ok(env_path) =
            std::env::var("PTY_SOCKET").or_else(|_| std::env::var("PTY_MULTIPLEXER_SOCKET"))
        {
            let p = PathBuf::from(env_path);
            if p.exists() {
                return Some(Self::new(p));
            }
        }
        for c in &[
            "/tmp/pty-multiplexer.sock",
            "/tmp/pty.sock",
            "/var/run/pty.sock",
        ] {
            let p = Path::new(c);
            if p.exists() {
                return Some(Self::new(p.to_path_buf()));
            }
        }
        None
    }

    async fn rpc_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, DomainViolation> {
        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            fault(format!(
                "Failed connecting to socket {}: {e}",
                self.socket_path.display()
            ))
        })?;

        let payload = serde_json::json!({
            "jsonrpc": "2.0", "id": next_req_id(), "method": method, "params": params
        });
        let mut msg = serde_json::to_string(&payload)
            .map_err(|e| fault(format!("Serialization error: {e}")))?;
        msg.push('\n');

        stream
            .write_all(msg.as_bytes())
            .await
            .map_err(|e| fault(format!("Socket write error: {e}")))?;

        let (reader, _) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();
        let line = lines
            .next_line()
            .await
            .map_err(|e| fault(format!("Socket read error: {e}")))?
            .ok_or_else(|| fault("PTY socket closed connection prematurely"))?;

        let resp: serde_json::Value =
            serde_json::from_str(&line).map_err(|e| fault(format!("JSON-RPC parse error: {e}")))?;
        if let Some(err) = resp.get("error") {
            return Err(fault(format!("PTY RPC error: {err}")));
        }
        Ok(resp
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    }

    fn run_async<F, T>(f: F) -> Result<T, DomainViolation>
    where
        F: std::future::Future<Output = Result<T, DomainViolation>> + Send + 'static,
        T: Send + 'static,
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
                tokio::task::block_in_place(|| handle.block_on(f))
            } else {
                std::thread::scope(|s| {
                    s.spawn(|| {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .map_err(|e| fault(format!("Tokio init error: {e}")))?;
                        rt.block_on(f)
                    })
                    .join()
                    .map_err(|_| fault("Thread panicked during socket RPC"))?
                })
            }
        } else {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| fault(format!("Tokio init error: {e}")))?;
            rt.block_on(f)
        }
    }
}

impl TerminalMultiplexerPort for PtySocketAdapter {
    fn spawn_terminal(
        &self,
        req: &MultiplexedTerminalRequest,
    ) -> Result<MultiplexedTerminalHandle, DomainViolation> {
        let env_map: serde_json::Map<String, serde_json::Value> = req
            .env_vars
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();
        let params = serde_json::json!({
            "argv": req.command_argv, "cwd": req.working_dir, "env": env_map, "timeout": req.timeout_ms / 1000
        });
        let this = self.clone();
        let res = Self::run_async(async move { this.rpc_call("terminal.run", params).await })?;
        let term_id = res["id"].as_str().unwrap_or("term-unknown").to_string();
        let pid = res["pid"].as_u64().unwrap_or(0) as u32;
        Ok(MultiplexedTerminalHandle::new(
            term_id,
            req.task_id.clone(),
            pid,
            current_timestamp_iso(),
        ))
    }

    fn query_terminal_state(
        &self,
        handle: &MultiplexedTerminalHandle,
    ) -> Result<AgentSemanticState, DomainViolation> {
        let params = serde_json::json!({ "id": handle.terminal_id });
        let this = self.clone();
        let res = Self::run_async(async move { this.rpc_call("terminal.get", params).await })?;
        let state = res["state"]
            .as_str()
            .unwrap_or("working")
            .to_ascii_lowercase();
        let kind = res["kind"].as_str().unwrap_or("");
        let detail = res["detail"].as_str().unwrap_or("").to_string();
        let exit_code = res["exit_code"].as_i64().unwrap_or(0) as i32;

        match state.as_str() {
            "working" => Ok(AgentSemanticState::Working),
            "blocked" => Ok(AgentSemanticState::Blocked {
                kind: BlockReasonKind::from_str_name(kind),
                detail,
            }),
            "done" | "idle" => Ok(AgentSemanticState::Done { exit_code }),
            "error" | "failed" => Ok(AgentSemanticState::Failed {
                reason: if detail.is_empty() {
                    "Terminal execution failed".to_string()
                } else {
                    detail
                },
            }),
            _ => Ok(AgentSemanticState::Working),
        }
    }

    fn send_terminal_input(
        &self,
        handle: &MultiplexedTerminalHandle,
        input: &str,
    ) -> Result<(), DomainViolation> {
        let params = serde_json::json!({ "id": handle.terminal_id, "text": input });
        let this = self.clone();
        let _ = Self::run_async(async move { this.rpc_call("terminal.send", params).await })?;
        Ok(())
    }

    fn read_terminal_buffer(
        &self,
        handle: &MultiplexedTerminalHandle,
        max_lines: usize,
    ) -> Result<String, DomainViolation> {
        let params = serde_json::json!({ "id": handle.terminal_id, "lines": max_lines });
        let this = self.clone();
        let res = Self::run_async(async move { this.rpc_call("terminal.read", params).await })?;
        Ok(res["buffer"].as_str().unwrap_or("").to_string())
    }

    fn terminate_terminal(
        &self,
        handle: &MultiplexedTerminalHandle,
    ) -> Result<(), DomainViolation> {
        let params = serde_json::json!({ "id": handle.terminal_id });
        let this = self.clone();
        let _ = Self::run_async(async move { this.rpc_call("terminal.kill", params).await })?;
        Ok(())
    }
}
