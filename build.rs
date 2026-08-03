// Is executed before the build process starts

use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
  // Find files for build
  let project_root = std::env::var("CARGO_MANIFEST_DIR").expect("Impossible to read CARGO_MANIFEST_DIR");
  let project_path = PathBuf::from(project_root);
  let env_path = project_path.join(".env");

  // Forces recompilation if variables or .env changed, even without code changes
  println!("cargo:rerun-if-changed={}", env_path.display());
  for key in ["PROGRESS_KEY", "ATTEST_KEY", "SOLUTION_KEY"] {
    println!("cargo:rerun-if-env-changed={}", key);
  }

  let keys = ["PROGRESS_KEY", "ATTEST_KEY", "SOLUTION_KEY"];
  let mut dotenv_values = HashMap::new();

  // Check for local .env file - existing env vars are still preferred over .env values
  if env_path.exists() {
    for entry in dotenvy::from_path_iter(&env_path).unwrap_or_else(|_| panic!("❌ Impossible to load the .env file from : {}", env_path.display())) {
      let (key, value) = entry.unwrap_or_else(|_| panic!("❌ Impossible to read an entry from : {}", env_path.display()));
      dotenv_values.insert(key, value);
    }
  }

  // Retrieve keys
  for key in keys {
    let value = std::env::var(key)
      .ok()
      .or_else(|| dotenv_values.get(key).cloned())
      .unwrap_or_else(|| panic!("❌ Compilation error : The key '{}' is missing in your environment and .env file.", key));
    if value.len() != 32 {
      panic!(
        "❌ Compilation error : The key '{}' must be exactly 32 bytes long (current length: {} bytes).",
        key,
        value.len()
      );
    }
    // Inject inside compiler
    println!("cargo:rustc-env={}={}", key, value);
  }
}
