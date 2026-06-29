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

# For windows shell to be supported (suppose code is multi-platforms ready)
set shell := ["bash", "-uc"]
set windows-shell := ["cmd.exe", "/c"]

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
    printf "  %-12s %s\n" "rustc"   "$(rustc --version 2>/dev/null   || echo 'NOT FOUND - needed for Rust exercises')"
    printf "  %-12s %s\n" "python3" "$(python3 --version 2>/dev/null || echo 'NOT FOUND - needed for Python exercises')"
    if pytest_v=$(python3 -m pytest --version 2>/dev/null); then
        printf "  %-12s %s\n" "pytest"  "$pytest_v"
    else
        printf "  %-12s %s\n" "pytest"  "not found - optional, falls back to unittest"
    fi
    printf "  %-12s %s\n" "go"      "$(go version 2>/dev/null        || echo 'NOT FOUND - needed for Go exercises')"
    printf "  %-12s %s\n" "g++"     "$(g++ --version 2>/dev/null | head -1 || echo 'NOT FOUND - needed for C++ exercises')"
    if pkg-config --exists catch2-with-main 2>/dev/null; then
        printf "  %-12s %s\n" "catch2"  "$(pkg-config --modversion catch2-with-main 2>/dev/null)"
    else
        printf "  %-12s %s\n" "catch2"  "not found - needed for C++ exercises (brew install catch2)"
    fi
    if [[ -n "${RIPES_PATH:-}" ]]; then
        printf "  %-12s %s\n" "ripes" "RIPES_PATH=${RIPES_PATH}"
    elif command -v ripes >/dev/null 2>&1; then
        printf "  %-12s %s\n" "ripes" "$(ripes --version 2>/dev/null || echo "found at $(which ripes)")"
    elif [[ -x "{{ project_directory }}/ripes/macos/Ripes.app/Contents/MacOS/Ripes" ]]; then
        printf "  %-12s %s\n" "ripes" "bundled binary: ripes/macos/Ripes.app/Contents/MacOS/Ripes"
    else
        printf "  %-12s %s\n" "ripes" "NOT FOUND - set RIPES_PATH or see https://github.com/mortbopet/Ripes/releases"
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
    if v=$(cargo --version 2>/dev/null);        then ok   "cargo"   "$v"; else fail "cargo"   "not found - install Rust: https://rustup.rs"; fi
    if v=$(rustfmt --version 2>/dev/null);      then ok   "rustfmt" "$v"; else fail "rustfmt" "not found - run: rustup component add rustfmt"; fi
    if v=$(cargo clippy --version 2>/dev/null); then ok   "clippy"  "$v"; else fail "clippy"  "not found - run: rustup component add clippy"; fi

    echo ""
    echo "--- Exercise toolchains ---"
    if v=$(rustc --version 2>/dev/null);   then ok   "rustc"   "$v"; else fail "rustc"   "not found - needed for Rust exercises (install: https://rustup.rs)"; fi
    if v=$(python3 --version 2>/dev/null); then ok   "python3" "$v"; else fail "python3" "not found - needed for Python exercises (brew install python)"; fi
    if v=$(python3 -m pytest --version 2>/dev/null); then
                                                ok   "pytest"  "$v"
    else
                                                warn "pytest"  "not found - optional, Python exercises will fall back to unittest"
    fi
    if v=$(go version 2>/dev/null);        then ok   "go"      "$v"; else fail "go"      "not found - needed for Go exercises (brew install go)"; fi
    if v=$(g++ --version 2>/dev/null | head -1); then ok   "g++"     "$v"; else fail "g++"     "not found - needed for C++ exercises (brew install gcc or xcode-select --install)"; fi
    if pkg-config --exists catch2-with-main 2>/dev/null; then
        v=$(pkg-config --modversion catch2-with-main 2>/dev/null)
                                                ok   "catch2"  "v$v"
    else
                                                warn "catch2"  "not found - C++ exercises won't compile (brew install catch2)"
    fi

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
        warn "ripes" "not found - RISC-V exercises won't run; set RIPES_PATH or see https://github.com/mortbopet/Ripes/releases"
    fi

    echo ""
    if [[ $errors -gt 0 ]]; then
        echo "✗ $errors error(s) found - fix the above before building or running exercises."
        exit 1
    elif [[ $warnings -gt 0 ]]; then
        echo "✓ All required tools present ($warnings optional warning(s) - see above)."
    else
        echo "✓ All dependencies satisfied."
    fi

# Setup all dependencies (platform-specific: macOS / Linux / Windows)
[macos]
setup:
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

    # ── Rust (via rustup - preferred over the brew formula for development) ───
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

    # ── C++ (g++ + Catch2) ────────────────────────────────────────────────────
    if ! command -v g++ >/dev/null 2>&1; then
        echo "g++ not found - installing Xcode Command Line Tools..."
        xcode-select --install 2>/dev/null || echo "  (already installing or installed)"
    else
        echo "✓ g++:     $(g++ --version | head -1)"
    fi
    if ! pkg-config --exists catch2-with-main 2>/dev/null; then
        echo "Installing Catch2..."
        brew install catch2
    else
        echo "✓ catch2:  v$(pkg-config --modversion catch2-with-main)"
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
        echo "  (lq will use it automatically - no further action needed)"
    else
        echo "  Ripes is not available via Homebrew."
        echo "  Download from: https://github.com/mortbopet/Ripes/releases"
        echo "  Then set: export RIPES_PATH=/path/to/Ripes.app/Contents/MacOS/Ripes"
    fi

    echo ""
    echo "✓ Installation complete. Run 'just check-deps' to verify."

[linux]
setup:
    #!/usr/bin/env bash
    set -euo pipefail

    # ── Detect package manager ────────────────────────────────────────────────
    if command -v apt-get >/dev/null 2>&1; then
        PM="apt"
        INSTALL="sudo apt-get install -y"
        UPDATE="sudo apt-get update -qq"
    elif command -v dnf >/dev/null 2>&1; then
        PM="dnf"
        INSTALL="sudo dnf install -y"
        UPDATE="true"
    elif command -v pacman >/dev/null 2>&1; then
        PM="pacman"
        INSTALL="sudo pacman -S --noconfirm --needed"
        UPDATE="sudo pacman -Sy"
    else
        echo "✗ No supported package manager found (apt, dnf, pacman)."
        echo "  Install dependencies manually and run 'just check-deps' to verify."
        exit 1
    fi
    echo "Detected package manager: $PM"
    $UPDATE

    # ── Rust (via rustup) ─────────────────────────────────────────────────────
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
        case $PM in
            apt)    $INSTALL python3 python3-pip python3-venv ;;
            dnf)    $INSTALL python3 python3-pip ;;
            pacman) $INSTALL python python-pip ;;
        esac
    else
        echo "✓ python3: $(python3 --version)"
    fi
    if ! python3 -m pytest --version >/dev/null 2>&1; then
        echo "Installing pytest..."
        pip3 install --quiet --user pytest 2>/dev/null || python3 -m pip install --quiet --user pytest
    else
        echo "✓ pytest:  $(python3 -m pytest --version 2>/dev/null | head -1)"
    fi

    # ── Go ────────────────────────────────────────────────────────────────────
    if ! command -v go >/dev/null 2>&1; then
        echo "Installing Go..."
        case $PM in
            apt)    $INSTALL golang-go ;;
            dnf)    $INSTALL golang ;;
            pacman) $INSTALL go ;;
        esac
    else
        echo "✓ go:      $(go version)"
    fi

    # ── C++ (g++ + pkg-config + Catch2) ───────────────────────────────────────
    if ! command -v g++ >/dev/null 2>&1; then
        echo "Installing g++..."
        case $PM in
            apt)    $INSTALL g++ ;;
            dnf)    $INSTALL gcc-c++ ;;
            pacman) $INSTALL gcc ;;
        esac
    else
        echo "✓ g++:     $(g++ --version | head -1)"
    fi
    if ! command -v pkg-config >/dev/null 2>&1; then
        echo "Installing pkg-config..."
        case $PM in
            apt)    $INSTALL pkg-config ;;
            dnf)    $INSTALL pkgconf-pkg-config ;;
            pacman) $INSTALL pkgconf ;;
        esac
    fi
    if ! pkg-config --exists catch2-with-main 2>/dev/null; then
        echo "Installing Catch2..."
        case $PM in
            apt)    $INSTALL catch2 ;;
            dnf)    $INSTALL catch2-devel ;;
            pacman) $INSTALL catch2 ;;
        esac
    else
        echo "✓ catch2:  v$(pkg-config --modversion catch2-with-main)"
    fi

    # ── Ripes (RISC-V simulator) ──────────────────────────────────────────────
    echo ""
    echo "--- Ripes (RISC-V simulator) ---"
    if [[ -n "${RIPES_PATH:-}" ]]; then
        echo "✓ RIPES_PATH=${RIPES_PATH} (using env var)"
    elif command -v ripes >/dev/null 2>&1; then
        echo "✓ ripes found in PATH: $(which ripes)"
    elif [[ -x "{{ project_directory }}/ripes/linux/Ripes.AppImage" ]]; then
        echo "✓ Bundled binary: ripes/linux/Ripes.AppImage"
        echo "  (lq will use it automatically - no further action needed)"
    else
        echo "  Ripes is not packaged for most distros."
        echo "  Download the AppImage from: https://github.com/mortbopet/Ripes/releases"
        echo "  Then set: export RIPES_PATH=/path/to/Ripes.AppImage"
    fi

    echo ""
    echo "✓ Installation complete. Run 'just check-deps' to verify."

[windows]
setup:
    #!/usr/bin/env pwsh
    $ErrorActionPreference = "Stop"

    # ── Winget check ──────────────────────────────────────────────────────────
    $hasWinget  = [bool](Get-Command winget  -ErrorAction SilentlyContinue)
    $hasChoco   = [bool](Get-Command choco   -ErrorAction SilentlyContinue)
    if (-not $hasWinget -and -not $hasChoco) {
        Write-Host "✗ Neither winget nor choco found."
        Write-Host "  Install winget (App Installer from Microsoft Store) or Chocolatey, then re-run."
        exit 1
    }
    function Pkg-Install($wingetId, $chocoId) {
        if ($hasWinget) { winget install --accept-source-agreements --accept-package-agreements -e --id $wingetId }
        elseif ($hasChoco) { choco install $chocoId -y }
    }

    # ── Rust (via rustup) ─────────────────────────────────────────────────────
    if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
        Write-Host "Installing Rust via rustup..."
        Pkg-Install "Rustlang.Rustup" "rustup.install"
        Write-Host "  → Restart your terminal after install, then run 'just setup' again."
    } else {
        Write-Host "✓ rustup: $(rustup --version 2>&1)"
        rustup component add rustfmt clippy 2>$null
        Write-Host "✓ rustc:   $(rustc --version)"
        Write-Host "✓ rustfmt: $(rustfmt --version)"
        Write-Host "✓ clippy:  $(cargo clippy --version)"
    }

    # ── Python ────────────────────────────────────────────────────────────────
    if (-not (Get-Command python3 -ErrorAction SilentlyContinue) -and `
        -not (Get-Command python  -ErrorAction SilentlyContinue)) {
        Write-Host "Installing Python..."
        Pkg-Install "Python.Python.3.12" "python"
    } else {
        $pyCmd = if (Get-Command python3 -ErrorAction SilentlyContinue) { "python3" } else { "python" }
        Write-Host "✓ python:  $(& $pyCmd --version)"
    }
    $pyCmd = if (Get-Command python3 -ErrorAction SilentlyContinue) { "python3" } else { "python" }
    & $pyCmd -m pytest --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Installing pytest..."
        & $pyCmd -m pip install --quiet pytest
    } else {
        Write-Host "✓ pytest:  installed"
    }

    # ── Go ────────────────────────────────────────────────────────────────────
    if (-not (Get-Command go -ErrorAction SilentlyContinue)) {
        Write-Host "Installing Go..."
        Pkg-Install "GoLang.Go" "golang"
    } else {
        Write-Host "✓ go:      $(go version)"
    }

    # ── C++ (MSYS2 MinGW toolchain + Catch2) ─────────────────────────────────
    #
    # On Windows the recommended C++ toolchain for lq exercises is MSYS2 with
    # the UCRT MinGW environment.  MSYS2 provides g++, pkg-config, and Catch2
    # in a single coherent package set.
    #
    #   1. Install MSYS2        : winget install MSYS2.MSYS2
    #   2. Open "MSYS2 UCRT64"  terminal and run:
    #        pacman -S mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-catch2 mingw-w64-ucrt-x86_64-pkg-config
    #   3. Add C:\msys64\ucrt64\bin to your system PATH.
    #
    if (-not (Get-Command g++ -ErrorAction SilentlyContinue)) {
        Write-Host ""
        Write-Host "--- C++ toolchain (MSYS2) ---"
        if (-not (Test-Path "C:\msys64\usr\bin\bash.exe")) {
            Write-Host "Installing MSYS2..."
            Pkg-Install "MSYS2.MSYS2" "msys2"
            Write-Host "  → After install, open 'MSYS2 UCRT64' and run:"
        } else {
            Write-Host "  MSYS2 is installed but g++ is not on PATH."
            Write-Host "  Open 'MSYS2 UCRT64' and run:"
        }
        Write-Host "    pacman -S mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-catch2 mingw-w64-ucrt-x86_64-pkg-config"
        Write-Host "  Then add C:\msys64\ucrt64\bin to your system PATH."
    } else {
        Write-Host "✓ g++:     $(g++ --version 2>&1 | Select-Object -First 1)"
        $hasPkg = [bool](Get-Command pkg-config -ErrorAction SilentlyContinue)
        if ($hasPkg) {
            pkg-config --exists catch2-with-main 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "✓ catch2:  v$(pkg-config --modversion catch2-with-main)"
            } else {
                Write-Host "⚠ catch2:  not found — install via MSYS2: pacman -S mingw-w64-ucrt-x86_64-catch2"
            }
        } else {
            Write-Host "⚠ pkg-config not found — install via MSYS2: pacman -S mingw-w64-ucrt-x86_64-pkg-config"
        }
    }

    # ── Ripes (RISC-V simulator) ──────────────────────────────────────────────
    Write-Host ""
    Write-Host "--- Ripes (RISC-V simulator) ---"
    if ($env:RIPES_PATH) {
        Write-Host "✓ RIPES_PATH=$($env:RIPES_PATH) (using env var)"
    } elseif (Get-Command ripes -ErrorAction SilentlyContinue) {
        Write-Host "✓ ripes found in PATH"
    } elseif (Test-Path "{{ project_directory }}\ripes\win\Ripes.exe") {
        Write-Host "✓ Bundled binary: ripes\win\Ripes.exe"
    } else {
        Write-Host "  Ripes is not available via winget/choco."
        Write-Host "  Download from: https://github.com/mortbopet/Ripes/releases"
        Write-Host '  Then set: $env:RIPES_PATH = "C:\path\to\Ripes.exe"'
    }

    Write-Host ""
    Write-Host "✓ Setup complete. Run 'just check-deps' to verify."

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
run-test args=args:
    cargo run -- --repo {{ test_repo }} {{args}}

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
