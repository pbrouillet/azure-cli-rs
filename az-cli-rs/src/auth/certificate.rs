/// Service principal certificate authentication — client assertion JWT flow.
///
/// Loads a PEM certificate, computes SHA-1 thumbprint, builds a signed JWT
/// client assertion, and exchanges it for an access token via OAuth2 client_credentials.
use crate::auth::oauth2::TokenResponse;
use crate::error::{AzrsError, Result};

use base64::Engine;
use rsa::pkcs8::DecodePrivateKey;
use rsa::signature::SignatureEncoding;
use sha1::Digest as Sha1Digest;
use sha2::Digest as Sha2Digest;

/// Perform service principal login with a PEM certificate.
///
/// Steps:
/// 1. Load PEM file → extract RSA private key + X.509 certificate
/// 2. Compute SHA-1 thumbprint of the certificate (DER) → base64url → x5t claim
/// 3. Build JWT header + payload (client_id, tenant, audience, exp)
/// 4. Sign JWT with RS256
/// 5. POST to token endpoint with client_assertion grant
pub async fn login_with_certificate(
    authority: &str,
    client_id: &str,
    certificate_path: &str,
    certificate_password: Option<&str>,
    scopes: &[String],
) -> Result<TokenResponse> {
    if certificate_password.is_some() {
        return Err(AzrsError::Auth(
            "PFX/PKCS#12 certificates with passwords are not yet supported. Use a PEM file without a password.".into()
        ));
    }

    let pem_data = std::fs::read_to_string(certificate_path).map_err(|e| {
        AzrsError::Auth(format!("Failed to read certificate file '{certificate_path}': {e}"))
    })?;

    let (private_key, cert_der) = parse_pem(&pem_data)?;
    let thumbprint = compute_thumbprint(&cert_der);
    let token_url = format!("{authority}/oauth2/v2.0/token");
    let assertion = build_client_assertion(client_id, &token_url, &thumbprint, &private_key)?;

    let scope_str = scopes.join(" ");

    let client = reqwest::Client::new();
    let resp = client
        .post(&token_url)
        .form(&[
            ("client_id", client_id),
            ("client_assertion", &assertion),
            (
                "client_assertion_type",
                "urn:ietf:params:oauth:client-assertion-type:jwt-bearer",
            ),
            ("grant_type", "client_credentials"),
            ("scope", &scope_str),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AzrsError::Auth(format!(
            "Certificate-based SP login failed: {body}"
        )));
    }

    let token: TokenResponse = resp.json().await?;
    Ok(token.finalize())
}

/// Parse a PEM file to extract the RSA private key and the first X.509 certificate (DER bytes).
fn parse_pem(pem_data: &str) -> Result<(rsa::RsaPrivateKey, Vec<u8>)> {
    // Extract private key
    let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(pem_data).map_err(|e| {
        AzrsError::Auth(format!(
            "Failed to parse PKCS#8 private key from PEM: {e}. \
             Ensure the PEM contains a PKCS#8 private key (BEGIN PRIVATE KEY)."
        ))
    })?;

    // Extract certificate DER
    let cert_der = extract_cert_der(pem_data)?;

    Ok((private_key, cert_der))
}

/// Extract the first X.509 certificate DER bytes from PEM data.
fn extract_cert_der(pem_data: &str) -> Result<Vec<u8>> {
    let begin = "-----BEGIN CERTIFICATE-----";
    let end = "-----END CERTIFICATE-----";

    let start = pem_data.find(begin).ok_or_else(|| {
        AzrsError::Auth(
            "No CERTIFICATE block found in PEM file. Ensure the file contains both a private key and certificate.".into()
        )
    })? + begin.len();

    let finish = pem_data[start..].find(end).ok_or_else(|| {
        AzrsError::Auth("Malformed CERTIFICATE block in PEM file".into())
    })? + start;

    let b64: String = pem_data[start..finish]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    base64::engine::general_purpose::STANDARD
        .decode(&b64)
        .map_err(|e| AzrsError::Auth(format!("Failed to decode certificate base64: {e}")))
}

/// Compute SHA-1 thumbprint of the DER-encoded certificate, returned as base64url.
fn compute_thumbprint(cert_der: &[u8]) -> String {
    let mut hasher = sha1::Sha1::new();
    hasher.update(cert_der);
    let hash = hasher.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

/// Build a signed JWT client assertion for the token endpoint.
fn build_client_assertion(
    client_id: &str,
    audience: &str,
    x5t: &str,
    private_key: &rsa::RsaPrivateKey,
) -> Result<String> {
    let now = chrono::Utc::now().timestamp();
    let exp = now + 600; // 10 minutes
    let jti = uuid::Uuid::new_v4().to_string();

    // Header
    let header = serde_json::json!({
        "alg": "RS256",
        "typ": "JWT",
        "x5t": x5t,
    });
    let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&header).unwrap());

    // Payload
    let payload = serde_json::json!({
        "iss": client_id,
        "sub": client_id,
        "aud": audience,
        "exp": exp,
        "nbf": now,
        "iat": now,
        "jti": jti,
    });
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&payload).unwrap());

    // Signing input
    let signing_input = format!("{header_b64}.{payload_b64}");

    // Sign with RS256
    use rsa::pkcs1v15::SigningKey;
    use rsa::signature::Signer;
    let signing_key = SigningKey::<sha2::Sha256>::new(private_key.clone());
    let signature = signing_key.sign(signing_input.as_bytes());
    let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(signature.to_bytes());

    Ok(format!("{signing_input}.{sig_b64}"))
}
