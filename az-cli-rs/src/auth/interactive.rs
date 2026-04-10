/// Interactive browser-based OAuth2 authorization code flow with PKCE.
///
/// 1. Start a local HTTP server on a random port
/// 2. Open the browser to the authorize endpoint
/// 3. Receive the auth code via redirect
/// 4. Exchange the code for tokens
use crate::auth::oauth2::{self, TokenResponse, AZURE_CLI_CLIENT_ID};
use crate::error::{AzrsError, Result};
use url::Url;

/// Perform interactive login and return a finalized TokenResponse.
pub async fn login(authority: &str, scopes: &[String]) -> Result<TokenResponse> {
    let (code_verifier, code_challenge) = oauth2::generate_pkce();
    let scope_str = scopes.join(" ");

    // Start local HTTP server on a random port
    let server =
        tiny_http::Server::http("127.0.0.1:0").map_err(|e| AzrsError::Auth(e.to_string()))?;
    let port = server.server_addr().to_ip().unwrap().port();
    let redirect_uri = format!("http://localhost:{port}");

    // Build authorization URL
    let authorize_url = format!(
        "{authority}/oauth2/v2.0/authorize?\
         client_id={AZURE_CLI_CLIENT_ID}\
         &response_type=code\
         &redirect_uri={redirect_uri}\
         &scope={scope_str}+offline_access+openid+profile\
         &code_challenge={code_challenge}\
         &code_challenge_method=S256\
         &prompt=select_account"
    );

    eprintln!("Opening browser for login...");
    eprintln!("If the browser doesn't open, visit:");
    eprintln!("  {authorize_url}");

    if open::that(&authorize_url).is_err() {
        eprintln!("Failed to open browser automatically.");
    }

    // Handle exactly one request
    let request = server
        .recv()
        .map_err(|e| AzrsError::Auth(format!("Failed to receive redirect: {e}")))?;

    let request_url = format!("http://localhost{}", request.url());
    let parsed = Url::parse(&request_url).map_err(|e| AzrsError::Auth(e.to_string()))?;

    // Extract the authorization code from query parameters
    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string());

    let error = parsed
        .query_pairs()
        .find(|(k, _)| k == "error")
        .map(|(_, v)| v.to_string());

    let error_description = parsed
        .query_pairs()
        .find(|(k, _)| k == "error_description")
        .map(|(_, v)| v.to_string());

    // Send a response to the browser
    let response_html = if code.is_some() {
        "<html><body><h2>Login successful!</h2><p>You can close this window.</p></body></html>"
    } else {
        "<html><body><h2>Login failed.</h2><p>Check the terminal for details.</p></body></html>"
    };
    let response = tiny_http::Response::from_string(response_html)
        .with_header("Content-Type: text/html".parse::<tiny_http::Header>().unwrap());
    let _ = request.respond(response);

    if let Some(err) = error {
        let desc = error_description.unwrap_or_default();
        return Err(AzrsError::Auth(format!("Authorization failed: {err} — {desc}")));
    }

    let code = code.ok_or_else(|| AzrsError::Auth("No authorization code received".into()))?;

    // Exchange authorization code for tokens
    let token_url = format!("{authority}/oauth2/v2.0/token");
    let client = reqwest::Client::new();
    let resp = client
        .post(&token_url)
        .form(&[
            ("client_id", AZURE_CLI_CLIENT_ID),
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
            ("code_verifier", &code_verifier),
            (
                "scope",
                &format!("{scope_str} offline_access openid profile"),
            ),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AzrsError::Auth(format!("Token exchange failed: {body}")));
    }

    let token: TokenResponse = resp.json().await?;
    Ok(token.finalize())
}
