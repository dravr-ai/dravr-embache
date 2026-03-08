// ABOUTME: Re-exports the runner factory from the core embacle crate
// ABOUTME: Delegates to embacle::factory::create_runner for all provider instantiation
//
// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 dravr.ai

pub use embacle::factory::create_runner;
