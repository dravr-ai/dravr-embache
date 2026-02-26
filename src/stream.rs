// ABOUTME: Stream wrapper that owns a child process for proper lifecycle management
// ABOUTME: Prevents zombie processes, drains stderr, and kills child on drop
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::process::Child;
use tokio::task::JoinHandle;
use tokio_stream::Stream;

use crate::types::{RunnerError, StreamChunk};

/// Maximum stderr to buffer during streaming (1 MiB)
pub(crate) const MAX_STREAMING_STDERR_BYTES: usize = 1024 * 1024;

/// Guards a child process for the lifetime of a streaming response.
///
/// When the stream is dropped (after natural completion or early
/// cancellation), the child process is killed and the stderr drain
/// task is aborted. This prevents zombie processes and resource leaks.
///
/// All fields are `Unpin`, so `GuardedStream` is `Unpin` and the
/// `Stream` impl can safely access inner fields through `Pin<&mut Self>`.
pub struct GuardedStream {
    inner: Pin<Box<dyn Stream<Item = Result<StreamChunk, RunnerError>> + Send>>,
    child: Option<Child>,
    stderr_task: Option<JoinHandle<Vec<u8>>>,
}

impl GuardedStream {
    /// Create a guarded stream wrapping a child process.
    ///
    /// The `stderr_task` drains the child's stderr in the background
    /// to prevent buffer-full deadlocks where the child blocks on
    /// write to a full stderr pipe.
    pub fn new(
        inner: impl Stream<Item = Result<StreamChunk, RunnerError>> + Send + 'static,
        child: Child,
        stderr_task: JoinHandle<Vec<u8>>,
    ) -> Self {
        Self {
            inner: Box::pin(inner),
            child: Some(child),
            stderr_task: Some(stderr_task),
        }
    }
}

impl Stream for GuardedStream {
    type Item = Result<StreamChunk, RunnerError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

impl Drop for GuardedStream {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
        }
        if let Some(task) = self.stderr_task.take() {
            task.abort();
        }
    }
}
