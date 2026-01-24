//! Utility functions for retrieving AWS IAM actions data.
//!
//! This module provides functionality to fetch and cache AWS IAM actions data from
//! the AWS IAM Actions JSON API. It manages local caching to reduce network requests
//! and improve performance on subsequent runs.

const AWS_IAM_ACTIONS_URL: &str = "https://www.awsiamactions.io/json";

/// Determines the file path for the AWS IAM actions cache.
///
/// Creates and returns the path to the cache file in the user's home directory
/// at `~/.cache/aws_iam_expansion/aws_iam_actions.json`. The cache directory is
/// created if it doesn't already exist.
///
/// # Panics
///
/// Panics if the cache directory cannot be created due to filesystem permission issues.
///
/// # Returns
///
/// A `String` containing the absolute path to the cache file.
fn cache_file_path() -> String {
    let home_dir = std::path::PathBuf::from(shellexpand::tilde("~").to_string());
    let cache_dir = home_dir.join(".cache/aws_iam_expansion");
    std::fs::create_dir_all(&cache_dir).expect("Could not create cache directory");
    cache_dir
        .join("aws_iam_actions.json")
        .to_str()
        .unwrap()
        .to_string()
}

/// Retrieves the AWS IAM actions JSON data, using cache when available.
///
/// This function first checks if a cached copy of AWS IAM actions exists locally.
/// If it does, the cached version is returned immediately. Otherwise, it fetches
/// the data from the AWS IAM Actions API, caches it for future use, and returns it.
///
/// The cache is stored at `~/.cache/aws_iam_expansion/aws_iam_actions.json`.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(String)`: The JSON string containing all available AWS IAM actions and services
/// - `Err(Box<dyn std::error::Error>)`: An error if the request fails or file operations fail
///
/// # Examples
///
/// ```no_run
/// let iam_actions_json = retrieve_iam_actions_json()?;
/// let services = serde_json::from_str::<Vec<AwsService>>(&iam_actions_json)?;
/// ```
pub fn retrieve_iam_actions_json() -> Result<String, Box<dyn std::error::Error>> {
    let cache_path = cache_file_path();
    if std::path::Path::new(&cache_path).exists() {
        println!("[*] Using cached AWS IAM actions data...");
        let cached_data = std::fs::read_to_string(&cache_path)?;
        Ok(cached_data)
    } else {
        println!("[*] Fetching AWS IAM actions...");
        let iam_actions_json = reqwest::blocking::Client::new()
            .get(AWS_IAM_ACTIONS_URL)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:146.0) Gecko/20100101 Firefox/146.0",
            )
            .send()?
            .text()?;
        std::fs::write(&cache_path, &iam_actions_json)?;
        Ok(iam_actions_json)
    }
}

/// Deletes the cached AWS IAM actions data file.
///
/// This function removes the cache file located at `~/.cache/aws_iam_expansion/aws_iam_actions.json`
/// if it exists. This is useful for forcing a fresh fetch of the latest AWS IAM actions data
/// from the API on the next retrieval.
///
/// # Returns
/// A `Result` indicating success or failure of the deletion operation.
///
/// # Examples
///
/// ```no_run
/// delete_iam_actions_cache()?;
/// ```
pub fn delete_iam_actions_cache() -> Result<(), Box<dyn std::error::Error>> {
    let cache_path = cache_file_path();
    if std::path::Path::new(&cache_path).exists() {
        std::fs::remove_file(&cache_path)?;
        println!("[*] Deleted AWS IAM actions cache.");
    } else {
        println!("[!] No AWS IAM actions cache found to delete.");
    }
    Ok(())
}

/// Updates the cached AWS IAM actions data by fetching the latest version.
///
/// This function deletes the existing cache file (if any) and retrieves the latest
/// AWS IAM actions data from the API, storing it in the cache for future use.
///
/// # Returns
///
/// A `Result` indicating success or failure of the update operation.
///
/// # Examples
///
/// ```no_run
/// update_iam_actions_cache()?;
/// ```
pub fn update_iam_actions_cache() -> Result<(), Box<dyn std::error::Error>> {
    delete_iam_actions_cache()?;
    retrieve_iam_actions_json()?;
    println!("[*] Updated AWS IAM actions cache.");
    Ok(())
}
