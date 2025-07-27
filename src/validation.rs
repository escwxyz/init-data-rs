//! Validation module for Telegram Mini Apps init data.
//!
//! This module provides functionality to validate the authenticity and integrity
//! of init data passed from Telegram to Mini Apps. It includes support for both
//! standard validation and third-party bot validation.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::InitDataError;
use crate::model::InitData;
use crate::{parse, sign};

/// Default expiration time for init data in seconds (24 hours)
const DEFAULT_EXPIRATION: u64 = 86400;

/// Extracts and validates the hash from init data string.
///
/// # Arguments
/// * `init_data` - The raw init data string containing the hash
///
/// # Returns
/// * `Ok((base_data, hash))` - Tuple containing the base data and valid hash
/// * `Err(InitDataError)` - Error if hash is missing, invalid, or malformed
fn extract_hash(init_data: &str) -> Result<(String, String), InitDataError> {
    let (base_data, hash) = if let Some(pos) = init_data.find("&hash=") {
        let (base, hash_part) = init_data.split_at(pos);
        let hash = &hash_part[6..]; // Skip "&hash="
        (base.to_string(), hash.to_string())
    } else {
        return Err(InitDataError::HashMissing);
    };

    if !hash.chars().all(|c| c.is_ascii_hexdigit()) || hash.len() != 64 {
        return Err(InitDataError::HashInvalid);
    }

    Ok((base_data, hash))
}

/// Validates the authenticity and integrity of Telegram Mini Apps init data.
///
/// This function performs several checks:
/// 1. Validates the format of the init data
/// 2. Extracts and verifies the hash
/// 3. Checks the data hasn't expired
/// 4. Parses the data into a strongly-typed structure
///
/// # Arguments
/// * `init_data` - Raw init data string from Telegram Mini App
/// * `token` - Bot token used for validation
/// * `expires_in` - Optional expiration time in seconds (defaults to 24 hours), set to 0 to disable expiration check
///
/// # Returns
/// * `Ok(InitData)` - Parsed and validated init data
/// * `Err(InitDataError)` - Various validation or parsing errors
///
/// # Example
/// ```
/// use init_data_rs::validate;
///
/// let init_data = "query_id=123&auth_date=1662771648&hash=...";
/// let result = validate(init_data, "BOT_TOKEN", None);
/// ```
pub fn validate(init_data: &str, token: &str, expires_in: Option<u64>) -> Result<InitData, InitDataError> {
    if init_data.is_empty() || !init_data.contains('=') {
        return Err(InitDataError::UnexpectedFormat(
            "init_data is empty or malformed".to_string(),
        ));
    }

    let (base_data, hash) = extract_hash(init_data)?;

    let expected_hash = sign(&base_data, token)?;

    if hash != expected_hash {
        return Err(InitDataError::HashInvalid);
    }

    let data = parse(init_data)?;

    let expires_in = expires_in.unwrap_or(DEFAULT_EXPIRATION);
    if expires_in > 0 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        if data.auth_date + expires_in < now {
            return Err(InitDataError::Expired);
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOT_TOKEN: &str = "5768337691:AAH5YkoiEuPk8-FZa32hStHTqXiLPtAEhx8";
    const INVALID_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    // Without signature
    const VALID_INIT_DATA: &str = "query_id=AAHdF6IQAAAAAN0XohDhrOrc&user=%7B%22id%22%3A279058397%2C%22first_name%22%3A%22Vladislav%22%2C%22last_name%22%3A%22Kibenko%22%2C%22username%22%3A%22vdkfrost%22%2C%22language_code%22%3A%22ru%22%2C%22is_premium%22%3Atrue%7D&auth_date=1662771648&hash=c501b71e775f74ce10e377dea85a7ea24ecd640b223ea86dfe453e0eaed2e2b2";

    #[test]
    fn test_validate_empty_data() {
        let result = validate("", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
    }

    #[test]
    fn test_validate_invalid_format() {
        let result = validate("invalid_format", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
    }

    #[test]
    fn test_validate_missing_hash() {
        let data = "query_id=test&auth_date=123";
        let token = "valid:token";
        let result = validate(data, token, None);
        assert!(matches!(result, Err(InitDataError::HashMissing)));
    }

    #[test]
    fn test_validate_invalid_hash() {
        let result = validate("query_id=test123&hash=invalid", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
    }

    #[test]
    fn test_validate_expired() {
        let base_data = VALID_INIT_DATA
            .replace("auth_date=1662771648", "auth_date=1000000000")
            .replace(
                "hash=c501b71e775f74ce10e377dea85a7ea24ecd640b223ea86dfe453e0eaed2e2b2",
                "",
            );

        let hash = sign(&base_data, BOT_TOKEN).unwrap();
        let init_data = format!("{}&hash={}", base_data, hash);
        let result = validate(&init_data, BOT_TOKEN, Some(86400));
        assert!(matches!(result, Err(InitDataError::Expired)));
    }

    #[test]
    fn test_validate_no_expiration() {
        let base_data = VALID_INIT_DATA
            .replace("auth_date=1662771648", "auth_date=1000000000")
            .replace(
                "hash=c501b71e775f74ce10e377dea85a7ea24ecd640b223ea86dfe453e0eaed2e2b2",
                "",
            );

        let hash = sign(&base_data, BOT_TOKEN).unwrap();
        let init_data = format!("{}&hash={}", base_data, hash);
        let result = validate(&init_data, BOT_TOKEN, Some(0));
        println!("result: {:?}", result);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_valid_data() {
        let result = validate(VALID_INIT_DATA, BOT_TOKEN, Some(0)); // Disable expiration check for test

        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.auth_date, 1662771648);
        assert!(data.user.is_some());

        if let Some(user) = data.user {
            assert_eq!(user.id, 279058397);
            assert_eq!(user.first_name, "Vladislav");
            assert_eq!(user.last_name, Some("Kibenko".to_string()));
            assert_eq!(user.username, Some("vdkfrost".to_string()));
            assert_eq!(user.language_code, Some("ru".to_string()));
            assert_eq!(user.is_premium, Some(true));
        }
    }

    #[test]
    fn test_validate_malformed_hash() {
        let result = validate("query_id=test123&hash=", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
    }

    #[test]
    fn test_validate_hash_format_length() {
        let result = validate("query_id=test123&hash=abc123", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));

        // Test hash that's too long
        let result = validate(&format!("query_id=test123&hash={}0", INVALID_HASH), BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
    }

    #[test]
    fn test_validate_hash_format_invalid_chars() {
        // Test hash with invalid characters
        let result = validate(
            "query_id=test123&hash=gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
            BOT_TOKEN,
            None,
        );
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
    }

    #[test]
    fn test_validate_hash_extraction_failure() {
        // Test case where hash= is at the end without a value
        let result = validate("query_id=test123&hash=&other=value", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));

        // Test case where hash= is in the middle without a value
        let result = validate("query_id=test123&hash=&auth_date=123", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
    }

    #[test]
    fn test_validate_impossible_hash_extraction() {
        // This test is for line 35
        // We check for &hash= first, but try to force the else branch
        let result = validate("query_id=test123&hash=abc\n&hash=def", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
    }

    #[test]
    fn test_validate_hash_extraction_corner_case() {
        // Test case where hash= is at the end of string (no value, no other params)
        let result = validate("query_id=test123&hash=", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));

        // Test with escaped &
        let result = validate("query_id=test123%26hash=abc", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashMissing)));

        // Test with URL-encoded &hash=
        let result = validate("query_id=test123%26hash%3Dabc", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashMissing)));
    }

    #[test]
    fn test_extract_hash() {
        // Test valid hash extraction
        let init_data = "query_id=test123&hash=1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = extract_hash(init_data);
        assert!(result.is_ok());
        let (base, hash) = result.unwrap();
        assert_eq!(base, "query_id=test123");
        assert_eq!(hash, "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");

        // Test missing hash
        let result = extract_hash("query_id=test123");
        assert!(matches!(result, Err(InitDataError::HashMissing)));

        // Test invalid hash format
        let result = extract_hash("query_id=test123&hash=invalid");
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
    }

    #[test]
    fn test_validate_incorrect_hash() {
        let base_data = "query_id=AAHdF6IQAAAAAN0XohDhrOrc&user=%7B%22id%22%3A279058397%2C%22first_name%22%3A%22Vladislav%22%2C%22last_name%22%3A%22Kibenko%22%2C%22username%22%3A%22vdkfrost%22%2C%22language_code%22%3A%22ru%22%2C%22is_premium%22%3Atrue%7D&auth_date=1662771648";
        // Use an obviously invalid hash (all zeros)
        let invalid_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let init_data = format!("{}&hash={}", base_data, invalid_hash);
        let result = validate(&init_data, BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
    }
}
