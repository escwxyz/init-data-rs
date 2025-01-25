use serde_json::Value;
use std::collections::BTreeMap;
use url::form_urlencoded;

use crate::error::InitDataError;
use crate::model::InitData;

const STRING_PROPS: [&str; 1] = ["start_param"];

/// Parse converts passed init data presented as query string to InitData object.
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
                format!("\"{}\":{}", k, v)
            } else {
                format!("\"{}\":\"{}\"", k, v.replace('\"', "\\\""))
            }
        })
        .collect();

    let json_str = format!("{{{}}}", json_pairs.join(","));

    serde_json::from_str(&json_str).map_err(|e| InitDataError::UnexpectedFormat(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ChatType;

    const PARSE_TEST_INIT_DATA: &str = "query_id=AAHdF6IQAAAAAN0XohDhrOrc&user=%7B%22id%22%3A279058397%2C%22first_name%22%3A%22Vladislav%22%2C%22last_name%22%3A%22Kibenko%22%2C%22username%22%3A%22vdkfrost%22%2C%22language_code%22%3A%22ru%22%2C%22is_premium%22%3Atrue%7D&auth_date=1662771648&hash=c501b71e775f74ce10e377dea85a7ea24ecd640b223ea86dfe453e0eaed2e2b2&start_param=abc";

    #[test]
    fn test_parse_invalid_format() {
        let result = parse(&format!("{};", PARSE_TEST_INIT_DATA));
        assert!(matches!(result, Err(InitDataError::UnexpectedFormat(_))));

        assert!(matches!(parse("invalid"), Err(InitDataError::UnexpectedFormat(_))));
        assert!(matches!(parse("a;b;c"), Err(InitDataError::UnexpectedFormat(_))));
    }

    #[test]
    fn test_parse_valid_data() {
        let result = parse(PARSE_TEST_INIT_DATA).unwrap();

        assert_eq!(result.query_id, Some("AAHdF6IQAAAAAN0XohDhrOrc".to_string()));
        assert_eq!(result.auth_date, 1662771648);
        assert_eq!(result.start_param, Some("abc".to_string()));
        assert_eq!(
            result.hash,
            "c501b71e775f74ce10e377dea85a7ea24ecd640b223ea86dfe453e0eaed2e2b2"
        );

        if let Some(user) = result.user {
            assert_eq!(user.id, 279058397);
            assert_eq!(user.first_name, "Vladislav");
            assert_eq!(user.last_name, Some("Kibenko".to_string()));
            assert_eq!(user.username, Some("vdkfrost".to_string()));
            assert_eq!(user.language_code, Some("ru".to_string()));
            assert_eq!(user.is_premium, Some(true));
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
        let init_data = "chat=%7B%22id%22%3A-100123456789%2C%22type%22%3A%22supergroup%22%2C%22title%22%3A%22Test%20Group%22%7D&auth_date=1662771648&hash=abc";
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
        let init_data = "start_param=test123&auth_date=1662771648&hash=abc";
        let result = parse(init_data).unwrap();
        assert_eq!(result.start_param, Some("test123".to_string()));
    }
}
