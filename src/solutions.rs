//! Sealing of solution files for two-tier exercise repositories.
//!
//! Exercise repos come in two flavours:
//! * **Teacher** — the source of truth, with `solution/solution.md` and
//!   `solution/main.*` in readable plaintext.
//! * **Student** — a published copy where the contents of every `solution/`
//!   directory are encrypted so they cannot be read by opening the files.
//!
//! A CI job on the teacher repo runs [`seal_solutions_in`] and pushes the result
//! to the student repo. LangQuest reads solution files through
//! [`read_maybe_sealed`], which transparently accepts either form (detected by
//! the sealed-file magic marker), so the *same binary* serves both repos.
//!
//! Security note: the key is embedded in the binary, so this prevents casual
//! reading of solutions (in an editor, via `git`, etc.), not extraction by a
//! determined reverse-engineer — the same ceiling as the rest of the system.

use std::fs;
use std::io;
use std::path::Path;

use crate::crypto;

/// Embedded key used to seal solution files (everything under a `solution/` dir).
///
/// Distinct from the progress/attestation keys so the concerns stay separated.
const SOLUTION_KEY: [u8; 32] = *b"lq-solution-key-v1-classroom-ok!";

/// Read a file that may be plaintext or sealed, returning its text contents.
///
/// If the file begins with the lq sealed magic marker it is decrypted with the
/// embedded solution key; otherwise its bytes are returned as UTF-8. This is the
/// single entry point LangQuest uses to read solution files, so a plaintext
/// teacher repo and an encrypted student repo behave identically.
pub fn read_maybe_sealed(path: &Path) -> io::Result<String> {
  let bytes = fs::read(path)?;
  let plain = if crypto::is_sealed(&bytes) {
    crypto::open(&SOLUTION_KEY, &bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
  } else {
    bytes
  };
  String::from_utf8(plain).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Seal every file under each `solution/` directory in `root`, in place.
///
/// Idempotent: files that are already sealed are left untouched, so re-running
/// in CI (or on an already-published tree) is safe. Returns the number of files
/// newly sealed.
pub fn seal_solutions_in(root: &Path) -> io::Result<usize> {
  let mut count = 0;
  walk(root, false, Transform::Seal, &mut count)?;
  Ok(count)
}

/// Decrypt every sealed file under each `solution/` directory in `root`, in
/// place (the inverse of [`seal_solutions_in`]). Idempotent. Returns the number
/// of files decrypted.
///
/// Intentionally **not** exposed as an `lq` subcommand: shipping a bulk-decrypt
/// command would let a student recover every plaintext solution from a sealed
/// student repo. It exists only as a library entry point (used by tests and any
/// teacher-side tooling built against the crate); the teacher's plaintext source
/// repo is the record of truth.
pub fn unseal_solutions_in(root: &Path) -> io::Result<usize> {
  let mut count = 0;
  walk(root, false, Transform::Unseal, &mut count)?;
  Ok(count)
}

/// Direction of an in-place transform.
#[derive(Clone, Copy)]
enum Transform {
  Seal,
  Unseal,
}

/// Recursively walk `dir`, applying `transform` to every file that lives under a
/// `solution/` directory. `in_solution` is `true` once we have descended into
/// such a directory. Hidden entries (`.git`, `.lq.*`, …) are skipped.
fn walk(dir: &Path, in_solution: bool, transform: Transform, count: &mut usize) -> io::Result<()> {
  for entry in fs::read_dir(dir)? {
    let entry = entry?;
    let name = entry.file_name();
    let name = name.to_string_lossy();
    if name.starts_with('.') {
      continue; // never descend into .git or touch dotfiles
    }

    let file_type = entry.file_type()?;
    let path = entry.path();

    if file_type.is_dir() {
      let now_in_solution = in_solution || name == "solution";
      walk(&path, now_in_solution, transform, count)?;
    } else if file_type.is_file() && in_solution && transform_file(&path, transform)? {
      *count += 1;
    }
  }
  Ok(())
}

/// Apply `transform` to a single file in place. Returns `true` if the file was
/// changed (a no-op when already in the target state).
fn transform_file(path: &Path, transform: Transform) -> io::Result<bool> {
  let bytes = fs::read(path)?;
  let sealed = crypto::is_sealed(&bytes);

  let out = match transform {
    Transform::Seal if !sealed => crypto::seal(&SOLUTION_KEY, &bytes),
    Transform::Unseal if sealed => crypto::open(&SOLUTION_KEY, &bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
    // Already in the desired state — idempotent no-op.
    _ => return Ok(false),
  };

  fs::write(path, out)?;
  Ok(true)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("lq_solutions_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[test]
  fn read_maybe_sealed_passes_through_plaintext() {
    let dir = tmp_dir("plain");
    let f = dir.join("solution.md");
    fs::write(&f, "# hello\nplain text").unwrap();
    assert_eq!(read_maybe_sealed(&f).unwrap(), "# hello\nplain text");
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn read_maybe_sealed_decrypts_sealed() {
    let dir = tmp_dir("sealed");
    let f = dir.join("main.rs");
    let original = "fn main() { println!(\"answer\"); }";
    fs::write(&f, crypto::seal(&SOLUTION_KEY, original.as_bytes())).unwrap();

    // On disk it must not be readable plaintext…
    let raw = fs::read(&f).unwrap();
    assert!(crypto::is_sealed(&raw));
    assert!(!raw.windows(6).any(|w| w == b"answer"));
    // …but read_maybe_sealed recovers it.
    assert_eq!(read_maybe_sealed(&f).unwrap(), original);
    let _ = fs::remove_dir_all(&dir);
  }

  /// Build a mini exercise repo with one plaintext solution and return its root.
  fn repo_with_solution(tag: &str) -> std::path::PathBuf {
    let root = tmp_dir(tag);
    let sol = root.join("01-mod").join("01-ex").join("solution");
    fs::create_dir_all(&sol).unwrap();
    fs::write(sol.join("solution.md"), "---\ntitle = \"X\"\n---\nexplain").unwrap();
    fs::write(sol.join("main.rs"), "fn main() {}").unwrap();
    // A student working file OUTSIDE solution/ that must stay untouched.
    fs::write(root.join("01-mod").join("01-ex").join("main.rs"), "// starter").unwrap();
    root
  }

  #[test]
  fn seal_only_touches_solution_dirs_and_is_idempotent() {
    let root = repo_with_solution("seal");
    let starter = root.join("01-mod").join("01-ex").join("main.rs");
    let sol_md = root.join("01-mod").join("01-ex").join("solution").join("solution.md");

    let n = seal_solutions_in(&root).unwrap();
    assert_eq!(n, 2, "should seal solution.md + solution/main.rs");

    // Student starter file untouched (still plaintext).
    assert_eq!(fs::read_to_string(&starter).unwrap(), "// starter");
    // Solution file is now sealed on disk but readable via the helper.
    assert!(crypto::is_sealed(&fs::read(&sol_md).unwrap()));
    assert!(read_maybe_sealed(&sol_md).unwrap().contains("explain"));

    // Re-running seals nothing new (idempotent).
    assert_eq!(seal_solutions_in(&root).unwrap(), 0);
    let _ = fs::remove_dir_all(&root);
  }

  #[test]
  fn seal_then_unseal_restores_original() {
    let root = repo_with_solution("roundtrip");
    let sol_md = root.join("01-mod").join("01-ex").join("solution").join("solution.md");
    let before = fs::read(&sol_md).unwrap();

    assert_eq!(seal_solutions_in(&root).unwrap(), 2);
    assert_eq!(unseal_solutions_in(&root).unwrap(), 2);

    assert_eq!(fs::read(&sol_md).unwrap(), before);
    // Unsealing an already-plaintext tree is a no-op.
    assert_eq!(unseal_solutions_in(&root).unwrap(), 0);
    let _ = fs::remove_dir_all(&root);
  }

  #[test]
  fn corrupt_sealed_file_errors() {
    let dir = tmp_dir("corrupt");
    let f = dir.join("solution.md");
    let mut sealed = crypto::seal(&SOLUTION_KEY, b"secret");
    let last = sealed.len() - 1;
    sealed[last] ^= 0xff;
    fs::write(&f, sealed).unwrap();
    assert!(read_maybe_sealed(&f).is_err());
    let _ = fs::remove_dir_all(&dir);
  }
}
