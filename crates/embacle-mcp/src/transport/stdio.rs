// ABOUTME: Stdio transport reading newline-delimited JSON-RPC from stdin and writing to stdout
// ABOUTME: Standard MCP transport for integration with editors and CLI tool wrappers
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::sync::Arc;

use async_trait::async_trait;
use embacle::types::RunnerError;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error};

use crate::protocol::{JsonRpcRequest, JsonRpcResponse, PARSE_ERROR};
use crate::server::McpServer;
use crate::transport::McpTransport;

/// MCP transport over stdin/stdout using newline-delimited JSON-RPC
///
/// Each line on stdin is expected to be a complete JSON-RPC message.
/// Responses are written as single lines to stdout. Logs go to stderr
/// to avoid polluting the protocol channel.
pub struct StdioTransport;

#[async_trait]
impl McpTransport for StdioTransport {
    async fn serve(self, server: Arc<McpServer>) -> Result<(), RunnerError> {
        let stdin = BufReader::new(tokio::io::stdin());
        let mut stdout = tokio::io::stdout();
        let mut lines = stdin.lines();

        debug!("Stdio transport ready, waiting for JSON-RPC messages on stdin");

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    error!(error = %e, "Failed to parse JSON-RPC request");
                    let resp =
                        JsonRpcResponse::error(None, PARSE_ERROR, format!("Parse error: {e}"));
                    write_response(&mut stdout, &resp).await?;
                    continue;
                }
            };

            debug!(method = %request.method, "Handling MCP request");

            if let Some(response) = server.handle_request(request).await {
                write_response(&mut stdout, &response).await?;
            }
        }

        debug!("Stdin closed, shutting down stdio transport");
        Ok(())
    }
}

/// Serialize and write a JSON-RPC response as a single line to stdout
async fn write_response(
    stdout: &mut tokio::io::Stdout,
    response: &JsonRpcResponse,
) -> Result<(), RunnerError> {
    let json = serde_json::to_string(response)
        .map_err(|e| RunnerError::internal(format!("JSON serialization failed: {e}")))?;

    stdout
        .write_all(json.as_bytes())
        .await
        .map_err(|e| RunnerError::internal(format!("stdout write failed: {e}")))?;

    stdout
        .write_all(b"\n")
        .await
        .map_err(|e| RunnerError::internal(format!("stdout newline write failed: {e}")))?;

    stdout
        .flush()
        .await
        .map_err(|e| RunnerError::internal(format!("stdout flush failed: {e}")))?;

    Ok(())
}
