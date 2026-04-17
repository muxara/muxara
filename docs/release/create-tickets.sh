#!/usr/bin/env bash
#
# create-tickets.sh — Bulk-create GitHub issues, labels, and milestone for
# the Muxara v0.1.0 open-source release.
#
# Prerequisites:
#   - gh CLI installed and authenticated (https://cli.github.com)
#   - Target repo exists (e.g., muxara/muxara)
#
# Usage:
#   ./create-tickets.sh                        # Create everything
#   ./create-tickets.sh --dry-run              # Print commands without executing
#   ./create-tickets.sh --repo muxara/muxara   # Specify repo explicitly
#
set -euo pipefail

# --- Configuration -----------------------------------------------------------

REPO=""
DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run)  DRY_RUN=true; shift ;;
    --repo)     REPO="$2"; shift 2 ;;
    *)          echo "Unknown option: $1"; exit 1 ;;
  esac
done

if [[ -z "$REPO" ]]; then
  # Auto-detect from current git remote
  REPO=$(gh repo view --json nameWithOwner -q '.nameWithOwner' 2>/dev/null || true)
  if [[ -z "$REPO" ]]; then
    echo "Error: Could not detect repo. Use --repo OWNER/NAME"
    exit 1
  fi
fi

echo "Target repo: $REPO"
echo "Dry run: $DRY_RUN"
echo ""

# --- Helper -------------------------------------------------------------------

run() {
  if $DRY_RUN; then
    echo "[dry-run] $*"
  else
    "$@"
  fi
}

# Track created issue numbers for dependency references
declare -A ISSUES

create_issue() {
  local number="$1"
  local title="$2"
  local labels="$3"
  local body="$4"

  echo "Creating issue #${number}: ${title}"

  if $DRY_RUN; then
    echo "[dry-run] gh issue create --repo $REPO --title \"$title\" --label \"$labels\" --milestone \"v0.1.0 Public Release\" --body \"...\""
    ISSUES[$number]="(dry-run-${number})"
  else
    local result
    result=$(gh issue create \
      --repo "$REPO" \
      --title "$title" \
      --label "$labels" \
      --milestone "v0.1.0 Public Release" \
      --body "$body" 2>&1)
    local issue_url
    issue_url=$(echo "$result" | grep -o 'https://.*' | head -1)
    local issue_num
    issue_num=$(echo "$issue_url" | grep -o '[0-9]*$')
    ISSUES[$number]="$issue_num"
    echo "  -> Created: $issue_url"
  fi
}

# --- Labels -------------------------------------------------------------------

echo "=== Creating Labels ==="

declare -A LABELS=(
  ["metadata"]="#0E8A16:Package metadata and repo hygiene"
  ["documentation"]="#0075CA:Documentation and guides"
  ["community"]="#D876E3:Community health, templates, and contributor experience"
  ["ci/cd"]="#F9D0C4:Build, test, and release automation"
  ["distribution"]="#FEF2C0:Packaging, signing, and distribution channels"
  ["release"]="#B60205:Release execution and coordination"
)

for label in "${!LABELS[@]}"; do
  IFS=':' read -r color description <<< "${LABELS[$label]}"
  echo "Creating label: $label"
  run gh label create "$label" \
    --repo "$REPO" \
    --color "${color#\#}" \
    --description "$description" \
    --force
done

echo ""

# --- Milestone ----------------------------------------------------------------

echo "=== Creating Milestone ==="

run gh api \
  --method POST \
  "repos/${REPO}/milestones" \
  -f title="v0.1.0 Public Release" \
  -f description="All work required to publish Muxara as a professional open-source project" \
  -f state="open" \
  2>/dev/null || echo "Milestone may already exist, continuing..."

echo ""

# --- Issues -------------------------------------------------------------------

echo "=== Creating Issues ==="

# --- Metadata & Hygiene ---

create_issue 1 \
  "Add MIT LICENSE file" \
  "metadata" \
  "$(cat <<'BODY'
Add an MIT license file to the repository root. This is a legal prerequisite for open-source publication.

## Acceptance Criteria

- [ ] `LICENSE` file exists at repo root
- [ ] Uses standard MIT license text
- [ ] Copyright line: `Copyright (c) 2026 Muxara Contributors`
- [ ] `license` field added to `package.json`: `"MIT"`
- [ ] `license` field added to `src-tauri/Cargo.toml`: `"MIT"`
BODY
)"

create_issue 2 \
  "Update package metadata for public release" \
  "metadata" \
  "$(cat <<'BODY'
Update metadata fields across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` for public release.

**Depends on:** #1

## Acceptance Criteria

**package.json:**
- [ ] Remove `private: true`
- [ ] Add `description`: "Desktop control plane for managing parallel Claude Code sessions"
- [ ] Add `license`: "MIT"
- [ ] Add `repository`: `{ "type": "git", "url": "https://github.com/muxara/muxara.git" }`
- [ ] Add `author` and `keywords`

**src-tauri/Cargo.toml:**
- [ ] Add `license = "MIT"`, `repository`, `homepage`

**src-tauri/tauri.conf.json:**
- [ ] Set `bundle.macOS.minimumSystemVersion` to `"12.0"`
- [ ] Configure DMG layout
- [ ] Enable hardened runtime
BODY
)"

create_issue 3 \
  "Clean up .gitignore for public repo" \
  "metadata" \
  "$(cat <<'BODY'
Ensure `.gitignore` covers editor files, environment secrets, and build artifacts.

## Acceptance Criteria

- [ ] `.env*` patterns added
- [ ] `.vscode/` and `.idea/` added
- [ ] `*.swp`, `*.swo` added
- [ ] Existing entries preserved
BODY
)"

# --- Documentation ---

create_issue 4 \
  "Write README.md with hero section, screenshots, and install guide" \
  "documentation" \
  "$(cat <<'BODY'
Create a professional README that serves as the project's landing page.

**Depends on:** #1, #2

## Structure

1. Hero section (logo, tagline, badges)
2. Screenshot/GIF placeholder
3. What is Muxara? (problem/solution)
4. Features (bulleted highlights)
5. Requirements (macOS 12+, tmux, iTerm2, Claude Code)
6. Installation (Homebrew, DMG, source)
7. Quick Start
8. Configuration (link to docs)
9. Contributing (link to CONTRIBUTING.md)
10. License

## Acceptance Criteria

- [ ] All 10 sections present
- [ ] Badge row: build status, release, license, platform
- [ ] Screenshot placeholder with TODO
- [ ] Three installation methods documented
- [ ] All internal links work
BODY
)"

create_issue 5 \
  "Write CONTRIBUTING.md with dev setup and PR process" \
  "documentation,community" \
  "$(cat <<'BODY'
Create a contributor guide that makes it easy for newcomers to go from clone to submitted PR.

## Structure

1. Welcome and project philosophy
2. Prerequisites (Node 20+, Rust stable, tmux, iTerm2, macOS)
3. Development setup (step-by-step)
4. Project structure overview
5. Running tests
6. Code style (cargo fmt, cargo clippy)
7. How to submit changes (fork, branch, PR)
8. What makes a good PR
9. Issue triage and labels
10. Getting help

## Acceptance Criteria

- [ ] Step-by-step dev setup a newcomer can follow
- [ ] `npm run tauri dev` documented as main command
- [ ] Test commands documented
- [ ] PR process clear
- [ ] Commit conventions documented
- [ ] Links to CODE_OF_CONDUCT.md
- [ ] Welcoming, encouraging tone
BODY
)"

create_issue 6 \
  "Add CODE_OF_CONDUCT.md (Contributor Covenant v2.1)" \
  "community" \
  "$(cat <<'BODY'
Adopt the Contributor Covenant v2.1 as the project's code of conduct.

## Acceptance Criteria

- [ ] `CODE_OF_CONDUCT.md` at repo root
- [ ] Contributor Covenant v2.1 text
- [ ] Contact method for reporting violations
- [ ] Enforcement guidelines present
BODY
)"

create_issue 7 \
  "Add SECURITY.md with vulnerability reporting process" \
  "community" \
  "$(cat <<'BODY'
Create a security policy for responsible vulnerability disclosure.

## Acceptance Criteria

- [ ] `SECURITY.md` at repo root
- [ ] Clear reporting instructions (email or GitHub Security Advisories)
- [ ] Scope defined (Muxara only, not tmux/iTerm2/Claude Code)
- [ ] Response timeline stated (e.g., 48-hour acknowledgment)
- [ ] Supported versions table
BODY
)"

create_issue 8 \
  "Create CHANGELOG.md seeded from commit history" \
  "documentation" \
  "$(cat <<'BODY'
Create a changelog in [Keep a Changelog](https://keepachangelog.com/) format, seeded with all current features as the v0.1.0 entry.

## Acceptance Criteria

- [ ] Follows Keep a Changelog format
- [ ] `[Unreleased]` section at top
- [ ] `[0.1.0]` section with all current features grouped by category
- [ ] Comparison links at bottom
BODY
)"

create_issue 9 \
  "Add spike/README.md documenting the calibration process" \
  "documentation" \
  "$(cat <<'BODY'
Document the `spike/` directory as a calibration dataset for the session status classifier. The spike contains real captured terminal output used to derive the classifier's regex patterns. If Claude Code's output format changes, this process needs to be repeated.

## Contents to Document

- Purpose: Phase 0 research on tmux capture, state classification, end-to-end validation
- Directory contents: `findings.md`, `capture.sh`, `fixtures/` (real captures by state), `src/` (prototype classifier + tests)
- Recalibration process: run captures, compare with fixtures, update classifier patterns
- Why retained: classifier patterns are derived from these captures

## Acceptance Criteria

- [ ] `spike/README.md` exists
- [ ] Explains purpose of the spike directory
- [ ] Documents all contents
- [ ] Step-by-step recalibration process
- [ ] Links to production classifier (`src-tauri/src/tmux/classifier.rs`)
- [ ] Makes clear this is intentionally retained
BODY
)"

# --- Community & GitHub Config ---

create_issue 10 \
  "Create GitHub issue templates (bug report, feature request)" \
  "community" \
  "$(cat <<'BODY'
Add YAML-based issue templates for structured bug reports and feature requests.

## Templates

**Bug Report** (`bug_report.yml`):
- Muxara version, macOS version, tmux version, iTerm2 version
- Steps to reproduce, expected/actual behavior
- Terminal output/logs, screenshot (optional)

**Feature Request** (`feature_request.yml`):
- Problem description, proposed solution, alternatives

**Config** (`config.yml`):
- Blank issue option, link to Discussions

## Acceptance Criteria

- [ ] `.github/ISSUE_TEMPLATE/bug_report.yml` exists
- [ ] `.github/ISSUE_TEMPLATE/feature_request.yml` exists
- [ ] `.github/ISSUE_TEMPLATE/config.yml` exists
- [ ] Templates render correctly on GitHub
BODY
)"

create_issue 11 \
  "Create PR template with checklist" \
  "community" \
  "$(cat <<'BODY'
Add a pull request template that prompts contributors to describe changes and confirm conventions.

## Acceptance Criteria

- [ ] `.github/PULL_REQUEST_TEMPLATE.md` exists
- [ ] Sections: Summary, Related Issues, Test Plan, Checklist
- [ ] Checklist: tests pass, docs updated, CHANGELOG updated, no secrets
- [ ] Concise enough for repeat contributors
BODY
)"

create_issue 12 \
  "Configure GitHub repo settings (branch protection, topics, discussions)" \
  "community" \
  "$(cat <<'BODY'
Configure the GitHub repository for professional open-source maintenance. Manual task via GitHub UI or `gh` CLI.

**Depends on:** #13 (CI must exist for required status checks)

## Settings

- [ ] Description: "Desktop control plane for managing parallel Claude Code sessions"
- [ ] Topics: claude-code, tmux, developer-tools, tauri, macos, session-manager
- [ ] Branch protection on `main` (require PR review, status checks, no force push)
- [ ] Enable Discussions
- [ ] Disable wiki
- [ ] Enable auto-delete head branches
BODY
)"

# --- CI/CD ---

create_issue 13 \
  "Add CI workflow for build, test, and lint" \
  "ci/cd" \
  "$(cat <<'BODY'
Add a GitHub Actions CI workflow that runs on every push to `main` and all PRs.

## Workflow: `.github/workflows/ci.yml`

- Runner: `macos-latest`
- Steps: checkout, Node 20, Rust stable, `npm ci`, `npm run build`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`
- Cache: Cargo registry + target, npm cache

## Acceptance Criteria

- [ ] `.github/workflows/ci.yml` exists
- [ ] Triggers on push to `main` and all PRs
- [ ] Runs on macOS
- [ ] Frontend builds, Rust fmt/clippy/test all run
- [ ] Dependencies cached
BODY
)"

create_issue 14 \
  "Add release workflow with Tauri DMG builds" \
  "ci/cd,distribution" \
  "$(cat <<'BODY'
Add a GitHub Actions workflow that builds signed macOS DMGs on version tag push using `tauri-apps/tauri-action`.

**Depends on:** #13, #17

## Workflow: `.github/workflows/release.yml`

- Trigger: push tag `v*`
- Matrix: aarch64-apple-darwin (Apple Silicon), x86_64-apple-darwin (Intel)
- Uses `tauri-apps/tauri-action@v0`
- Code signing + notarization via secrets
- Creates draft GitHub Release with DMGs attached

## Acceptance Criteria

- [ ] `.github/workflows/release.yml` exists
- [ ] Triggers on `v*` tags
- [ ] Builds for both architectures
- [ ] Code signing secrets referenced
- [ ] Creates draft release with DMGs
BODY
)"

create_issue 15 \
  "Add version sync check across package files" \
  "ci/cd" \
  "$(cat <<'BODY'
Add a CI step that verifies version consistency across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.

**Depends on:** #13

## Acceptance Criteria

- [ ] Version sync check runs in CI
- [ ] Fails build on mismatch
- [ ] Clear error message showing which files differ
- [ ] Optional: `scripts/sync-version.sh` helper to bump all three
BODY
)"

# --- Apple Developer Setup ---

create_issue 16 \
  "Document Apple Developer certificate and API key setup" \
  "distribution" \
  "$(cat <<'BODY'
Create a guide for generating the Apple Developer certificates and App Store Connect API key needed for code signing and notarization.

## Guide: `docs/apple-signing-guide.md`

1. Developer ID Application certificate (CSR, download, export .p12)
2. App Store Connect API key (.p8 download, Key ID, Issuer ID)
3. Base64 encoding for GitHub Secrets

## Acceptance Criteria

- [ ] `docs/apple-signing-guide.md` exists
- [ ] Step-by-step certificate generation
- [ ] API key generation documented
- [ ] Base64 encoding instructions
- [ ] All secret names listed
- [ ] Warning about one-time .p8 download
BODY
)"

create_issue 17 \
  "Configure GitHub Secrets for Apple code signing" \
  "distribution,ci/cd" \
  "$(cat <<'BODY'
Add Apple signing credentials as GitHub repository secrets.

**Depends on:** #16

## Secrets

| Secret | Value |
|--------|-------|
| `APPLE_CERTIFICATE` | Base64-encoded .p12 |
| `APPLE_CERTIFICATE_PASSWORD` | .p12 export password |
| `KEYCHAIN_PASSWORD` | Random CI keychain password |
| `APPLE_API_ISSUER` | Issuer ID |
| `APPLE_API_KEY` | Key ID |
| `APPLE_API_KEY_PATH` | .p8 file contents |

## Acceptance Criteria

- [ ] All 6 secrets configured
- [ ] Release workflow references them correctly
- [ ] Test with `v0.1.0-rc.1` tag
- [ ] DMG is signed and notarized
BODY
)"

# --- Distribution ---

create_issue 18 \
  "Create homebrew-muxara tap with cask formula" \
  "distribution" \
  "$(cat <<'BODY'
Create `muxara/homebrew-muxara` repo with a Homebrew cask formula pointing to GitHub Release DMGs.

**Depends on:** #14

## Cask Formula

- Architecture-aware URLs (ARM/Intel)
- SHA256 checksums
- `depends_on macos: ">= :monterey"`
- `app "Muxara.app"`
- `zap` stanza for preferences cleanup

## Acceptance Criteria

- [ ] `muxara/homebrew-muxara` repo exists
- [ ] `Casks/muxara.rb` formula present
- [ ] `brew tap muxara/muxara` succeeds
- [ ] `brew install --cask muxara` works
- [ ] Uninstall and zap work correctly
BODY
)"

create_issue 19 \
  "Add Homebrew tap update automation" \
  "distribution,ci/cd" \
  "$(cat <<'BODY'
Automate cask formula updates when new releases are published.

**Depends on:** #18

## Acceptance Criteria

- [ ] Publishing a release triggers formula update
- [ ] Version and SHA256 updated automatically
- [ ] Formula update is auto-merged or creates a PR
- [ ] Tap remains functional after updates
BODY
)"

# --- Release ---

create_issue 20 \
  "Execute v0.1.0 release" \
  "release" \
  "$(cat <<'BODY'
Final release coordination. Execute the full checklist to publish Muxara v0.1.0.

**Depends on:** All previous tickets (#1-#19)

## Checklist

- [ ] All 19 preceding tickets completed
- [ ] Repo transferred/created under `muxara` org
- [ ] All URLs updated to reflect new org
- [ ] GitHub Secrets configured
- [ ] CHANGELOG.md v0.1.0 date set
- [ ] Version is `0.1.0` across all files
- [ ] `git tag v0.1.0 && git push origin v0.1.0`
- [ ] Release workflow completes successfully
- [ ] Both DMGs tested (Apple Silicon + Intel)
- [ ] Code signature verified: `codesign --verify --deep Muxara.app`
- [ ] Notarization verified: `spctl --assess --type execute Muxara.app`
- [ ] Release notes added and published
- [ ] Homebrew formula updated
- [ ] `brew install --cask muxara` tested
- [ ] App launches and basic functionality works
- [ ] README badges resolve correctly
BODY
)"

echo ""
echo "=== Summary ==="
echo "Created labels: ${!LABELS[*]}"
echo "Created milestone: v0.1.0 Public Release"
echo "Created issues: 20"
echo ""
echo "Next steps:"
echo "  1. Create a GitHub Project board named 'Open-Source Launch'"
echo "     gh project create --owner muxara --title 'Open-Source Launch' --body 'Tracking all work for the v0.1.0 public release'"
echo ""
echo "  2. Add all issues to the project board:"
echo "     for i in \$(gh issue list --repo $REPO --milestone 'v0.1.0 Public Release' --json number -q '.[].number'); do"
echo "       gh project item-add PROJECT_NUMBER --owner muxara --url https://github.com/$REPO/issues/\$i"
echo "     done"
echo ""
echo "Done!"
