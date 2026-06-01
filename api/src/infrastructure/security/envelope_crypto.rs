use base64::{engine::general_purpose::STANDARD, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng, rand_core::RngCore},
    Key, XChaCha20Poly1305, XNonce,
};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const MSG_PREFIX: &str = "enc:v1:";
const BIN_PREFIX: &[u8] = b"encb:v1:";

#[derive(Serialize, Deserialize)]
struct Envelope {
    key_id: String,
    wrap_nonce_b64: String,
    wrapped_dek_b64: String,
    data_nonce_b64: String,
    ciphertext_b64: String,
}

#[derive(Clone)]
pub struct EnvelopeCrypto {
    key_id: String,
    kek: Option<[u8; 32]>,
}

impl EnvelopeCrypto {
    pub fn new(key_id: String, master_key_b64: Option<String>) -> AppResult<Self> {
        let kek = match master_key_b64 {
            Some(k) => {
                let bytes = STANDARD
                    .decode(k)
                    .map_err(|e| AppError::Validation(format!("invalid CHAT_MASTER_KEY_B64: {e}")))?;
                if bytes.len() != 32 {
                    return Err(AppError::Validation("CHAT_MASTER_KEY_B64 must decode to 32 bytes".into()));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Some(arr)
            }
            None => None,
        };
        Ok(Self { key_id, kek })
    }

    pub fn encrypt_text(&self, plaintext: &str) -> AppResult<String> {
        let Some(kek) = self.kek else {
            return Ok(plaintext.to_string());
        };
        let env = self.encrypt_with_kek(&kek, plaintext.as_bytes())?;
        Ok(format!("{MSG_PREFIX}{}", serde_json::to_string(&env).map_err(|e| AppError::Internal(e.to_string()))?))
    }

    pub fn decrypt_text(&self, value: &str) -> AppResult<String> {
        if !value.starts_with(MSG_PREFIX) {
            return Ok(value.to_string());
        }
        let Some(kek) = self.kek else {
            return Err(AppError::Validation("encrypted payload encountered without master key".into()));
        };
        let env: Envelope = serde_json::from_str(&value[MSG_PREFIX.len()..])
            .map_err(|e| AppError::Validation(format!("invalid encrypted message payload: {e}")))?;
        let plain = self.decrypt_with_kek(&kek, &env)?;
        String::from_utf8(plain).map_err(|e| AppError::Internal(e.to_string()))
    }

    pub fn encrypt_bytes(&self, plaintext: &[u8]) -> AppResult<Vec<u8>> {
        let Some(kek) = self.kek else {
            return Ok(plaintext.to_vec());
        };
        let env = self.encrypt_with_kek(&kek, plaintext)?;
        let mut out = BIN_PREFIX.to_vec();
        out.extend(serde_json::to_vec(&env).map_err(|e| AppError::Internal(e.to_string()))?);
        Ok(out)
    }

    pub fn decrypt_bytes(&self, value: &[u8]) -> AppResult<Vec<u8>> {
        if !value.starts_with(BIN_PREFIX) {
            return Ok(value.to_vec());
        }
        let Some(kek) = self.kek else {
            return Err(AppError::Validation("encrypted attachment encountered without master key".into()));
        };
        let env: Envelope = serde_json::from_slice(&value[BIN_PREFIX.len()..])
            .map_err(|e| AppError::Validation(format!("invalid encrypted attachment payload: {e}")))?;
        self.decrypt_with_kek(&kek, &env)
    }

    pub fn is_encryption_enabled(&self) -> bool {
        self.kek.is_some()
    }

    fn encrypt_with_kek(&self, kek: &[u8; 32], plaintext: &[u8]) -> AppResult<Envelope> {
        let mut dek = [0u8; 32];
        OsRng.fill_bytes(&mut dek);

        let mut wrap_nonce = [0u8; 24];
        OsRng.fill_bytes(&mut wrap_nonce);
        let mut data_nonce = [0u8; 24];
        OsRng.fill_bytes(&mut data_nonce);

        let wrap_cipher = XChaCha20Poly1305::new(Key::from_slice(kek));
        let wrapped_dek = wrap_cipher
            .encrypt(XNonce::from_slice(&wrap_nonce), dek.as_slice())
            .map_err(|_| AppError::Internal("failed to wrap DEK".into()))?;

        let data_cipher = XChaCha20Poly1305::new(Key::from_slice(&dek));
        let ciphertext = data_cipher
            .encrypt(XNonce::from_slice(&data_nonce), plaintext)
            .map_err(|_| AppError::Internal("failed to encrypt payload".into()))?;

        Ok(Envelope {
            key_id: self.key_id.clone(),
            wrap_nonce_b64: STANDARD.encode(wrap_nonce),
            wrapped_dek_b64: STANDARD.encode(wrapped_dek),
            data_nonce_b64: STANDARD.encode(data_nonce),
            ciphertext_b64: STANDARD.encode(ciphertext),
        })
    }

    fn decrypt_with_kek(&self, kek: &[u8; 32], env: &Envelope) -> AppResult<Vec<u8>> {
        let wrap_nonce = STANDARD
            .decode(&env.wrap_nonce_b64)
            .map_err(|e| AppError::Validation(format!("invalid wrap nonce: {e}")))?;
        let wrapped_dek = STANDARD
            .decode(&env.wrapped_dek_b64)
            .map_err(|e| AppError::Validation(format!("invalid wrapped dek: {e}")))?;
        let data_nonce = STANDARD
            .decode(&env.data_nonce_b64)
            .map_err(|e| AppError::Validation(format!("invalid data nonce: {e}")))?;
        let ciphertext = STANDARD
            .decode(&env.ciphertext_b64)
            .map_err(|e| AppError::Validation(format!("invalid ciphertext: {e}")))?;

        let wrap_cipher = XChaCha20Poly1305::new(Key::from_slice(kek));
        let dek = wrap_cipher
            .decrypt(XNonce::from_slice(&wrap_nonce), wrapped_dek.as_slice())
            .map_err(|_| AppError::Validation("invalid wrapped DEK".into()))?;

        let data_cipher = XChaCha20Poly1305::new(Key::from_slice(&dek));
        data_cipher
            .decrypt(XNonce::from_slice(&data_nonce), ciphertext.as_slice())
            .map_err(|_| AppError::Validation("invalid ciphertext".into()))
    }
}
