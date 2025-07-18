use aes_gcm::{Aes256Gcm, Nonce}; // AES-GCM 256-bit
use aes_gcm::aead::{Aead, KeyInit};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Sha256, Digest};

pub fn encrypt_and_hash(content: &str, key_bytes: &[u8]) -> (Vec<u8>, String) {
    // 初始化加密器
    let cipher = Aes256Gcm::new_from_slice(key_bytes)
        .expect("Key 長度錯誤，需為 32 bytes");

    // 產生隨機 nonce（12 bytes）
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密
    let ciphertext = cipher.encrypt(nonce, content.as_bytes())
        .expect("加密失敗");

    // 對加密結果取 hash
    let mut hasher = Sha256::new();
    hasher.update(&ciphertext);
    let hash = format!("{:x}", hasher.finalize());

    (ciphertext, hash)
}