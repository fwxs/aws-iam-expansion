//! Command-line interface and argument parsing for the AWS IAM expansion toolkit.
//!
//! This module defines the CLI structure and handles subcommand execution for expanding
//! AWS IAM permissions. It provides functionality to list available AWS services and
//! expand actions for a specific service using prefix matching.

use clap::Subcommand;
use std::collections::HashMap;

/// Represents the "expand" subcommand for expanding AWS IAM actions.
///
/// This command allows users to expand wildcard service actions to see all specific
/// permissions available for a given AWS service. It supports optional prefix matching
/// to filter actions by a partial action name.
#[derive(Debug, clap::Args)]
pub struct ExpandSubCommand {
    /// The action name prefix to filter actions (e.g., "Create" to find "CreateUser", "CreateRole").
    ///
    /// Wildcards (*) in the prefix are automatically removed during matching.
    /// If not provided, all actions for the service are returned.
    #[arg(long, required = false, requires = "service_name")]
    prefix: Option<String>,

    /// The AWS service name (prefix) to expand actions for (e.g., "iam", "s3", "ec2").
    ///
    /// This is the service prefix that appears in IAM action names (e.g., "iam" in "iam:CreateUser").
    #[arg(long, required = true)]
    service_name: String,
}

impl ExpandSubCommand {
    /// Executes the expand subcommand to find and display all actions for a given service.
    ///
    /// This function builds a trie data structure from all available IAM actions for efficient
    /// prefix searching. It then queries the trie with the service name and optional action prefix
    /// to retrieve matching actions and displays them to the user.
    ///
    /// # Arguments
    ///
    /// * `available_services_permissions` - A HashMap mapping service prefixes to their associated
    ///   service and action data, typically populated from the AWS IAM actions JSON.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the command executes successfully
    /// - `Err(Box<dyn std::error::Error>)` if an error occurs during execution
    ///
    /// # Errors
    ///
    /// This function will exit the program with exit code 1 if the specified service is not found
    /// in the available services.
    fn handle(
        &self,
        available_services_permissions: HashMap<String, Vec<crate::types::AwsService>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "[+] Expanding AWS IAM actions for '{}' service...",
            self.service_name
        );

        let services_permissions: Vec<String> = available_services_permissions
            .values()
            .flatten()
            .flat_map(|service| service.actions.iter().map(|action| action.name.to_string()))
            .collect::<Vec<String>>();

        let trie = trie_rs::Trie::from_iter(services_permissions);
        let service_name = self.service_name.as_str();

        if !available_services_permissions.contains_key(service_name) {
            eprintln!("[!] Error: Service '{}' not found.", service_name);
            std::process::exit(1);
        }
        let trie_query = match &self.prefix {
            Some(prefix) => format!("{}:{}", service_name, prefix.replace('*', "")),
            None => format!("{}:", service_name),
        };
        trie.predictive_search(trie_query)
            .collect::<Vec<String>>()
            .into_iter()
            .for_each(|action| {
                println!("\t[-] {}", action);
            });

        Ok(())
    }
}

#[derive(Debug, clap::Args)]
pub struct ExpandFileSubCommand {
    /// Optional AWS file containing IAM policy to expand actions from.
    ///
    /// This argument conflicts with both `prefix` and `service_name` arguments.
    #[arg(long = "policy-file", required = true)]
    policy_file: String,

    /// Optional output file to save the expanded actions.
    ///
    /// This argument requires the `input_policy_file` argument to be specified.
    #[arg(long = "output-file", required = false, requires = "policy_file")]
    output_file: Option<String>,
}

impl ExpandFileSubCommand {
    /// Executes the expand-file subcommand to expand actions in a given IAM policy file.
    ///
    /// This function reads the specified IAM policy file, expands any wildcard actions
    /// using a trie built from available IAM actions, and outputs the expanded policy
    /// either to the console or to a specified output file.
    ///
    /// # Arguments
    ///
    /// * `available_services_permissions` - A HashMap mapping service prefixes to their associated
    ///  service and action data, typically populated from the AWS IAM actions JSON.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the command executes successfully
    /// - `Err(Box<dyn std::error::Error>)` if an error occurs during execution
    ///
    /// # Errors
    ///
    /// This function will return an error if there are issues reading the policy file,
    /// parsing the JSON content, or writing to the output file.
    fn handle(
        &self,
        available_services_permissions: HashMap<String, Vec<crate::types::AwsService>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut policy_content = serde_json::from_str::<crate::types::AWSPolicyDocument>(
            &std::fs::read_to_string(&self.policy_file)?,
        )?;

        let services_permissions: Vec<String> = available_services_permissions
            .values()
            .flatten()
            .flat_map(|service| service.actions.iter().map(|action| action.name.to_string()))
            .collect::<Vec<String>>();

        let trie = trie_rs::Trie::from_iter(services_permissions);

        for statement in &mut policy_content.statement {
            statement.action = serde_json::Value::Array(
                self.expand_actions(&statement.action, &trie)
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect::<Vec<serde_json::Value>>(),
            );

            if let Some(not_action) = &statement.not_action {
                statement.not_action = Some(serde_json::Value::Array(
                    self.expand_actions(not_action, &trie)
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect::<Vec<serde_json::Value>>(),
                ));
            }
        }

        self.output_results(&policy_content)
    }

    /// Expands actions from a serde_json::Value using the provided trie.
    ///
    /// Supports both string and array formats for actions.
    ///
    /// # Arguments
    ///
    /// * `action_value` - The serde_json::Value representing the action(s) to expand.
    /// * `trie` - The trie containing all available IAM actions for prefix searching.
    ///
    /// # Returns
    ///
    /// A vector of expanded action strings.
    fn expand_actions(
        &self,
        action_value: &serde_json::Value,
        trie: &trie_rs::Trie<u8>,
    ) -> Vec<String> {
        match action_value {
            serde_json::Value::String(action) => self.expand_string_actions(action, trie),
            serde_json::Value::Array(actions) => self.expand_array_actions(actions, trie),
            _ => {
                eprintln!("[!] Unsupported action format in policy.");
                Vec::new()
            }
        }
    }

    /// Expands actions from a string using the provided trie.
    ///
    /// # Arguments
    ///
    /// * `action_str` - The action string to expand.
    /// * `trie` - The trie containing all available IAM actions for prefix searching.
    /// # Returns
    ///
    /// A vector of expanded action strings.
    fn expand_string_actions(&self, action_str: &str, trie: &trie_rs::Trie<u8>) -> Vec<String> {
        trie.predictive_search(action_str.replace('*', ""))
            .collect::<Vec<String>>()
    }

    /// Expands actions from an array of serde_json::Value using the provided trie.
    ///
    /// # Arguments
    ///
    /// * `action_array` - The array of serde_json::Value representing the actions to expand.
    /// * `trie` - The trie containing all available IAM actions for prefix searching.
    ///
    /// # Returns
    ///
    /// A vector of expanded action strings.
    fn expand_array_actions(
        &self,
        action_array: &Vec<serde_json::Value>,
        trie: &trie_rs::Trie<u8>,
    ) -> Vec<String> {
        action_array
            .iter()
            .flat_map(|action_value| {
                if let serde_json::Value::String(action) = action_value {
                    self.expand_string_actions(action, trie)
                } else {
                    Vec::new()
                }
            })
            .collect::<std::collections::HashSet<String>>()
            .into_iter()
            .collect::<Vec<String>>()
    }

    /// Outputs the expanded policy results to either a file or the console.
    ///
    /// # Arguments
    ///
    /// * `policy_content` - The expanded AWS IAM policy document to output.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the output operation is successful
    /// - `Err(Box<dyn std::error::Error>)` if an error occurs during output operation
    fn output_results(
        &self,
        policy_content: &crate::types::AWSPolicyDocument,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.output_file.is_some() {
            println!(
                "[+] Writing expanded policy to file: {}",
                self.output_file.as_ref().unwrap()
            );
            std::fs::write(
                self.output_file.as_ref().unwrap(),
                serde_json::to_string_pretty(&policy_content)?,
            )?;
        } else {
            println!(
                "[*] Expanded Policy: {}",
                serde_json::to_string_pretty(&policy_content)?
            );
        }
        Ok(())
    }
}

/// Enumeration of available CLI subcommands.
///
/// This enum represents the different actions the user can perform with the toolkit.
/// Each variant corresponds to a specific operation that can be invoked from the command line.
#[derive(Debug, Subcommand)]
pub enum Action {
    /// List all available AWS services and their service prefixes.
    ///
    /// This command displays every AWS service available in the IAM actions database,
    /// showing the service prefix that can be used with the expand command.
    #[command(name = "list-services")]
    ListServices,

    /// Expand AWS IAM actions based on a service name and optional action prefix.
    ///
    /// This command shows all specific IAM actions available for a given AWS service,
    /// optionally filtered by an action name prefix.
    Expand(ExpandSubCommand),

    /// Expand AWS IAM actions from a specified policy file.
    ///
    /// This command reads an IAM policy file, expands any wildcard actions,
    /// and outputs the expanded policy either to the console or to a specified output file.
    #[command(name = "expand-file")]
    ExpandFile(ExpandFileSubCommand),
}

/// Represents the top-level command-line arguments and options.
///
/// This struct is the entry point for argument parsing and contains the subcommand
/// that the user wishes to execute.
#[derive(Debug, clap::Parser)]
pub struct Args {
    /// The subcommand to execute (either "list-services" or "expand").
    #[clap(subcommand)]
    action: Action,
}

impl Args {
    /// Processes and executes the parsed command-line arguments.
    ///
    /// This function dispatches to the appropriate handler based on the subcommand specified
    /// by the user. It uses the provided available services and permissions data to fulfill
    /// the requested operation.
    ///
    /// # Arguments
    ///
    /// * `available_services_permissions` - A HashMap mapping service prefixes to their associated
    ///   service and action data from the AWS IAM actions JSON.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the command executes successfully
    /// - `Err(Box<dyn std::error::Error>)` if an error occurs during execution
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let args = Args::parse();
    /// args.handle(available_services_permissions)?;
    /// ```
    pub fn handle(
        &self,
        available_services_permissions: HashMap<String, Vec<crate::types::AwsService>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &self.action {
            Action::ListServices => {
                println!("[*] Listing AWS IAM services");
                available_services_permissions
                    .keys()
                    .into_iter()
                    .for_each(|service_prefix| println!("\t[+] {}", service_prefix));

                Ok(())
            }
            Action::Expand(expand_sub_cmd) => expand_sub_cmd.handle(available_services_permissions),
            Action::ExpandFile(expand_file_sub_cmd) => {
                expand_file_sub_cmd.handle(available_services_permissions)
            }
        }
    }
}
