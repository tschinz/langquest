##################################################
# Variables
#

rust_env := "rustup show"
rust_edition := "2024"
open := if os() == "linux" { "xdg-open" } else if os() == "macos" { "open" } else { "start \"\" /max" }
app_name := "lq"
crate_name := "lq"
args := ""
project_directory := justfile_directory()
release := `git describe --tags --always`
version := "0.1.0"
url := "https://github.com/tschinz/langquest"
test_repo := justfile_directory() / "tests" / "sample-repo"

##################################################
# Default
#

# List all available commands
default:
    @just --list

##################################################
# Info & Dependencies
#

# Print environment info (OS, arch, toolchains)
info:
    #!/usr/bin/env bash
    set +e
    echo "OS          : {{ os() }}"
    echo "Arch        : {{ arch() }}"
    echo "Project     : {{ project_directory }}"
    echo "App         : {{ app_name }}"
    echo "Version     : {{ version }}"
    echo "Test repo   : {{ test_repo }}"
    echo ""
    echo "--- Rust toolchain (lq build) ---"
    rustup show 2>/dev/null || echo "rustup not found"
    echo ""
    echo "--- Exercise toolchains ---"
    printf "  %-12s %s\n" "rustc"   "$(rustc --version 2>/dev/null   || echo 'NOT FOUND — needed for Rust exercises')"
    printf "  %-12s %s\n" "python3" "$(python3 --version 2>/dev/null || echo 'NOT FOUND — needed for Python exercises')"
    if pytest_v=$(python3 -m pytest --version 2>/dev/null); then
        printf "  %-12s %s\n" "pytest"  "$pytest_v"
    else
        printf "  %-12s %s\n" "pytest"  "not found — optional, falls back to unittest"
    fi
    printf "  %-12s %s\n" "go"      "$(go version 2>/dev/null        || echo 'NOT FOUND — needed for Go exercises')"
    if [[ -n "${RIPES_PATH:-}" ]]; then
        printf "  %-12s %s\n" "ripes" "RIPES_PATH=${RIPES_PATH}"
    elif command -v ripes >/dev/null 2>&1; then
        printf "  %-12s %s\n" "ripes" "$(ripes --version 2>/dev/null || echo "found at $(which ripes)")"
    elif [[ -x "{{ project_directory }}/ripes/macos/Ripes.app/Contents/MacOS/Ripes" ]]; then
        printf "  %-12s %s\n" "ripes" "bundled binary: ripes/macos/Ripes.app/Contents/MacOS/Ripes"
    else
        printf "  %-12s %s\n" "ripes" "NOT FOUND — set RIPES_PATH or see https://github.com/mortbopet/Ripes/releases"
    fi

# Check that all required tools are available (build tools are fatal; exercise toolchains warn)
check-deps:
    #!/usr/bin/env bash
    set +e
    errors=0
    warnings=0

    ok()   { printf "  ✓ %-10s %s\n" "$1" "$2"; }
    warn() { printf "  ⚠ %-10s %s\n" "$1" "$2"; warnings=$((warnings+1)); }
    fail() { printf "  ✗ %-10s %s\n" "$1" "$2"; errors=$((errors+1)); }

    echo "--- lq build tools ---"
    if v=$(cargo --version 2>/dev/null);        then ok   "cargo"   "$v"; else fail "cargo"   "not found — install Rust: https://rustup.rs"; fi
    if v=$(rustfmt --version 2>/dev/null);      then ok   "rustfmt" "$v"; else fail "rustfmt" "not found — run: rustup component add rustfmt"; fi
    if v=$(cargo clippy --version 2>/dev/null); then ok   "clippy"  "$v"; else fail "clippy"  "not found — run: rustup component add clippy"; fi

    echo ""
    echo "--- Exercise toolchains ---"
    if v=$(rustc --version 2>/dev/null);   then ok   "rustc"   "$v"; else fail "rustc"   "not found — needed for Rust exercises (install: https://rustup.rs)"; fi
    if v=$(python3 --version 2>/dev/null); then ok   "python3" "$v"; else fail "python3" "not found — needed for Python exercises (brew install python)"; fi
    if v=$(python3 -m pytest --version 2>/dev/null); then
                                                ok   "pytest"  "$v"
    else
                                                warn "pytest"  "not found — optional, Python exercises will fall back to unittest"
    fi
    if v=$(go version 2>/dev/null);        then ok   "go"      "$v"; else fail "go"      "not found — needed for Go exercises (brew install go)"; fi

    echo ""
    echo "--- Ripes (RISC-V simulator) ---"
    if [[ -n "${RIPES_PATH:-}" ]]; then
        ok "ripes" "RIPES_PATH=${RIPES_PATH}"
    elif command -v ripes >/dev/null 2>&1; then
        ok "ripes" "found in PATH: $(which ripes)"
    elif [[ -x "{{ project_directory }}/ripes/macos/Ripes.app/Contents/MacOS/Ripes" ]]; then
        ok "ripes" "bundled binary: ripes/macos/Ripes.app/Contents/MacOS/Ripes"
    elif [[ -x "{{ project_directory }}/ripes/linux/Ripes.AppImage" ]]; then
        ok "ripes" "bundled binary: ripes/linux/Ripes.AppImage"
    elif [[ -x "{{ project_directory }}/ripes/win/Ripes.exe" ]]; then
        ok "ripes" "bundled binary: ripes/win/Ripes.exe"
    else
        warn "ripes" "not found — RISC-V exercises won't run; set RIPES_PATH or see https://github.com/mortbopet/Ripes/releases"
    fi

    echo ""
    if [[ $errors -gt 0 ]]; then
        echo "✗ $errors error(s) found — fix the above before building or running exercises."
        exit 1
    elif [[ $warnings -gt 0 ]]; then
        echo "✓ All required tools present ($warnings optional warning(s) — see above)."
    else
        echo "✓ All dependencies satisfied."
    fi

# Setup all dependencies on macOS via Homebrew (installs brew itself if absent)
setup-macos:
    #!/usr/bin/env bash
    set -euo pipefail

    # ── Homebrew ──────────────────────────────────────────────────────────────
    if ! command -v brew >/dev/null 2>&1; then
        echo "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        # Add brew to PATH for Apple Silicon (M1/M2/M3)
        if [[ -f "/opt/homebrew/bin/brew" ]]; then
            eval "$(/opt/homebrew/bin/brew shellenv)"
        fi
    else
        echo "✓ Homebrew: $(brew --version | head -1)"
    fi

    # ── Rust (via rustup — preferred over the brew formula for development) ───
    if ! command -v rustup >/dev/null 2>&1; then
        echo "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
        source "${HOME}/.cargo/env"
    else
        echo "✓ rustup: $(rustup --version 2>/dev/null)"
    fi
    rustup component add rustfmt clippy 2>/dev/null
    echo "✓ rustc:   $(rustc --version)"
    echo "✓ rustfmt: $(rustfmt --version)"
    echo "✓ clippy:  $(cargo clippy --version)"

    # ── Python ────────────────────────────────────────────────────────────────
    if ! command -v python3 >/dev/null 2>&1; then
        echo "Installing Python..."
        brew install python
    else
        echo "✓ python3: $(python3 --version)"
    fi
    if ! python3 -m pytest --version >/dev/null 2>&1; then
        echo "Installing pytest..."
        pip3 install --quiet pytest
    else
        echo "✓ pytest:  $(python3 -m pytest --version 2>/dev/null | head -1)"
    fi

    # ── Go ────────────────────────────────────────────────────────────────────
    if ! command -v go >/dev/null 2>&1; then
        echo "Installing Go..."
        brew install go
    else
        echo "✓ go:      $(go version)"
    fi

    # ── Ripes (RISC-V simulator) ──────────────────────────────────────────────
    # Ripes is not available via Homebrew. lq ships a bundled macOS binary;
    # users can also set RIPES_PATH to point at their own installation.
    echo ""
    echo "--- Ripes (RISC-V simulator) ---"
    if [[ -n "${RIPES_PATH:-}" ]]; then
        echo "✓ RIPES_PATH=${RIPES_PATH} (using env var)"
    elif command -v ripes >/dev/null 2>&1; then
        echo "✓ ripes found in PATH: $(which ripes)"
    elif [[ -x "{{ project_directory }}/ripes/macos/Ripes.app/Contents/MacOS/Ripes" ]]; then
        echo "✓ Bundled binary: ripes/macos/Ripes.app/Contents/MacOS/Ripes"
        echo "  (lq will use it automatically — no further action needed)"
    else
        echo "  Ripes is not available via Homebrew."
        echo "  Download from: https://github.com/mortbopet/Ripes/releases"
        echo "  Then set: export RIPES_PATH=/path/to/Ripes.app/Contents/MacOS/Ripes"
    fi

    echo ""
    echo "✓ Installation complete. Run 'just check-deps' to verify."

# create a release version of the program
changelog version=version:
  git cliff --unreleased --tag {{version}} --prepend CHANGELOG.md


##################################################
# Build & Run
#

# install the release version (default is the latest)
install-release release=release:
    cargo install --git {{ url }} --tag {{ release }}

# install the nightly release
install-nightly:
    cargo install --git {{ url }}

# Build and copy the release version of the program
build:
    cargo build --release
    mkdir -p bin && cp target/release/{{ app_name }} bin/


# Run the program in debug mode
run args=args:
    cargo run -- {{ args }}

# Run cargo check (fast compile check, no codegen)
check:
    cargo check

##################################################
# Test & Lint
#

# Run all tests
test:
    cargo test

# Run clippy with strict warnings
clippy:
    cargo clippy -- -D warnings

# Format source with rustfmt (edition 2024)
rustfmt:
    cargo fmt --all

##################################################
# Quick-test shortcuts (uses test_repo)
#

# Run `lq --repo <test_repo> status` for quick testing
status-test:
    cargo run -- --repo {{ test_repo }} status

# Run `lq --repo <test_repo> --reset` for quick testing
reset-test:
    cargo run -- --repo {{ test_repo }} --reset

# Run `lq --repo <test_repo> --reset` for quick testing
run-test:
    cargo run -- --repo {{ test_repo }}

##################################################
# Documentation
#

# Generate and open rustdoc documentation
doc:
    @echo "Generating rustdoc documentation..."
    cargo doc --no-deps --document-private-items
    @echo "✓ Documentation generated"
    @echo "Opening documentation in browser..."
    {{ open }} target/doc/{{ crate_name }}/index.html

# Generate rustdoc documentation without opening
doc-build:
    @echo "Generating rustdoc documentation..."
    cargo doc --no-deps --document-private-items
    @echo "✓ Documentation generated at target/doc/{{ crate_name }}/index.html"

# Generate SBOM for Dependecy Track
sbom:
    cargo sbom --output-format cyclone_dx_json_1_6 >> target/sbom-cyclone_dx_1_6.json

# Upload SBOM to Dependency Track (requires DT_API_KEY, DT_PROJECT_UUID, DT_BASE_URL env vars)
sbom-upload:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Uploading SBOM to Dependency Track..."
    # Load .env file if it exists
    if [[ -f .env ]]; then
        echo "Loading configuration from .env file..."
        export $(grep -v '^#' .env | grep -v '^$' | xargs)
    fi
    if [[ -z "${DT_API_KEY:-}" ]] || [[ -z "${DT_PROJECT_UUID:-}" ]] || [[ -z "${DT_BASE_URL:-}" ]]; then
        echo "Error: Required environment variables not set:"
        echo "  DT_API_KEY - Your Dependency Track API key"
        echo "  DT_PROJECT_UUID - Your project UUID"
        echo "  DT_BASE_URL - Your Dependency Track base URL"
        echo ""
        echo "Example:"
        echo "  export DT_BASE_URL=https://dt-api.zahno.dev"
        echo "  export DT_API_KEY=your_api_key_here"
        echo "  export DT_PROJECT_UUID=your_project_uuid_here"
        echo "  just sbom-upload"
        exit 1
    fi
    just sbom
    curl -X POST "${DT_BASE_URL}/api/v1/bom" \
        -H "X-Api-Key: ${DT_API_KEY}" \
        -H "Content-Type: multipart/form-data" \
        -F "project=${DT_PROJECT_UUID}" \
        -F "bom=@target/sbom-cyclone_dx_1_6.json"
    echo "✓ SBOM uploaded successfully to Dependency Track"

# Trivy comprehensive security scan (alias for backwards compatibility)
trivy:
    trivy fs --scanners vuln,secret,misconfig --format table .

##################################################
# Clean
#

# Clean build artifacts and test detritus
clean:
    cargo clean
    @rm -rf {{ project_directory / "bin" }}
    @echo "Cleaning test artifacts..."
    @find {{ project_directory }} -name ".lq_test" -exec rm -rf {} + 2>/dev/null || true
    @find {{ project_directory }} -name ".lq_main" -exec rm -rf {} + 2>/dev/null || true
    @find {{ project_directory }} -name ".lq_main.o" -exec rm -rf {} + 2>/dev/null || true
    @find {{ project_directory / "tests" }} -name "lq.toml" -exec rm -f {} + 2>/dev/null || true
    @echo "Clean complete."

##################################################
# Release Readiness
#

# Check steps for publishing is_lib ["true"|"false"]
publish-check is_lib="false":
  #!/usr/bin/env bash
  echo "Run all tests"
  cargo test
  echo "Run clippy"
  cargo clippy
  echo "Format code"
  cargo fmt --all
  echo "Build documentation"
  cargo doc --open
  echo "Test documentation examples"
  if [ "{{is_lib}}" = "true" ]; then
    cargo test --doc
  fi
  echo "Run benchmarks (if available)"
  cargo bench
  echo "Run security audit"
  cargo audit
  echo "Test Publishing"
  cargo publish --dry-run
