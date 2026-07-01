//! GitHub-identity binding and offline attestation for cheat-resistant progress.
//!
//! Progress is bound to the student's GitHub account so a solved file cannot be
//! shared with another student. Verification prefers a live check via the `gh`
//! CLI (`gh api user`); when offline it falls back to a locally cached,
//! machine-bound [`Attestation`] written on the last successful online check.
//!
//! The decision logic lives in the pure [`decide`] function so it can be
//! exhaustively unit-tested without touching the network, `gh`, or the machine.
//!
//! Ceiling: this runs on the student's machine, so patching the binary or
//! extracting the embedded key defeats it. It raises casual file-sharing from
//! "trivial" to "requires reverse engineering"; true anti-cheat needs a server.

use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Embedded key used to seal the local attestation cache.
const ATTEST_KEY: [u8; 32] = *b"lq-attestation-key-v1-keep-safe!";

/// Filename of the machine-bound attestation cache, alongside `lq.toml`.
pub const ATTEST_FILE: &str = ".lq.attest";

/// Maximum age of an offline attestation before a fresh online check is
/// required, in seconds (30 days).
pub const MAX_OFFLINE_AGE: u64 = 30 * 24 * 60 * 60;

/// A GitHub account identity.
///
/// `id` is the immutable numeric account id (never reused, cannot be changed by
/// the user); `login` is the display handle (can be renamed) and is stored for
/// human-readable messages only. Binding decisions key on `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubIdentity {
  /// Immutable numeric GitHub account id.
  pub id: u64,
  /// GitHub login/handle (display only).
  pub login: String,
}

/// Locally cached proof of a past successful online identity check, bound to
/// this machine. Sealed on disk so it cannot be edited or forged by hand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
  /// The numeric GitHub id that was verified online.
  pub github_id: u64,
  /// The login at the time of verification (display only).
  pub login: String,
  /// Fingerprint of the machine this attestation was created on.
  pub machine_fp: String,
  /// Unix timestamp (seconds) of the last successful online verification.
  pub verified_at: u64,
}

/// Why the current session is not allowed to use the stored progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenyReason {
  /// Live GitHub identity does not match the progress owner (a transferred file).
  IdentityMismatch {
    /// The owner recorded in the progress file.
    expected: u64,
    /// The live GitHub id of whoever is running `lq`.
    got: u64,
  },
  /// Offline and the attestation is for a different account than the owner.
  AttestationUserMismatch,
  /// Offline and the attestation was made on a different machine (transferred cache).
  AttestationMachineMismatch,
  /// Offline and the attestation is older than [`MAX_OFFLINE_AGE`].
  AttestationExpired,
  /// Offline with no usable attestation for existing progress.
  OfflineNoAttestation,
  /// Fresh progress but offline with no attestation to bootstrap from.
  BootstrapRequiresOnline,
}

impl std::fmt::Display for DenyReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DenyReason::IdentityMismatch { expected, got } => write!(
        f,
        "this progress belongs to GitHub account #{expected}, but you are signed in as #{got}. \
         Progress cannot be transferred between students."
      ),
      DenyReason::AttestationUserMismatch => write!(f, "offline: the cached identity does not match this progress file's owner."),
      DenyReason::AttestationMachineMismatch => write!(
        f,
        "offline: the cached identity was created on a different machine. \
         Connect to the internet to re-verify your GitHub identity."
      ),
      DenyReason::AttestationExpired => write!(f, "offline: your cached identity has expired. Connect to the internet to re-verify."),
      DenyReason::OfflineNoAttestation => write!(
        f,
        "offline and no cached identity found. Connect to the internet to verify your GitHub identity."
      ),
      DenyReason::BootstrapRequiresOnline => write!(
        f,
        "first launch requires an internet connection to bind your progress to your GitHub account. \
         Ensure `gh auth login` is set up and you are online."
      ),
    }
  }
}

/// Outcome of the pure authorization policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
  /// The session may proceed as `owner`. When `refresh_attestation` is true the
  /// caller should write a fresh attestation (an online check just succeeded).
  Allow {
    /// The identity the progress is (now) bound to.
    owner: GithubIdentity,
    /// Whether to rewrite the on-disk attestation.
    refresh_attestation: bool,
  },
  /// The session is rejected for the given reason.
  Deny(DenyReason),
}

/// Validate an attestation for the current machine and freshness window.
fn attest_ok<'a>(attest: Option<&'a Attestation>, machine_fp: &str, now: u64, max_age: u64) -> Result<&'a Attestation, DenyReason> {
  match attest {
    None => Err(DenyReason::OfflineNoAttestation),
    Some(a) if a.machine_fp != machine_fp => Err(DenyReason::AttestationMachineMismatch),
    Some(a) if now.saturating_sub(a.verified_at) > max_age => Err(DenyReason::AttestationExpired),
    Some(a) => Ok(a),
  }
}

/// Pure authorization policy — the core of the cheat-resistance system.
///
/// Given the progress owner (if any), the live GitHub identity (if online), a
/// cached attestation (if any), and the current machine fingerprint / time,
/// decide whether the session may proceed.
///
/// * Online: the live identity is authoritative. It must match the recorded
///   owner (or binds a fresh file); the attestation should be refreshed.
/// * Offline: fall back to the attestation, which must be for this machine, not
///   expired, and match the progress owner.
pub fn decide(
  stored_owner: Option<GithubIdentity>,
  live: Option<GithubIdentity>,
  attest: Option<&Attestation>,
  machine_fp: &str,
  now: u64,
  max_age: u64,
) -> Decision {
  match (stored_owner, live) {
    // --- Online: the live GitHub identity is authoritative. ---
    (Some(owner), Some(live)) => {
      if owner.id == live.id {
        Decision::Allow {
          owner: live,
          refresh_attestation: true,
        }
      } else {
        Decision::Deny(DenyReason::IdentityMismatch {
          expected: owner.id,
          got: live.id,
        })
      }
    }
    (None, Some(live)) => Decision::Allow {
      owner: live,
      refresh_attestation: true,
    },

    // --- Offline: fall back to the machine-bound attestation. ---
    (Some(owner), None) => match attest_ok(attest, machine_fp, now, max_age) {
      Ok(a) if a.github_id == owner.id => Decision::Allow {
        owner,
        refresh_attestation: false,
      },
      Ok(_) => Decision::Deny(DenyReason::AttestationUserMismatch),
      Err(reason) => Decision::Deny(reason),
    },
    (None, None) => match attest_ok(attest, machine_fp, now, max_age) {
      Ok(a) => Decision::Allow {
        owner: GithubIdentity {
          id: a.github_id,
          login: a.login.clone(),
        },
        refresh_attestation: false,
      },
      Err(_) => Decision::Deny(DenyReason::BootstrapRequiresOnline),
    },
  }
}

/// Current Unix time in seconds (saturating at 0 before the epoch).
pub fn unix_now() -> u64 {
  SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Compute a stable fingerprint for this machine.
///
/// Combines a hardware/OS machine id (immutable without admin rights) with the
/// current user and hostname, hashed to a hex string. Used to bind the offline
/// attestation so it cannot be copied to another machine.
pub fn machine_fingerprint() -> String {
  let mut hasher = Sha256::new();
  hasher.update(machine_id().as_bytes());
  hasher.update(b"\0");
  hasher.update(os_user().as_bytes());
  hasher.update(b"\0");
  hasher.update(hostname().as_bytes());
  let digest = hasher.finalize();
  hex(&digest)
}

/// Read a platform machine id. Falls back to an empty string if unavailable
/// (the user/hostname components still provide binding).
fn machine_id() -> String {
  // Linux / systemd.
  for p in ["/etc/machine-id", "/var/lib/dbus/machine-id"] {
    if let Ok(s) = std::fs::read_to_string(p) {
      let s = s.trim();
      if !s.is_empty() {
        return s.to_string();
      }
    }
  }
  // macOS: IOPlatformUUID via ioreg. Absolute path so the fingerprint does not
  // depend on `$PATH` (which would make the id unstable between runs).
  if let Ok(out) = Command::new("/usr/sbin/ioreg").args(["-rd1", "-c", "IOPlatformExpertDevice"]).output()
    && out.status.success()
  {
    let text = String::from_utf8_lossy(&out.stdout);
    if let Some(uuid) = text.lines().find(|l| l.contains("IOPlatformUUID")).and_then(|l| l.split('"').nth(3)) {
      return uuid.to_string();
    }
  }
  String::new()
}

/// Current OS username from the environment.
fn os_user() -> String {
  std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default()
}

/// Current hostname.
///
/// Prefers PATH-independent sources so the fingerprint stays stable regardless
/// of the caller's environment: the `HOSTNAME` env var, then Linux's
/// `/proc/sys/kernel/hostname`, then the `hostname` binary at its absolute path.
fn hostname() -> String {
  if let Ok(h) = std::env::var("HOSTNAME")
    && !h.is_empty()
  {
    return h;
  }
  if let Ok(h) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
    let h = h.trim();
    if !h.is_empty() {
      return h.to_string();
    }
  }
  for bin in ["/bin/hostname", "/usr/bin/hostname"] {
    if let Ok(out) = Command::new(bin).output()
      && out.status.success()
    {
      return String::from_utf8_lossy(&out.stdout).trim().to_string();
    }
  }
  String::new()
}

/// Lowercase hex encoding of a byte slice.
fn hex(bytes: &[u8]) -> String {
  let mut s = String::with_capacity(bytes.len() * 2);
  for b in bytes {
    s.push(char::from_digit((b >> 4) as u32, 16).unwrap());
    s.push(char::from_digit((b & 0x0f) as u32, 16).unwrap());
  }
  s
}

/// Resolve the live GitHub identity via the `gh` CLI (`gh api user`).
///
/// Returns `None` when `gh` is missing, unauthenticated, offline, or its output
/// cannot be parsed — the caller then falls back to the offline attestation.
pub fn resolve_online() -> Option<GithubIdentity> {
  let out = Command::new("gh").args(["api", "user", "--jq", "{id: .id, login: .login}"]).output().ok()?;
  if !out.status.success() {
    return None;
  }
  #[derive(Deserialize)]
  struct GhUser {
    id: u64,
    login: String,
  }
  let user: GhUser = serde_json::from_slice(&out.stdout).ok()?;
  Some(GithubIdentity {
    id: user.id,
    login: user.login,
  })
}

/// Read and decrypt the attestation next to `dir`, if present and valid.
pub fn read_attestation(dir: &Path) -> Option<Attestation> {
  let data = std::fs::read(dir.join(ATTEST_FILE)).ok()?;
  let plain = crate::crypto::open(&ATTEST_KEY, &data).ok()?;
  serde_json::from_slice(&plain).ok()
}

/// Seal and write `attest` next to `dir`. Errors are non-fatal (offline trust
/// simply won't be refreshed) and are returned for optional logging.
pub fn write_attestation(dir: &Path, attest: &Attestation) -> std::io::Result<()> {
  let plain = serde_json::to_vec(attest).expect("attestation serialises");
  let sealed = crate::crypto::seal(&ATTEST_KEY, &plain);
  std::fs::write(dir.join(ATTEST_FILE), sealed)
}

/// Enforce identity binding for a session.
///
/// `dir` is the directory holding `lq.toml` / the attestation. `stored_owner`
/// is the owner recorded in the (already decrypted) progress file. On success
/// returns the identity the progress is bound to and, when an online check
/// succeeded, refreshes the on-disk attestation. On denial returns the
/// [`DenyReason`] describing why.
pub fn authorize(dir: &Path, stored_owner: Option<GithubIdentity>) -> Result<GithubIdentity, DenyReason> {
  let live = resolve_online();
  let attest = read_attestation(dir);
  let fp = machine_fingerprint();
  let now = unix_now();

  match decide(stored_owner, live, attest.as_ref(), &fp, now, MAX_OFFLINE_AGE) {
    Decision::Allow { owner, refresh_attestation } => {
      if refresh_attestation {
        let fresh = Attestation {
          github_id: owner.id,
          login: owner.login.clone(),
          machine_fp: fp,
          verified_at: now,
        };
        // Best-effort: failure to persist just means no offline trust yet.
        let _ = write_attestation(dir, &fresh);
      }
      Ok(owner)
    }
    Decision::Deny(reason) => Err(reason),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ident(id: u64) -> GithubIdentity {
    GithubIdentity {
      id,
      login: format!("user{id}"),
    }
  }

  fn attestation(id: u64, fp: &str, verified_at: u64) -> Attestation {
    Attestation {
      github_id: id,
      login: format!("user{id}"),
      machine_fp: fp.to_string(),
      verified_at,
    }
  }

  const FP: &str = "machine-a";
  // A realistic "now" comfortably larger than MAX_OFFLINE_AGE to avoid underflow.
  const NOW: u64 = 2_000_000_000;

  // --- Online path ---------------------------------------------------------

  #[test]
  fn online_matching_owner_allows_and_refreshes() {
    let d = decide(Some(ident(42)), Some(ident(42)), None, FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(
      d,
      Decision::Allow {
        owner: ident(42),
        refresh_attestation: true
      }
    );
  }

  #[test]
  fn online_fresh_progress_binds_to_live_identity() {
    let d = decide(None, Some(ident(7)), None, FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(
      d,
      Decision::Allow {
        owner: ident(7),
        refresh_attestation: true
      }
    );
  }

  #[test]
  fn online_mismatch_is_denied_transfer() {
    // Alice's file (#1) opened while Bob (#2) is signed in.
    let d = decide(Some(ident(1)), Some(ident(2)), None, FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(d, Decision::Deny(DenyReason::IdentityMismatch { expected: 1, got: 2 }));
  }

  #[test]
  fn online_overrides_even_a_valid_attestation() {
    // A stale attestation for the wrong user must not rescue a mismatch.
    let att = attestation(1, FP, NOW);
    let d = decide(Some(ident(1)), Some(ident(2)), Some(&att), FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(d, Decision::Deny(DenyReason::IdentityMismatch { expected: 1, got: 2 }));
  }

  // --- Offline path with existing progress ---------------------------------

  #[test]
  fn offline_valid_attestation_allows_no_refresh() {
    let att = attestation(42, FP, NOW - 10);
    let d = decide(Some(ident(42)), None, Some(&att), FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(
      d,
      Decision::Allow {
        owner: ident(42),
        refresh_attestation: false
      }
    );
  }

  #[test]
  fn offline_no_attestation_is_denied() {
    let d = decide(Some(ident(42)), None, None, FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(d, Decision::Deny(DenyReason::OfflineNoAttestation));
  }

  #[test]
  fn offline_wrong_machine_is_denied_transfer() {
    // Bob copied Alice's progress *and* her attestation to his laptop.
    let att = attestation(42, "machine-alice", NOW - 10);
    let d = decide(Some(ident(42)), None, Some(&att), "machine-bob", NOW, MAX_OFFLINE_AGE);
    assert_eq!(d, Decision::Deny(DenyReason::AttestationMachineMismatch));
  }

  #[test]
  fn offline_expired_attestation_is_denied() {
    let att = attestation(42, FP, NOW - MAX_OFFLINE_AGE - 1);
    let d = decide(Some(ident(42)), None, Some(&att), FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(d, Decision::Deny(DenyReason::AttestationExpired));
  }

  #[test]
  fn offline_attestation_for_other_user_is_denied() {
    // Valid attestation for #99 on this machine, but the progress owner is #42.
    let att = attestation(99, FP, NOW - 10);
    let d = decide(Some(ident(42)), None, Some(&att), FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(d, Decision::Deny(DenyReason::AttestationUserMismatch));
  }

  #[test]
  fn offline_expiry_boundary_is_inclusive() {
    // Exactly at max age is still allowed; one second older is not.
    let att_edge = attestation(42, FP, NOW - MAX_OFFLINE_AGE);
    assert!(matches!(
      decide(Some(ident(42)), None, Some(&att_edge), FP, NOW, MAX_OFFLINE_AGE),
      Decision::Allow { .. }
    ));
  }

  // --- Offline bootstrap ---------------------------------------------------

  #[test]
  fn offline_fresh_progress_bootstraps_from_valid_attestation() {
    let att = attestation(5, FP, NOW - 10);
    let d = decide(None, None, Some(&att), FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(
      d,
      Decision::Allow {
        owner: ident(5),
        refresh_attestation: false
      }
    );
  }

  #[test]
  fn offline_fresh_progress_without_attestation_requires_online() {
    let d = decide(None, None, None, FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(d, Decision::Deny(DenyReason::BootstrapRequiresOnline));
  }

  #[test]
  fn offline_fresh_progress_with_wrong_machine_requires_online() {
    let att = attestation(5, "other-machine", NOW - 10);
    let d = decide(None, None, Some(&att), FP, NOW, MAX_OFFLINE_AGE);
    assert_eq!(d, Decision::Deny(DenyReason::BootstrapRequiresOnline));
  }

  // --- Supporting primitives ----------------------------------------------

  #[test]
  fn machine_fingerprint_is_stable_and_nonempty() {
    let a = machine_fingerprint();
    let b = machine_fingerprint();
    assert_eq!(a, b);
    assert_eq!(a.len(), 64); // SHA-256 hex
  }

  #[test]
  fn attestation_seals_and_reopens_via_disk() {
    let dir = std::env::temp_dir().join(format!("lq_attest_test_{}", unix_now()));
    std::fs::create_dir_all(&dir).unwrap();

    let att = attestation(123, &machine_fingerprint(), unix_now());
    write_attestation(&dir, &att).expect("write");
    let loaded = read_attestation(&dir).expect("read back");
    assert_eq!(loaded, att);

    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn tampered_attestation_file_is_ignored() {
    let dir = std::env::temp_dir().join(format!("lq_attest_tamper_{}", unix_now()));
    std::fs::create_dir_all(&dir).unwrap();

    let att = attestation(123, "m", unix_now());
    write_attestation(&dir, &att).unwrap();

    // Corrupt the sealed file — it must be treated as absent, not trusted.
    let path = dir.join(ATTEST_FILE);
    let mut bytes = std::fs::read(&path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    std::fs::write(&path, &bytes).unwrap();

    assert!(read_attestation(&dir).is_none());

    let _ = std::fs::remove_dir_all(&dir);
  }
}
