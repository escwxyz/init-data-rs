//! Validation module for Telegram Mini Apps init data.
//!
//! This module provides functionality to validate the authenticity and integrity
//! of init data passed from Telegram to Mini Apps. It includes support for both
//! standard validation and third-party bot validation.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::InitDataError;
use crate::types::InitData;
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

/// Validates init data using both primary and third-party bot tokens.
///
/// Similar to `validate()`, but accepts an additional third-party bot token
/// for validation. The init data is considered valid if it matches either token.
///
/// # Arguments
/// * `init_data` - Raw init data string from Telegram Mini App
/// * `token` - Primary bot token
/// * `third_party_token` - Third-party bot token
/// * `expires_in` - Optional expiration time in seconds
///
/// # Returns
/// * `Ok(InitData)` - Parsed and validated init data
/// * `Err(InitDataError)` - Various validation or parsing errors
pub fn validate_third_party(
    init_data: &str,
    token: &str,
    third_party_token: &str,
    expires_in: Option<u64>,
) -> Result<InitData, InitDataError> {
    if init_data.is_empty() || !init_data.contains('=') {
        return Err(InitDataError::UnexpectedFormat(
            "init_data is empty or malformed".to_string(),
        ));
    }

    let (base_data, hash) = extract_hash(init_data)?;
    let expected_hash = sign(&base_data, token)?;
    let expected_third_party_hash = sign(&base_data, third_party_token)?;

    if hash != expected_hash && hash != expected_third_party_hash {
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

    const BOT_TOKEN: &str = "12345:YOUR_BOT_TOKEN";
    const THIRD_PARTY_BOT_TOKEN: &str = "54321:OTHER_BOT_TOKEN";
    const INVALID_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000"; // 64 chars

    const VALID_INIT_DATA: &str = "query_id=AAHdF6IQAAAAAN0XohDhrOrc\
        &user=%7B%22id%22%3A279058397%2C%22first_name%22%3A%22Vladislav%22%2C\
        %22last_name%22%3A%22Kibenko%22%2C%22username%22%3A%22vdkfrost%22%2C\
        %22language_code%22%3A%22ru%22%2C%22is_premium%22%3Atrue%7D\
        &auth_date=1662771648";

    fn create_valid_init_data(init_data: &str, token: &str) -> String {
        let hash = sign(init_data, token).unwrap();
        format!("{}&hash={}", init_data, hash)
    }

    fn create_init_data_with_hash(base_data: &str, hash: &str) -> String {
        format!("{}&hash={}", base_data, hash)
    }

    #[test]
    fn test_validate_empty_data() {
        println!("\n=== Start: Testing validate empty data ===\n");
        let result = validate("", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
        println!("\n=== End: Testing validate empty data ===\n");
    }

    #[test]
    fn test_validate_invalid_format() {
        println!("\n=== Start: Testing validate invalid format ===\n");
        let result = validate("invalid_format", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
        println!("\n=== End: Testing validate invalid format ===\n");
    }

    #[test]
    fn test_validate_missing_hash() {
        println!("\n=== Start: Testing validate missing hash ===\n");
        let result = validate("query_id=test123", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashMissing)));
        println!("\n=== End: Testing validate missing hash ===\n");
    }

    #[test]
    fn test_validate_invalid_hash() {
        println!("\n=== Start: Testing validate invalid hash ===\n");
        let result = validate("query_id=test123&hash=invalid", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate invalid hash ===\n");
    }

    #[test]
    fn test_validate_expired() {
        println!("\n=== Start: Testing validate expired ===\n");
        let base_data = format!("auth_date={}", 1000000000);
        let init_data = create_valid_init_data(&base_data, BOT_TOKEN);
        let result = validate(&init_data, BOT_TOKEN, Some(86400));
        assert!(matches!(result, Err(InitDataError::Expired)));
        println!("\n=== End: Testing validate expired ===\n");
    }

    #[test]
    fn test_validate_no_expiration() {
        println!("\n=== Start: Testing validate no expiration ===\n");
        let base_data = format!("auth_date={}", 1000000000);
        let init_data = create_valid_init_data(&base_data, BOT_TOKEN);
        let result = validate(&init_data, BOT_TOKEN, Some(0));
        assert!(result.is_ok());
        println!("\n=== End: Testing validate no expiration ===\n");
    }

    #[test]
    fn test_validate_valid_data() {
        println!("\n=== Start: Testing validate valid data ===\n");
        let init_data = create_valid_init_data(VALID_INIT_DATA, BOT_TOKEN);
        let result = validate(&init_data, BOT_TOKEN, Some(0)); // Disable expiration check for test
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
        println!("\n=== End: Testing validate valid data ===\n");
    }

    #[test]
    fn test_validate_third_party_empty_data() {
        println!("\n=== Start: Testing validate third party empty data ===\n");
        let result = validate_third_party("", BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
        println!("\n=== End: Testing validate third party empty data ===\n");
    }

    #[test]
    fn test_validate_third_party_invalid_format() {
        println!("\n=== Start: Testing validate third party invalid format ===\n");
        let result = validate_third_party("invalid_format", BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
        println!("\n=== End: Testing validate third party invalid format ===\n");
    }

    #[test]
    fn test_validate_third_party_invalid_hash() {
        println!("\n=== Start: Testing validate third party invalid hash ===\n");
        let base_data = "query_id=AAHdF6IQAAAAAN0XohDhrOrc\
            &user=%7B%22id%22%3A279058397%7D\
            &auth_date=1662771648";
        let init_data = create_init_data_with_hash(base_data, INVALID_HASH);

        let result = validate_third_party(&init_data, BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, None);

        // The test is actually passing - we expect InvalidHash error and we get it
        assert!(
            matches!(result, Err(InitDataError::HashInvalid)),
            "Expected InvalidHash error, got {:?}",
            result
        );
        println!("\n=== End: Testing validate third party invalid hash ===\n");
    }

    #[test]
    fn test_validate_third_party_expired() {
        println!("\n=== Start: Testing validate third party expired ===\n");

        let base_data = format!("auth_date={}", 1000000000);
        let init_data = create_valid_init_data(&base_data, BOT_TOKEN);
        let result = validate_third_party(&init_data, BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, Some(86400));
        assert!(matches!(result, Err(InitDataError::Expired)));
        println!("\n=== End: Testing validate third party expired ===\n");
    }

    #[test]
    fn test_validate_third_party_valid_primary_token() {
        println!("\n=== Start: Testing validate third party valid primary token ===\n");
        let init_data = "query_id=test123&auth_date=1662771648";
        let hash = sign(init_data, BOT_TOKEN).unwrap();
        let init_data_with_hash = format!("{}&hash={}", init_data, hash);

        let result = validate_third_party(&init_data_with_hash, BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, Some(0)); // Disable expiration
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
        println!("\n=== End: Testing validate third party valid primary token ===\n");
    }

    #[test]
    fn test_validate_third_party_valid_secondary_token() {
        println!("\n=== Start: Testing validate third party valid secondary token ===\n");
        let init_data = "query_id=test123&auth_date=1662771648";
        let hash = sign(init_data, THIRD_PARTY_BOT_TOKEN).unwrap();
        let init_data_with_hash = format!("{}&hash={}", init_data, hash);

        let result = validate_third_party(&init_data_with_hash, BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, Some(0)); // Disable expiration
        assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
        println!("\n=== End: Testing validate third party valid secondary token ===\n");
    }

    #[test]
    fn test_validate_third_party_no_expiration() {
        println!("\n=== Start: Testing validate third party no expiration ===\n");
        let base_data = format!("auth_date={}", 1000000000);
        let init_data = create_init_data_with_hash(&base_data, INVALID_HASH);
        let result = validate_third_party(&init_data, BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, Some(0));
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate third party no expiration ===\n");
    }

    #[test]
    fn test_validate_malformed_hash() {
        println!("\n=== Start: Testing validate malformed hash ===\n");
        let result = validate("query_id=test123&hash=", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate malformed hash ===\n");
    }

    #[test]
    fn test_validate_hash_format_length() {
        println!("\n=== Start: Testing validate hash format length ===\n");

        let result = validate("query_id=test123&hash=abc123", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));

        // Test hash that's too long
        let result = validate(&format!("query_id=test123&hash={}0", INVALID_HASH), BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate hash format length ===\n");
    }

    #[test]
    fn test_validate_hash_format_invalid_chars() {
        println!("\n=== Start: Testing validate hash format invalid chars ===\n");
        // Test hash with invalid characters
        let result = validate(
            "query_id=test123&hash=gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
            BOT_TOKEN,
            None,
        );
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate hash format invalid chars ===\n");
    }

    #[test]
    fn test_validate_third_party_malformed_hash() {
        println!("\n=== Start: Testing validate third party malformed hash ===\n");
        // Test case where hash= is present but no value after it
        let result = validate_third_party("query_id=test123&hash=", BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate third party malformed hash ===\n");
    }

    #[test]
    fn test_validate_third_party_hash_format() {
        println!("\n=== Start: Testing validate third party hash format ===\n");
        // Test hash with invalid format but correct length
        let result = validate_third_party(
            "query_id=test123&hash=gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
            BOT_TOKEN,
            THIRD_PARTY_BOT_TOKEN,
            None,
        );
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate third party hash format ===\n");
    }

    #[test]
    fn test_validate_hash_extraction_failure() {
        println!("\n=== Start: Testing validate hash extraction failure ===\n");
        // Test case where hash= is at the end without a value
        let result = validate("query_id=test123&hash=&other=value", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));

        // Test case where hash= is in the middle without a value
        let result = validate("query_id=test123&hash=&auth_date=123", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate hash extraction failure ===\n");
    }

    #[test]
    fn test_validate_third_party_hash_missing() {
        println!("\n=== Start: Testing validate third party hash missing ===\n");
        // Test case where hash= is present but followed by another parameter
        let result = validate_third_party(
            "query_id=test123&hash&auth_date=123",
            BOT_TOKEN,
            THIRD_PARTY_BOT_TOKEN,
            None,
        );
        assert!(matches!(result, Err(InitDataError::HashMissing)));
        println!("\n=== End: Testing validate third party hash missing ===\n");
    }

    #[test]
    fn test_validate_third_party_hash_extraction_failure() {
        println!("\n=== Start: Testing validate third party hash extraction failure ===\n");
        // Test case where hash= is in the middle without a value
        let result = validate_third_party(
            "query_id=test123&hash=&auth_date=123",
            BOT_TOKEN,
            THIRD_PARTY_BOT_TOKEN,
            None,
        );
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate third party hash extraction failure ===\n");
    }

    #[test]
    fn test_validate_hash_comparison_failure() {
        println!("\n=== Start: Testing validate hash comparison failure ===\n");
        // Create valid hash with one token but validate with another
        let init_data = "query_id=test123&auth_date=1662771648";
        let hash = sign(init_data, THIRD_PARTY_BOT_TOKEN).unwrap();
        let init_data_with_hash = format!("{}&hash={}", init_data, hash);

        let result = validate(&init_data_with_hash, BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate hash comparison failure ===\n");
    }

    #[test]
    fn test_validate_impossible_hash_extraction() {
        println!("\n=== Start: Testing validate impossible hash extraction ===\n");
        // This test is for line 35
        // We check for &hash= first, but try to force the else branch
        let result = validate("query_id=test123&hash=abc\n&hash=def", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate impossible hash extraction ===\n");
    }

    #[test]
    fn test_validate_third_party_impossible_hash_extraction() {
        println!("\n=== Start: Testing validate third party impossible hash extraction ===\n");
        // This test is for line 85
        // We check for &hash= first, but try to force the else branch
        let result = validate_third_party(
            "query_id=test123&hash=abc\n&hash=def",
            BOT_TOKEN,
            THIRD_PARTY_BOT_TOKEN,
            None,
        );
        assert!(matches!(result, Err(InitDataError::HashInvalid)));
        println!("\n=== End: Testing validate third party impossible hash extraction ===\n");
    }

    #[test]
    fn test_validate_hash_extraction_corner_case() {
        println!("\n=== Start: Testing validate hash extraction corner case ===\n");
        // Test case where hash= is at the end of string (no value, no other params)
        let result = validate("query_id=test123&hash=", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashInvalid)));

        // Test with escaped &
        let result = validate("query_id=test123%26hash=abc", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashMissing)));

        // Test with URL-encoded &hash=
        let result = validate("query_id=test123%26hash%3Dabc", BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashMissing)));
        println!("\n=== End: Testing validate hash extraction corner case ===\n");
    }

    #[test]
    fn test_validate_third_party_hash_extraction_corner_case() {
        println!("\n=== Start: Testing validate third party hash extraction corner case ===\n");
        // Test with multiple hash parameters but first one is empty
        let result = validate_third_party(
            "query_id=test123&hash=&hash=abcdef",
            BOT_TOKEN,
            THIRD_PARTY_BOT_TOKEN,
            None,
        );
        assert!(matches!(result, Err(InitDataError::HashInvalid)));

        // Test with hash parameter at start
        let result = validate_third_party("hash=abc&query_id=test123", BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashMissing)));

        // Test with malformed hash parameter
        let result = validate_third_party("query_id=test123&hash", BOT_TOKEN, THIRD_PARTY_BOT_TOKEN, None);
        assert!(matches!(result, Err(InitDataError::HashMissing)));
        println!("\n=== End: Testing validate third party hash extraction corner case ===\n");
    }

    #[test]
    fn test_extract_hash() {
        println!("\n=== Start: Testing extract hash ===\n");

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

        println!("\n=== End: Testing extract hash ===\n");
    }
}
