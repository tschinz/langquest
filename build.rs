// Is executed before the build process starts

use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
  let keys = ["PROGRESS_KEY", "ATTEST_KEY", "SOLUTION_KEY"];

  // Find files for build
  let project_root = std::env::var("CARGO_MANIFEST_DIR").expect("Impossible to read CARGO_MANIFEST_DIR");
  let project_path = PathBuf::from(project_root);
  let env_path = project_path.join(".env");
  let template_env_path = project_path.join(".env.template");

  // Forces recompilation if variables or .env changed, even without code changes
  if env_path.exists() {
    println!("cargo:rerun-if-changed={}", env_path.display());
  }
  if template_env_path.exists() {
    println!("cargo:rerun-if-changed={}", template_env_path.display());
  }
  for key in keys {
    println!("cargo:rerun-if-env-changed={}", key);
  }

  let mut dotenv_values = HashMap::new();
  let mut dotenv_template_values = HashMap::new();

  // Check for local .env file - existing env vars are still preferred over .env values
  if env_path.exists() {
    for entry in dotenvy::from_path_iter(&env_path).unwrap_or_else(|_| panic!("❌ Impossible to load the .env file from : {}", env_path.display())) {
      let (key, value) = entry.unwrap_or_else(|_| panic!("❌ Impossible to read an entry from : {}", env_path.display()));
      dotenv_values.insert(key, value);
    }
  }

  // Check for local .env.template file - existing env vars are still preferred over .env values
  if template_env_path.exists() {
    for entry in dotenvy::from_path_iter(&template_env_path)
      .unwrap_or_else(|_| panic!("❌ Impossible to load the .env.template file from : {}", template_env_path.display()))
    {
      let (key, value) = entry.unwrap_or_else(|_| panic!("❌ Impossible to read an entry from : {}", template_env_path.display()));
      dotenv_template_values.insert(key, value);
    }
  }

  // Retrieve keys
  for key in keys {
    let value = std::env::var(key)
      .ok()
      .inspect(|_| {
        println!("cargo:warning=👌 Using value from environment variables for {}", key);
      })
      .or_else(|| {
        dotenv_values.get(key).cloned().inspect(|_| {
          println!("cargo:warning=👌 Using value from .env for {}", key);
        })
      })
      .or_else(|| {
        dotenv_template_values.get(key).cloned().inspect(|_| {
          println!("cargo:warning=⚠️  Using value from .env.template for {}", key);
        })
      })
      .unwrap_or_else(|| {
        panic!(
          "❌ Compilation error : The key '{}' is missing in your environment, .env and .env.example files",
          key
        )
      });
    if value.len() != 32 {
      panic!(
        "❌ Compilation error : The key '{}' must be exactly 32 bytes long (current length: {} bytes)",
        key,
        value.len()
      );
    }
    // Inject inside compiler
    println!("cargo:rustc-env={}={}", key, value);
  }
}
