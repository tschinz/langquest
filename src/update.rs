//! Update check: compare the running version against the latest GitHub release.
//!
//! Best-effort, failed checks display a notice

use anyhow::Context;
use semver::Version;

const RELEASE_API_URL: &str = "https://api.github.com/repos/tschinz/langquest/releases/latest";
const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// Fetch latest release tag
fn fetch_latest_version() -> anyhow::Result<Version> {
  let useragent = ureq::Agent::config_builder().timeout_global(Some(REQUEST_TIMEOUT)).build().new_agent();

  let mut response = useragent
    .get(RELEASE_API_URL)
    .header("Accept", "application/vnd.github+json")
    .header("User-Agent", concat!("lq/", env!("CARGO_PKG_VERSION")))
    .call()
    .context("request failed")?;

  let body = response.body_mut().read_to_string().context("reading body failed")?;
  let json: serde_json::Value = serde_json::from_str(&body).context("parsing response failed")?;
  let tag = json.get("tag_name").and_then(serde_json::Value::as_str).context("no tag_name in response")?;
  Version::parse(tag.trim_start_matches('v')).context("invalid release tag")
}

/// Return a notice string: this might be the update hint or what went wrong during the check.
/// Return `None` if langquest is up to date.
pub fn check_for_update() -> Option<String> {
  let current = Version::parse(env!("CARGO_PKG_VERSION")).expect("valid built-in version");

  match fetch_latest_version() {
    Err(err) => Some(format!("Update check failed ({err}); skipping.")),
    Ok(latest) if latest > current => Some(format!(
      "Update available: v{} -> v{} (https://github.com/tschinz/langquest/releases)",
      current, latest
    )),
    Ok(_) => None,
  }
}

/// Check for an update. If one is available, offer to run the installer.
/// Returns `true` if an update was made which should result in langquest exiting.
pub fn check_and_prompt() -> bool {
  use std::io::Write as _; // For traits

  let Some(notice) = check_for_update() else {
    return false;
  };
  eprintln!("   {notice}");

  if notice.starts_with("Update available") {
    eprintln!("   Install with `{}`", INSTALL_COMMAND);
    eprint!("   Run it now? [y/N] ");
    let _ = std::io::stderr().flush();

    let mut input = String::new();
    let _ = std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut input);

    if input.trim().eq_ignore_ascii_case("y") {
      let _ = run_install();
      return true;
    }
  }
  false
}

/// One-liner that installs the latest release, per platform.
#[cfg(windows)]
pub const INSTALL_COMMAND: &str = "irm https://raw.githubusercontent.com/tschinz/langquest/refs/heads/main/scripts/install_latest_release.ps1 | iex";
#[cfg(not(windows))]
pub const INSTALL_COMMAND: &str = "curl -fsSL https://raw.githubusercontent.com/tschinz/langquest/refs/heads/main/scripts/install_latest_release.sh | sh";

/// Run the install command.
pub fn run_install() -> anyhow::Result<()> {
  if cfg!(windows) {
    std::process::Command::new("powershell")
      .args(["-NoProfile", "-Command", INSTALL_COMMAND])
      .status()?;
  } else {
    std::process::Command::new("sh").args(["-c", INSTALL_COMMAND]).status()?;
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_latest_version_rejects_garbage() {
    assert!(Version::parse("not-a-version").is_err());
  }

  #[test]
  fn check_for_update_returns_notice_or_none() {
    if let Some(notice) = check_for_update() {
      assert!(
        notice.starts_with("Update available") || notice.starts_with("Update check failed"),
        "unexpected notice: {notice}"
      );
    }
  }
}
