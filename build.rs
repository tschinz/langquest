// Is executed before the build process starts

use std::fs;
use std::path::PathBuf;

fn main() {
  // Find files for build
  let project_root = std::env::var("CARGO_MANIFEST_DIR").expect("Impossible de lire CARGO_MANIFEST_DIR");
  let project_path = PathBuf::from(project_root);
  let env_path = project_path.join(".env");
  let template_path = project_path.join(".env.template");

  // Forces recompilation if .env changed but no code
  println!("cargo:rerun-if-changed={}", env_path.display());

  // Check files exist, create .env from template if missing
  if !env_path.exists() {
    if template_path.exists() {
      fs::copy(&template_path, &env_path).unwrap();
      // Affiche un avertissement jaune dans le terminal de compilation Cargo
      println!("cargo:warning=⚠️  The .env file was missing. A new file has been created automatically from .env.template.");
    } else {
      panic!("❌ Error :the .env and .env.template files are both missing from {}", project_path.display());
    }
  }

  // Loads .env
  dotenvy::from_path(&env_path).unwrap_or_else(|_| panic!("❌ Impossible to load the .env file from : {}", env_path.display()));

  // Retrieve keys
  let keys = ["PROGRESS_KEY", "ATTEST_KEY", "SOLUTION_KEY"];
  for key in keys {
    let value = std::env::var(key).unwrap_or_else(|_| panic!("❌ Compilation error : The key '{}' is missing in your .env file.", key));
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
