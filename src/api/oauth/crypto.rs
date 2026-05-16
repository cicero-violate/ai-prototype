//! OAuth store encoding and PKCE helpers.

use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use sha2::{Digest, Sha256};

pub(crate) fn key_material(key: &str) -> Vec<u8> {
    Sha256::digest(key.as_bytes()).to_vec()
}

pub(crate) fn encrypt_blob(plaintext: &[u8], key_material: &[u8]) -> String {
    STANDARD.encode(xor_keystream(plaintext, key_material))
}

pub(crate) fn decrypt_blob(blob: &str, key_material: &[u8]) -> Result<Vec<u8>, String> {
    let ciphertext = STANDARD
        .decode(blob)
        .map_err(|e| format!("decode oauth store: {e}"))?;
    Ok(xor_keystream(&ciphertext, key_material))
}

pub(crate) fn verify_pkce(verifier: &str, challenge: &str) -> bool {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())) == challenge
}

fn xor_keystream(input: &[u8], key_material: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut counter = 0_u64;
    while out.len() < input.len() {
        let mut hasher = Sha256::new();
        hasher.update(key_material);
        hasher.update(counter.to_le_bytes());
        let block = hasher.finalize();
        for byte in block {
            if out.len() == input.len() {
                break;
            }
            out.push(input[out.len()] ^ byte);
        }
        counter = counter.wrapping_add(1);
    }
    out
}
