//! Entry point for the AWS IAM Expansion toolkit.
//!
//! This binary application provides a command-line interface to expand AWS IAM policy
//! wildcards and discover specific permissions behind wildcard service specifications.
//!
//! # Overview
//!
//! The application fetches AWS IAM actions data from the AWS IAM Actions API, caches it locally,
//! and provides commands to:
//! - List all available AWS services
//! - Expand IAM actions for a specific service with optional prefix filtering
//!
//! # Usage
//!
//! ```sh
//! # List all available AWS services
//! cargo run -- list-services
//!
//! # Expand all actions for a service
//! cargo run -- expand --service-name iam
//!
//! # Expand actions matching a prefix
//! cargo run -- expand --service-name iam --prefix Create
//! ```
//!
//! # Todo
//!
//! - Add option to read AWS IAM policy or Role from a local file for direct policy expansion.

use clap::Parser;

/// Main entry point for the AWS IAM Expansion toolkit.
///
/// This function orchestrates the entire application flow:
/// 1. Parses command-line arguments using the `clap` crate
/// 2. Retrieves AWS IAM actions data (from cache or API)
/// 3. Deserializes the JSON data into `AwsService` structures
/// 4. Organizes services by their service prefix in a HashMap
/// 5. Delegates command handling to the parsed arguments
///
/// # Returns
///
/// - `Ok(())` if the command executes successfully
/// - `Err(Box<dyn std::error::Error>)` if any error occurs during:
///   - Argument parsing
///   - Data retrieval (network or file I/O)
///   - JSON deserialization
///   - Command execution
///
/// # Errors
///
/// The function can return errors from:
/// - Network requests (if cache doesn't exist)
/// - File system operations (cache read/write)
/// - JSON parsing of the IAM actions data
/// - Service lookup or action expansion
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = aws_iam_expansion::command::Args::parse();
    let iam_actions_json = aws_iam_expansion::utils::retrieve_iam_actions_json()?;
    let mut available_services_permissions: std::collections::HashMap<
        String,
        Vec<aws_iam_expansion::types::AwsService>,
    > = std::collections::HashMap::new();

    for service in
        serde_json::from_str::<Vec<aws_iam_expansion::types::AwsService>>(&iam_actions_json)?
    {
        available_services_permissions
            .entry(service.prefix.clone())
            .or_insert_with(Vec::new)
            .push(service);
    }

    cli_args.handle(available_services_permissions)
}
