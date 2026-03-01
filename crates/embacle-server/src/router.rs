// ABOUTME: Axum router wiring all REST endpoints for the OpenAI-compatible API
// ABOUTME: Mounts completions, models, and health routes with optional auth middleware
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;

use crate::auth;
use crate::completions;
use crate::health;
use crate::models;
use crate::state::SharedState;

/// Build the application router with all endpoints
///
/// Routes:
/// - `POST /v1/chat/completions` — Chat completion (streaming and non-streaming)
/// - `GET /v1/models` — List available models
/// - `GET /health` — Provider health check
///
/// The auth middleware is applied to all routes. It only enforces
/// authentication when `EMBACLE_API_KEY` is set.
pub fn build(state: SharedState) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(completions::handle))
        .route("/v1/models", get(models::handle))
        .route("/health", get(health::handle))
        .layer(middleware::from_fn(auth::require_auth))
        .with_state(state)
}
