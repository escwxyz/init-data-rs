use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::collections::BTreeMap;
use url::form_urlencoded;

use crate::error::InitDataError;

/// Sign creates hash for init data using bot token.
/// 
/// # Errors
/// 
/// See `init_data_rs::parse` for possible errors
pub fn sign(init_data: &str, token: &str) -> Result<String, InitDataError> {
    if init_data.is_empty() {
        return Err(InitDataError::UnexpectedFormat("init_data is empty".to_string()));
    }

    if token.is_empty() {
        return Err(InitDataError::UnexpectedFormat("token is empty".to_string()));
    }

    let pairs = form_urlencoded::parse(init_data.as_bytes());
    let mut params: BTreeMap<String, String> = BTreeMap::new();

    for (key, value) in pairs {
        if key != "hash" {
            params.insert(key.to_string(), value.into_owned());
        }
    }

    let data_check_string = params
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    // More : https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app

    let mut hmac: Hmac<Sha256> = hmac::Hmac::new_from_slice("WebAppData".as_bytes())
        .map_err(|error| InitDataError::Internal(error.to_string()))?;

    hmac.update(token.as_bytes());

    let secret_key = hmac.finalize();

    let mut hmac: Hmac<Sha256> = hmac::Hmac::new_from_slice(secret_key.as_bytes())
        .map_err(|error| InitDataError::Internal(error.to_string()))?;

    hmac.update(data_check_string.as_bytes());

    Ok(hex::encode(hmac.finalize().as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOT_TOKEN: &str = "12345:YOUR_BOT_TOKEN";

    #[test]
    fn test_sign_empty_data() {
        let result = sign("", BOT_TOKEN);
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
    }

    #[test]
    fn test_sign_empty_token() {
        let base_data = "query_id=test&auth_date=123";
        let result = sign(base_data, "");
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
    }

    #[test]
    fn test_sign_valid_data() {
        let init_data = "query_id=AAHdF6IQAAAAAN0XohDhrOrc\
            &user=%7B%22id%22%3A279058397%2C%22first_name%22%3A%22Vladislav%22%2C\
            %22last_name%22%3A%22Kibenko%22%2C%22username%22%3A%22vdkfrost%22%2C\
            %22language_code%22%3A%22ru%22%2C%22is_premium%22%3Atrue%7D\
            &auth_date=1662771648";

        let result = sign(init_data, BOT_TOKEN).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_with_existing_hash() {
        let init_data = "query_id=AAHdF6IQAAAAAN0XohDhrOrc\
            &user=%7B%22id%22%3A279058397%7D\
            &auth_date=1662771648\
            &hash=existing_hash";

        let result = sign(init_data, BOT_TOKEN).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_consistency() {
        let init_data = "auth_date=1662771648&query_id=test123";

        let hash1 = sign(init_data, BOT_TOKEN).unwrap();
        let hash2 = sign(init_data, BOT_TOKEN).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_sign_different_tokens() {
        let init_data = "auth_date=1662771648&query_id=test123";

        let hash1 = sign(init_data, "token1").unwrap();
        let hash2 = sign(init_data, "token2").unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_sign_parameter_order() {
        let init_data1 = "auth_date=1662771648&query_id=test123";
        let init_data2 = "query_id=test123&auth_date=1662771648";

        let hash1 = sign(init_data1, BOT_TOKEN).unwrap();
        let hash2 = sign(init_data2, BOT_TOKEN).unwrap();

        assert_eq!(hash1, hash2);
    }
}
