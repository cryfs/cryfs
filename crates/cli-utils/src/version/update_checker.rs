use anyhow::{Result, anyhow};
use cryfs_version::Version;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use super::http_client::HttpClient;

const UPDATE_INFO_URL: &str = "https://www.cryfs.org/version_info.json";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

pub fn check_for_updates<'a>(
    http_client: impl HttpClient,
    current_version: Version<'a>,
) -> Result<UpdateCheckResult> {
    // TODO can we make this async?
    let response_json = http_client.get(UPDATE_INFO_URL, REQUEST_TIMEOUT)?;
    let response: VersionResponse = serde_json::from_str(&response_json)?;

    let security_warning = parse_warning(&response.warnings_for_version, current_version);
    let newest_version =
        Version::parse(&response.version_info.current).map_err(|err| anyhow!("{err}"))?;
    let released_newer_version = if current_version < newest_version {
        Some(response.version_info.current.clone())
    } else {
        None
    };

    Ok(UpdateCheckResult {
        released_newer_version,
        security_warning,
    })
}

#[derive(Debug)]
pub struct UpdateCheckResult {
    pub released_newer_version: Option<String>,
    pub security_warning: Option<String>,
}

#[derive(Deserialize)]
struct VersionResponse {
    version_info: VersionResponseVersionInfo,
    #[serde(rename = "warnings")]
    warnings_for_version: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct VersionResponseVersionInfo {
    current: String,
}

fn parse_warning<'a>(
    warnings: &Option<HashMap<String, String>>,
    version: Version<'a>,
) -> Option<String> {
    let warnings = warnings.as_ref()?;

    let running_version = version.to_string();
    warnings
        .iter()
        .filter_map(|(warning_version, warning)| {
            if running_version == *warning_version {
                Some(warning.clone())
            } else {
                None
            }
        })
        .next()
}

#[cfg(test)]
mod tests {
    use super::super::http_client::FakeHttpClient;
    use super::*;

    const OLDER_VERSION: Version<'static> = konst::unwrap_ctx!(Version::parse_const("0.11.0"));
    const CURRENT_VERSION: Version<'static> = konst::unwrap_ctx!(Version::parse_const("1.0.0"));
    const NEWER_VERSION: Version<'static> = konst::unwrap_ctx!(Version::parse_const("1.0.1"));

    #[test]
    fn http_error() {
        let client = FakeHttpClient::new();
        assert_eq!(
            format!("URL not found: {UPDATE_INFO_URL}"),
            check_for_updates(client, CURRENT_VERSION)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn invalid_json() {
        let mut client = FakeHttpClient::new();
        client.add_website(UPDATE_INFO_URL.to_string(), "invalid json".to_string());
        assert_eq!(
            "expected value at line 1 column 1",
            check_for_updates(client, CURRENT_VERSION)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn no_version_no_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(UPDATE_INFO_URL.to_string(), "{}".to_string());
        assert_eq!(
            "missing field `version_info` at line 1 column 2",
            check_for_updates(client, CURRENT_VERSION)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn no_version_empty_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            r#"{"warnings": {}}"#.to_string(),
        );
        assert_eq!(
            "missing field `version_info` at line 1 column 16",
            check_for_updates(client, CURRENT_VERSION)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn no_version_some_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(UPDATE_INFO_URL.to_string(), format!(r#"{{"warnings": {{"{OLDER_VERSION}": "other warning", "{CURRENT_VERSION}": "my warning text", "{NEWER_VERSION}": "yet another warning"}}}}"#));
        assert_eq!(
            "missing field `version_info` at line 1 column 101",
            check_for_updates(client, CURRENT_VERSION)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn invalid_version_no_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            r#"{"version_info": {"current": "invalid version"}}"#.to_string(),
        );
        assert_eq!(
            "Failed to parse version `invalid version`: invalid digit found in string",
            check_for_updates(client, CURRENT_VERSION)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn invalid_version_empty_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            r#"{"version_info": {"current": "invalid version"}, "warnings": {}}"#.to_string(),
        );
        assert_eq!(
            "Failed to parse version `invalid version`: invalid digit found in string",
            check_for_updates(client, CURRENT_VERSION)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn invalid_version_some_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(r#"{{"version_info": {{"current": "invalid version"}}, "warnings": {{"{OLDER_VERSION}": "other warning", "{CURRENT_VERSION}": "my warning text", "{NEWER_VERSION}": "yet another warning"}}}}"#),
        );
        assert_eq!(
            "Failed to parse version `invalid version`: invalid digit found in string",
            check_for_updates(client, CURRENT_VERSION)
                .unwrap_err()
                .to_string()
        );
    }

    #[test]
    fn older_version_no_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(r#"{{"version_info": {{"current": "{OLDER_VERSION}"}}}}"#),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(None, result.released_newer_version);
        assert_eq!(None, result.security_warning);
    }

    #[test]
    fn newer_version_no_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(r#"{{"version_info": {{"current": "{NEWER_VERSION}"}}}}"#),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(
            Some(NEWER_VERSION.to_string()),
            result.released_newer_version
        );
        assert_eq!(None, result.security_warning);
    }

    #[test]
    fn older_version_empty_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(r#"{{"version_info": {{"current": "{OLDER_VERSION}"}}, "warnings": {{}}}}"#),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(None, result.released_newer_version);
        assert_eq!(None, result.security_warning);
    }

    #[test]
    fn newer_version_empty_warnings() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(r#"{{"version_info": {{"current": "{NEWER_VERSION}"}}, "warnings": {{}}}}"#),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(
            Some(NEWER_VERSION.to_string()),
            result.released_newer_version
        );
        assert_eq!(None, result.security_warning);
    }

    #[test]
    fn older_version_with_warning_for_current_version() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(
                r#"{{"version_info": {{"current": "{OLDER_VERSION}"}}, "warnings": {{"{CURRENT_VERSION}": "my warning text"}}}}"#
            ),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(None, result.released_newer_version);
        assert_eq!(Some("my warning text".to_string()), result.security_warning);
    }

    #[test]
    fn newer_version_with_warning_for_current_version() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(
                r#"{{"version_info": {{"current": "{NEWER_VERSION}"}}, "warnings": {{"{CURRENT_VERSION}": "my warning text"}}}}"#
            ),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(
            Some(NEWER_VERSION.to_string()),
            result.released_newer_version
        );
        assert_eq!(Some("my warning text".to_string()), result.security_warning);
    }

    #[test]
    fn older_version_with_warning_for_current_and_other_versions() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(
                r#"{{"version_info": {{"current": "{OLDER_VERSION}"}}, "warnings": {{"{OLDER_VERSION}": "other warning", "{CURRENT_VERSION}": "my warning text", "{NEWER_VERSION}": "yet another warning"}}}}"#
            ),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(None, result.released_newer_version);
        assert_eq!(Some("my warning text".to_string()), result.security_warning);
    }

    #[test]
    fn newer_version_with_warning_for_current_and_other_versions() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(
                r#"{{"version_info": {{"current": "{NEWER_VERSION}"}}, "warnings": {{"{OLDER_VERSION}": "other warning", "{CURRENT_VERSION}": "my warning text", "{NEWER_VERSION}": "yet another warning"}}}}"#
            ),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(
            Some(NEWER_VERSION.to_string()),
            result.released_newer_version
        );
        assert_eq!(Some("my warning text".to_string()), result.security_warning);
    }

    #[test]
    fn older_version_with_warning_for_other_versions() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(
                r#"{{"version_info": {{"current": "{OLDER_VERSION}"}}, "warnings": {{"{OLDER_VERSION}": "other warning", "{NEWER_VERSION}": "yet another warning"}}}}"#
            ),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(None, result.released_newer_version);
        assert_eq!(None, result.security_warning);
    }

    #[test]
    fn newer_version_with_warning_for_other_versions() {
        let mut client = FakeHttpClient::new();
        client.add_website(
            UPDATE_INFO_URL.to_string(),
            format!(
                r#"{{"version_info": {{"current": "{NEWER_VERSION}"}}, "warnings": {{"{OLDER_VERSION}": "other warning", "{NEWER_VERSION}": "yet another warning"}}}}"#
            ),
        );
        let result = check_for_updates(client, CURRENT_VERSION).unwrap();
        assert_eq!(
            Some(NEWER_VERSION.to_string()),
            result.released_newer_version
        );
        assert_eq!(None, result.security_warning);
    }
}
