# AWS IAM Expansion Toolkit

A Rust-based toolkit for expanding AWS IAM policy wildcards and discovering specific permissions behind wildcard service specifications.

## Overview

The AWS IAM Expansion Toolkit helps security professionals and developers understand which exact AWS IAM actions are available for a given service. This is particularly useful when dealing with wildcard permissions in IAM policies (e.g., `s3:*` or `iam:*`) to understand the full scope of access being granted.

The toolkit fetches the complete list of AWS IAM actions and services from the [AWS IAM Actions API](https://www.awsiamactions.io/) and provides an efficient command-line interface to:

- **List all available AWS services** with their service prefixes
- **Expand IAM actions** for a specific service, optionally filtered by action name prefix
- **Efficiently search** for actions using trie-based prefix matching
- **Cache data locally** to avoid redundant API calls

## Command Line Usage

### Prerequisites

- Rust 1.70 or later
- Internet connection (for initial data fetch)

### Installation

```bash
git clone <repository-url>
cd aws-iam-expansion
cargo build --release
```

### Commands

#### List All AWS Services

Display all available AWS services and their service prefixes:

```bash
aws-iam-expansion list-services
```

Example output:
```
[*] Listing AWS IAM services
	[+] ec2
	[+] s3
	[+] iam
	[+] lambda
	...
```

#### Expand Actions for a Service

Expand all IAM actions available for a specific AWS service:

```bash
aws-iam-expansion expand --service-name iam
```

Example output:
```
[+] Expanding AWS IAM actions for 'iam' service...
	[-] iam:AddClientIDToOpenIDConnectProvider
	[-] iam:AddRoleToInstanceProfile
	[-] iam:AddUserToGroup
	...
```

#### Expand Actions with Prefix Filter

Filter expanded actions by a specific action name prefix:

```bash
aws-iam-expansion expand --service-name iam --prefix Create
```

Example output:
```
[+] Expanding AWS IAM actions for 'iam' service...
	[-] iam:CreateAccessKey
	[-] iam:CreateGroup
	[-] iam:CreateInstanceProfile
	[-] iam:CreateLoginProfile
	[-] iam:CreateOpenIDConnectProvider
	[-] iam:CreateRole
	[-] iam:CreateUser
	...
```

### Data Caching

The toolkit automatically caches the AWS IAM actions data locally at:

```
~/.cache/aws_iam_expansion/aws_iam_actions.json
```

On the first run, the data is fetched from the AWS IAM Actions API. Subsequent runs use the cached data unless you manually delete the cache file.

To force a fresh data fetch, simply delete the cache:

```bash
rm ~/.cache/aws_iam_expansion/aws_iam_actions.json
```

## Features

- **Efficient Search**: Uses trie data structure for fast prefix-based searching
- **Local Caching**: Caches API responses to minimize network requests
- **Simple CLI**: Intuitive command-line interface using the `clap` framework
- **Comprehensive Documentation**: Fully documented code with extensive docstrings

## Use Cases

- Audit IAM policies to understand the scope of wildcard permissions
- Discover available actions when creating least-privilege policies
- Generate complete action lists for security documentation
- Understand which permissions are granted by wildcard service specs

## TODOs

- [ ] Delete cache command
- [ ] Update to latest AWS IAM Actions API endpoint if it changes

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Any contributions are welcome! Please open issues or submit pull requests for bug fixes, enhancements, or new features.
