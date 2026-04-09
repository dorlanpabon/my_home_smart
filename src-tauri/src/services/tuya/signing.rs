use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::errors::{AppError, AppResult};

type HmacSha256 = Hmac<Sha256>;

pub fn hash_body(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn string_to_sign(method: &str, body: &str, canonical_url: &str) -> String {
    format!(
        "{}\n{}\n\n{}",
        method.to_uppercase(),
        hash_body(body),
        canonical_url
    )
}

pub fn sign(
    client_id: &str,
    client_secret: &str,
    access_token: Option<&str>,
    timestamp: &str,
    nonce: &str,
    string_to_sign: &str,
) -> AppResult<String> {
    let mut mac = HmacSha256::new_from_slice(client_secret.as_bytes())
        .map_err(|err| AppError::UnexpectedResponse(err.to_string()))?;

    let payload = format!(
        "{}{}{}{}{}",
        client_id,
        access_token.unwrap_or_default(),
        timestamp,
        nonce,
        string_to_sign
    );

    mac.update(payload.as_bytes());
    Ok(hex::encode_upper(mac.finalize().into_bytes()))
}

#[cfg(test)]
mod tests {
    use super::{hash_body, sign, string_to_sign};

    #[test]
    fn hashes_empty_body() {
        assert_eq!(
            hash_body(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn creates_signature() {
        let string_to_sign = string_to_sign("GET", "", "/v1.0/token?grant_type=1");
        let signature = sign(
            "client",
            "secret",
            None,
            "1711111111111",
            "nonce",
            &string_to_sign,
        )
        .unwrap();

        assert_eq!(signature.len(), 64);
    }
}
