//! Type definitions for AWS IAM services and actions.
//!
//! This module defines the core data structures used to represent AWS IAM services
//! and their associated actions, which are deserialized from the AWS IAM actions JSON.

/// Represents a single AWS IAM action.
///
/// An action is a specific permission that can be granted or denied in an AWS IAM policy.
/// Actions are associated with a particular service and have a type indicating their
/// category (e.g., "Read", "Write", "List", etc.).
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AwsAction {
    /// The name of the action (e.g., "s3:GetObject", "ec2:DescribeInstances").
    ///
    /// Deserialized from the "action" field in the JSON source.
    #[serde(rename = "action")]
    pub name: String,

    /// The type or category of the action (e.g., "Read", "Write", "List", "Tagging", "Permission management").
    ///
    /// Deserialized from the "type" field in the JSON source.
    #[serde(rename = "type")]
    pub action_type: String,
}

/// Represents an AWS service and its available actions.
///
/// A service is a high-level AWS offering (e.g., S3, EC2, IAM) that contains multiple
/// related IAM actions. This structure maps a service to all the actions that can be
/// performed on it.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AwsService {
    /// The full name of the AWS service (e.g., "Amazon Simple Storage Service").
    ///
    /// Deserialized from the "service" field in the JSON source.
    #[serde(rename = "service")]
    pub name: String,

    /// The service prefix used in IAM action names (e.g., "s3", "ec2", "iam").
    ///
    /// This prefix is used as the first part of an IAM action ARN-like string
    /// (e.g., "s3:GetObject" where "s3" is the prefix).
    /// Deserialized from the "servicePrefix" field in the JSON source.
    #[serde(rename = "servicePrefix")]
    pub prefix: String,

    /// A collection of all IAM actions available for this service.
    ///
    /// Each action represents a specific permission that can be granted or denied
    /// in an IAM policy for this service.
    #[serde(rename = "actions")]
    pub actions: Vec<AwsAction>,
}
