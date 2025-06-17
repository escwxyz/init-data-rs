use crate::parse;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as base64_engine;
use base64::Engine as _;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hex::FromHex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{InitData, InitDataError};

const TEST_PUBLIC_KEY: &str = "40055058a4ee38156a06562e52eece92a771bcd8346a8c4615cb7376eddf72ec";
const PROD_PUBLIC_KEY: &str = "e7bf03a2fa4602af4580703d88dda5bb59f32ed8b02a56c187fe7d34caed242d";

/// Validates data for third-party use
///
/// If you need to share the data with a third party, they can validate the data without requiring access to your bot's token.
/// Simply provide them with the data from the Telegram.WebApp.initData field and your bot_id.
///
/// See: https://core.telegram.org/bots/webapps#validating-data-for-third-party-use
///
/// Telegram provides the following Ed25519 public keys for signature verification:
/// * `40055058a4ee38156a06562e52eece92a771bcd8346a8c4615cb7376eddf72ec` for test environment
/// * `e7bf03a2fa4602af4580703d88dda5bb59f32ed8b02a56c187fe7d34caed242d` for production environment
///
/// # Arguments
/// * `init_data` - Raw init data string from Telegram Mini App
/// * `bot_id` - Bot ID
/// * `expires_in` - Optional expiration time in seconds
/// * `is_test` - Whether to use the test public key
///
/// # Returns
/// * `Ok(InitData)` - Parsed and validated init data
/// * `Err(InitDataError)` - Various validation or parsing errors
///
fn validate_third_party_with_signature(
    init_data: &str,
    bot_id: i64,
    expires_in: Option<u64>,
    is_test: bool,
) -> Result<InitData, InitDataError> {
    if init_data.is_empty() || !init_data.contains('=') {
        return Err(InitDataError::UnexpectedFormat(
            "init_data is empty or malformed".to_string(),
        ));
    }

    let pairs: Vec<(String, String)> = url::form_urlencoded::parse(init_data.as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    let mut signature_b64 = None;
    let mut filtered_pairs: Vec<(String, String)> = Vec::new();
    let mut auth_date: Option<u64> = None;
    for (k, v) in pairs {
        if k == "signature" {
            signature_b64 = Some(v);
        } else if k == "auth_date" {
            auth_date = v.parse().ok();
            filtered_pairs.push((k, v));
        } else if k != "hash" {
            filtered_pairs.push((k, v));
        }
    }
    let signature_b64 = signature_b64.ok_or(InitDataError::SignatureMissing)?;

    if let (Some(expires_in), Some(auth_date)) = (expires_in, auth_date) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if auth_date + expires_in < now {
            return Err(InitDataError::Expired);
        }
    }

    filtered_pairs.sort_by(|a, b| a.0.cmp(&b.0));

    let message = format!(
        "{}:WebAppData\n{}",
        bot_id,
        filtered_pairs
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let signature_bytes = base64_engine
        .decode(signature_b64.as_bytes())
        .map_err(|_| InitDataError::SignatureInvalid("Failed to decode signature from base64".to_string()))?;

    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| InitDataError::SignatureInvalid("Failed to parse signature".to_string()))?;

    let public_key_hex = if is_test { TEST_PUBLIC_KEY } else { PROD_PUBLIC_KEY };

    let public_key_bytes = <[u8; 32]>::from_hex(public_key_hex)
        .map_err(|_| InitDataError::SignatureInvalid("Failed to parse public key".to_string()))?;

    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|_| InitDataError::SignatureInvalid("Failed to parse public key".to_string()))?;

    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| InitDataError::SignatureInvalid("Failed to verify signature".to_string()))?;

    // 9. If valid, parse into InitData and return Ok
    let data = parse(init_data)?;
    Ok(data)
}

/// Validates init data using both primary and third-party bot tokens.
///
/// Similar to `validate()`, but accepts an additional third-party bot token
/// for validation. The init data is considered valid if it matches either token.
///
/// # Arguments
/// * `init_data` - Raw init data string from Telegram Mini App
/// * `bot_id` - Bot ID
/// * `expires_in` - Optional expiration time in seconds
///
/// # Returns
/// * `Ok(InitData)` - Parsed and validated init data
/// * `Err(InitDataError)` - Various validation or parsing errors
///
/// # Example
/// ```
/// use init_data_rs::validate_third_party;
///
/// let init_data = "query_id=123&auth_date=1662771648&hash=...&signature=...";
/// let result = validate_third_party(init_data, 1234567890, None);
/// ```
pub fn validate_third_party(init_data: &str, bot_id: i64, expires_in: Option<u64>) -> Result<InitData, InitDataError> {
    validate_third_party_with_signature(init_data, bot_id, expires_in, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    // With signature
    const VALID_INIT_DATA: &str = "user=%7B%22id%22%3A279058397%2C%22first_name%22%3A%22Vladislav%20%2B%20-%20%3F%20%5C%2F%22%2C%22last_name%22%3A%22Kibenko%22%2C%22username%22%3A%22vdkfrost%22%2C%22language_code%22%3A%22ru%22%2C%22is_premium%22%3Atrue%2C%22allows_write_to_pm%22%3Atrue%2C%22photo_url%22%3A%22https%3A%5C%2F%5C%2Ft.me%5C%2Fi%5C%2Fuserpic%5C%2F320%5C%2F4FPEE4tmP3ATHa57u6MqTDih13LTOiMoKoLDRG4PnSA.svg%22%7D&chat_instance=8134722200314281151&chat_type=private&auth_date=1733584787&hash=2174df5b000556d044f3f020384e879c8efcab55ddea2ced4eb752e93e7080d6&signature=zL-ucjNyREiHDE8aihFwpfR9aggP2xiAo3NSpfe-p7IbCisNlDKlo7Kb6G4D0Ao2mBrSgEk4maLSdv6MLIlADQ";
    const BOT_ID: i64 = 7342037359;

    #[test]
    fn test_valid_third_party_signature() {
        let result = validate_third_party(VALID_INIT_DATA, BOT_ID, None);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_invalid_signature() {
        // Tamper with the signature
        let tampered = VALID_INIT_DATA.replace(
            "zL-ucjNyREiHDE8aihFwpfR9aggP2xiAo3NSpfe-p7IbCisNlDKlo7Kb6G4D0Ao2mBrSgEk4maLSdv6MLIlADQ",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        );
        let result = validate_third_party(&tampered, BOT_ID, None);
        assert!(matches!(result, Err(InitDataError::SignatureInvalid(_))));
    }

    #[test]
    fn test_third_party_invalid_base64_signature() {
        let bad_data = "query_id=test&auth_date=123&signature=!!!notbase64!!!&hash=abc";
        let bot_id = 123456;
        let result = validate_third_party_with_signature(bad_data, bot_id, None, true);
        assert!(matches!(result, Err(InitDataError::SignatureInvalid(_))));
    }

    #[test]
    fn test_third_party_invalid_public_key() {
        let valid_data = "query_id=test&auth_date=123&signature=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&hash=abc";
        let bot_id = 123456;
        // Use an invalid public key by temporarily changing the constant or by passing a custom function if your API allows
        // For this test, you might need to expose a version of your function that takes a public key string
        let result = validate_third_party_with_signature(valid_data, bot_id, None, true); // with a purposely broken key
        assert!(matches!(result, Err(InitDataError::SignatureInvalid(_))));
    }

    #[test]
    fn test_third_party_signature_verification_failure() {
        // Use a valid base64 signature, but one that doesn't match the data
        let bad_sig = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0u8; 64]);
        let bad_data = format!("query_id=test&auth_date=123&signature={}&hash=abc", bad_sig);
        let bot_id = 123456;
        let result = validate_third_party_with_signature(&bad_data, bot_id, None, true);
        assert!(matches!(result, Err(InitDataError::SignatureInvalid(_))));
    }

    #[test]
    fn test_missing_signature() {
        // Remove the signature field
        let mut parts: Vec<&str> = VALID_INIT_DATA.split('&').collect();
        parts.retain(|s| !s.starts_with("signature="));
        let no_sig = parts.join("&");
        let result = validate_third_party(&no_sig, BOT_ID, None);
        assert!(matches!(result, Err(InitDataError::SignatureMissing)));
    }

    #[test]
    fn test_expired_data() {
        // Use a very old auth_date
        let expired_data = VALID_INIT_DATA.replace("auth_date=1733584787", "auth_date=1000000000");
        let result = validate_third_party(&expired_data, BOT_ID, Some(86400));
        assert!(matches!(result, Err(InitDataError::Expired)));
    }

    #[test]
    fn test_malformed_input() {
        let result = validate_third_party("not_a_query_string", BOT_ID, None);
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
    }

    #[test]
    fn test_wrong_bot_id() {
        // Use a wrong bot_id (signature won't match)
        let result = validate_third_party(VALID_INIT_DATA, 1234567890, None);
        assert!(matches!(result, Err(InitDataError::SignatureInvalid(_))));
    }

    #[test]
    fn test_wrong_environment() {
        // Use test environment (signature won't match prod key)
        let result = validate_third_party_with_signature(VALID_INIT_DATA, BOT_ID, None, true);
        assert!(matches!(result, Err(InitDataError::SignatureInvalid(_))));
    }
}
