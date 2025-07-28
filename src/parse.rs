use serde_json::Value;
use std::collections::BTreeMap;
use url::form_urlencoded;

use crate::error::InitDataError;
use crate::model::InitData;

const STRING_PROPS: [&str; 1] = ["start_param"];

/// Parse converts passed init data presented as query string to `InitData` object.
///
/// # Errors
///
/// This function returns an `Err` in one of the following cases:
///
/// - `auth_date` is missing
/// - hash is missing
/// - hash is invalid
/// - init data has unexpected format
/// - init data is expired
/// - signature is missing
/// - signature is invalid
/// - the library has an internal error while hmac-ing the string. this should never happen
pub fn parse(init_data: &str) -> Result<InitData, InitDataError> {
    if init_data.is_empty() {
        return Err(InitDataError::UnexpectedFormat("init_data is empty".to_string()));
    }

    if init_data.contains(';') || !init_data.contains('=') {
        return Err(InitDataError::UnexpectedFormat(
            "Invalid query string format".to_string(),
        ));
    }

    let pairs = form_urlencoded::parse(init_data.as_bytes());
    let mut params: BTreeMap<String, String> = BTreeMap::new();

    for (key, value) in pairs {
        params.insert(key.to_string(), value.into_owned());
    }

    let json_pairs: Vec<String> = params
        .iter()
        .map(|(k, v)| {
            if !STRING_PROPS.contains(&k.as_str()) && serde_json::from_str::<Value>(v).is_ok() {
                format!("\"{k}\":{v}")
            } else {
                format!("\"{k}\":\"{}\"", v.replace('\"', "\\\""))
            }
        })
        .collect();

    let json_str = format!("{{{}}}", json_pairs.join(","));

    let result =
        serde_json::from_str::<InitData>(&json_str).map_err(|err| InitDataError::UnexpectedFormat(err.to_string()))?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ChatType;

    const PARSE_TEST_INIT_DATA: &str = "user=%7B%22id%22%3A6601562775%2C%22first_name%22%3A%22%29%22%2C%22last_name%22%3A%22%22%2C%22username%22%3A%22trogloditik%22%2C%22language_code%22%3A%22en%22%2C%22allows_write_to_pm%22%3Atrue%2C%22photo_url%22%3A%22https%3A%5C%2F%5C%2Ft.me%5C%2Fi%5C%2Fuserpic%5C%2F320%5C%2FqABgrvbhV8g_iUjd_pSUuX1bBuXefFmspMjb57gedoGAKDPx5fxwEMIF8k62mWhS.svg%22%7D&chat_instance=-8599080687359297588&chat_type=sender&auth_date=1748683232&signature=5rhZg9sshLtKrdTSwGvXA60MRmqtfU0RPTmUIAdcOEAm2n1XRfQhf0hvQNZo9Nwx4G3Kk92RSelu_CrPzra7Aw&hash=c8fdc0e1608154171a77ef4ce838d114b0229d891ee55ac1ee566f14551433e8";

    #[test]
    fn test_parse_invalid_format() {
        let result = parse(&format!("{PARSE_TEST_INIT_DATA};"));
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));

        assert!(matches!(parse("invalid"), Err(InitDataError::UnexpectedFormat(_))));
        assert!(matches!(parse("a;b;c"), Err(InitDataError::UnexpectedFormat(_))));
    }

    #[test]
    fn test_parse_valid_data() {
        let result = parse(PARSE_TEST_INIT_DATA).unwrap();

        assert_eq!(result.query_id, None);
        assert_eq!(result.auth_date, 1748683232);
        assert_eq!(result.start_param, None);
        assert_eq!(
            result.hash,
            "c8fdc0e1608154171a77ef4ce838d114b0229d891ee55ac1ee566f14551433e8"
        );

        if let Some(user) = result.user {
            assert_eq!(user.id, 6601562775);
            assert_eq!(user.first_name, ")");
            assert_eq!(user.last_name, Some(String::new()));
            assert_eq!(user.username, Some("trogloditik".to_string()));
            assert_eq!(user.language_code, Some("en".to_string()));
            assert_eq!(user.is_premium, None);
        } else {
            panic!("User should be present");
        }
    }

    #[test]
    fn test_parse_empty_data() {
        let result = parse("");
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));
    }

    #[test]
    fn test_parse_with_chat() {
        let init_data = "chat=%7B%22id%22%3A-100123456789%2C%22type%22%3A%22supergroup%22%2C%22title%22%3A%22Test%20Group%22%7D&auth_date=1662771648&signature=abc&hash=abc";
        let result = parse(init_data).unwrap();

        if let Some(chat) = result.chat {
            assert_eq!(chat.id, -100123456789);
            assert!(matches!(chat.chat_type, ChatType::Supergroup));
            assert_eq!(chat.title, "Test Group");
        } else {
            panic!("Chat should be present");
        }
    }

    #[test]
    fn test_parse_start_param() {
        let init_data = "start_param=test123&auth_date=1662771648&signature=abc&hash=abc";
        let result = parse(init_data).unwrap();
        assert_eq!(result.start_param, Some("test123".to_string()));
    }
}
