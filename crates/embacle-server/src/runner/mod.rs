// ABOUTME: Runner management layer bridging REST handlers to embacle LlmProvider instances
// ABOUTME: Re-exports factory, provider parsing, and provider enumeration from core
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

pub mod factory;
pub mod multiplex;

pub use embacle::factory::{parse_runner_type, valid_provider_names, ALL_PROVIDERS};
