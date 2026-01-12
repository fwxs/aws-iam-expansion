//! AWS IAM Expansion Toolkit
//!
//! This library provides utilities for expanding AWS IAM policy permissions,
//! specifically for understanding which specific actions are represented by
//! wildcard services in IAM policies.
//!
//! # Overview
//!
//! The toolkit fetches the complete list of AWS IAM actions and services from
//! the AWS IAM Actions API (awsiamactions.io) and provides functionality to:
//!
//! - List all available AWS services and their service prefixes
//! - Expand IAM actions for a specific service, optionally filtered by prefix
//! - Efficiently search for actions using trie-based prefix matching
//!
//! # Modules
//!
//! - [`command`]: Command-line interface and argument parsing
//! - [`types`]: Core data structures for AWS services and actions
//! - [`utils`]: Utility functions for fetching and caching IAM actions data
//!
//! # Examples
//!
//! ```no_run
//! use std::collections::HashMap;
//! use aws_iam_expansion::{command::Args, types::AwsService, utils::retrieve_iam_actions_json};
//! use clap::Parser;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Parse command-line arguments
//! let cli_args = Args::parse();
//!
//! // Retrieve AWS IAM actions (cached if available)
//! let iam_actions_json = retrieve_iam_actions_json()?;
//!
//! // Deserialize and organize by service prefix
//! let mut services: HashMap<String, Vec<AwsService>> = HashMap::new();
//! for service in serde_json::from_str::<Vec<AwsService>>(&iam_actions_json)? {
//!     services.entry(service.prefix.clone())
//!         .or_insert_with(Vec::new)
//!         .push(service);
//! }
//!
//! // Handle the command
//! cli_args.handle(services)?;
//! # Ok(())
//! # }
//! ```

pub mod command;
pub mod types;
pub mod utils;
