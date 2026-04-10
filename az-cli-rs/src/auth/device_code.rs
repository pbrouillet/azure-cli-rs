/// OAuth2 device code flow for environments without a browser.
///
/// 1. Request a device code from the devicecode endpoint
/// 2. Display the user_code and verification_uri to the user
/// 3. Poll the token endpoint until the user completes auth
use crate::auth::oauth2::{TokenResponse, AZURE_CLI_CLIENT_ID};
use crate::error::{AzrsError, Result};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
#[allow(dead_code)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default = "default_interval")]
    interval: u64,
    expires_in: u64,
    message: String,
}

fn default_interval() -> u64 {
    5
}

#[derive(Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum PollResponse {
    Success(TokenResponse),
    Error(PollError),
}

#[derive(Deserialize)]
struct PollError {
    error: String,
    error_description: Option<String>,
}

/// Perform device code login and return a finalized TokenResponse.
pub async fn login(authority: &str, scopes: &[String]) -> Result<TokenResponse> {
    let scope_str = format!("{} offline_access openid profile", scopes.join(" "));
    let client = reqwest::Client::new();

    // Step 1: Request device code
    let dc_url = format!("{authority}/oauth2/v2.0/devicecode");
    let resp = client
        .post(&dc_url)
        .form(&[
            ("client_id", AZURE_CLI_CLIENT_ID),
            ("scope", scope_str.as_str()),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AzrsError::Auth(format!(
            "Device code request failed: {body}"
        )));
    }

    let dc: DeviceCodeResponse = resp.json().await?;

    // Step 2: Display instructions
    eprintln!("{}", dc.message);

    // Step 3: Poll for completion
    let token_url = format!("{authority}/oauth2/v2.0/token");
    let interval = Duration::from_secs(dc.interval);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(dc.expires_in);

    loop {
        tokio::time::sleep(interval).await;

        if tokio::time::Instant::now() > deadline {
            return Err(AzrsError::Auth(
                "Device code flow timed out. Please try again.".into(),
            ));
        }

        let resp = client
            .post(&token_url)
            .form(&[
                ("client_id", AZURE_CLI_CLIENT_ID),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", &dc.device_code),
            ])
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        if status.is_success() {
            let token: TokenResponse = serde_json::from_str(&body)?;
            return Ok(token.finalize());
        }

        // Parse the error
        let poll_err: std::result::Result<PollError, _> = serde_json::from_str(&body);
        match poll_err {
            Ok(err) => match err.error.as_str() {
                "authorization_pending" => continue,
                "slow_down" => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                "authorization_declined" => {
                    return Err(AzrsError::Auth("Authorization was declined by the user.".into()));
                }
                "expired_token" => {
                    return Err(AzrsError::Auth("Device code expired. Please try again.".into()));
                }
                other => {
                    let desc = err.error_description.unwrap_or_default();
                    return Err(AzrsError::Auth(format!("Auth error: {other} — {desc}")));
                }
            },
            Err(_) => {
                return Err(AzrsError::Auth(format!(
                    "Unexpected response from token endpoint: {body}"
                )));
            }
        }
    }
}
