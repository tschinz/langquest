//! Authenticated encryption for cheat-resistant on-disk data.
//!
//! Progress and attestation files are sealed with ChaCha20-Poly1305 (AEAD).
//! The Poly1305 authentication tag makes any tampering *detectable*: editing a
//! byte causes [`open`] to fail rather than silently returning altered data.
//!
//! Security note: the key is embedded in the binary, so this is
//! tamper-*evidence* against casual editing, not protection against an attacker
//! who reverse-engineers `lq`. See `identity` for the transfer-resistance layer.

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use thiserror::Error;

/// Magic marker prefixed to every sealed blob so a stray/foreign file is
/// rejected with a clear error instead of a confusing decrypt failure.
const MAGIC: &[u8; 4] = b"LQE1";
/// ChaCha20-Poly1305 nonce length in bytes.
const NONCE_LEN: usize = 12;
/// Length of the fixed header (`MAGIC` + nonce).
const HEADER_LEN: usize = 4 + NONCE_LEN;

/// Errors from sealing/opening encrypted blobs.
#[derive(Debug, Error)]
pub enum CryptoError {
  /// The blob is too short or does not start with the expected magic marker.
  #[error("not an lq encrypted file (bad magic or truncated)")]
  Format,

  /// Authentication failed: the blob was tampered with, truncated, or sealed
  /// with a different key.
  #[error("encrypted file failed integrity check (tampered or wrong key)")]
  Tamper,
}

/// Seal `plaintext` under `key`, returning `MAGIC || nonce || ciphertext+tag`.
///
/// A fresh random nonce is generated per call, so sealing identical plaintext
/// twice yields different blobs.
pub fn seal(key: &[u8; 32], plaintext: &[u8]) -> Vec<u8> {
  let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
  let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
  // Encryption over an in-memory buffer is infallible for ChaCha20-Poly1305.
  let ciphertext = cipher.encrypt(&nonce, plaintext).expect("aead encryption cannot fail in-memory");

  let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
  out.extend_from_slice(MAGIC);
  out.extend_from_slice(nonce.as_slice());
  out.extend_from_slice(&ciphertext);
  out
}

/// Open a blob produced by [`seal`], verifying integrity and returning the
/// plaintext.
///
/// Returns [`CryptoError::Format`] for a non-`lq` / truncated file and
/// [`CryptoError::Tamper`] when the authentication tag does not verify.
pub fn open(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
  if data.len() < HEADER_LEN || &data[..4] != MAGIC {
    return Err(CryptoError::Format);
  }
  let nonce = Nonce::from_slice(&data[4..HEADER_LEN]);
  let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
  cipher.decrypt(nonce, &data[HEADER_LEN..]).map_err(|_| CryptoError::Tamper)
}

#[cfg(test)]
mod tests {
  use super::*;

  const KEY: [u8; 32] = *b"unit-test-key-0123456789-abcdefg";

  #[test]
  fn roundtrip_recovers_plaintext() {
    let msg = b"current_exercise = 01-rust/01-hello";
    let sealed = seal(&KEY, msg);
    let opened = open(&KEY, &sealed).expect("open should succeed");
    assert_eq!(opened, msg);
  }

  #[test]
  fn ciphertext_is_not_plaintext() {
    let msg = b"best_score = 1.0";
    let sealed = seal(&KEY, msg);
    // The readable value must not appear verbatim in the sealed bytes.
    assert!(!sealed.windows(msg.len()).any(|w| w == msg));
  }

  #[test]
  fn nonce_is_randomised_per_seal() {
    let msg = b"same input";
    assert_ne!(seal(&KEY, msg), seal(&KEY, msg));
  }

  #[test]
  fn tampered_ciphertext_is_rejected() {
    let mut sealed = seal(&KEY, b"passed = false");
    // Flip a bit in the ciphertext body.
    let last = sealed.len() - 1;
    sealed[last] ^= 0x01;
    assert!(matches!(open(&KEY, &sealed), Err(CryptoError::Tamper)));
  }

  #[test]
  fn wrong_key_is_rejected() {
    let sealed = seal(&KEY, b"secret");
    let other: [u8; 32] = *b"a-different-key-9876543210-zyxwv";
    assert!(matches!(open(&other, &sealed), Err(CryptoError::Tamper)));
  }

  #[test]
  fn foreign_or_truncated_blob_is_rejected() {
    assert!(matches!(open(&KEY, b"not-lq"), Err(CryptoError::Format)));
    assert!(matches!(open(&KEY, &[]), Err(CryptoError::Format)));
  }
}
