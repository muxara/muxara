# Muxara v0.1.0 Public Release - Tickets

> **Milestone:** v0.1.0 Public Release
> **Project Board:** Open-Source Launch
> **Target org:** github.com/muxara/muxara

## Labels

| Label | Color | Description |
|-------|-------|-------------|
| `metadata` | `#0E8A16` | Package metadata and repo hygiene |
| `documentation` | `#0075CA` | Documentation and guides |
| `community` | `#D876E3` | Community health, templates, and contributor experience |
| `ci/cd` | `#F9D0C4` | Build, test, and release automation |
| `distribution` | `#FEF2C0` | Packaging, signing, and distribution channels |
| `release` | `#B60205` | Release execution and coordination |

---

## Metadata & Hygiene

### #1 - Add MIT LICENSE file

**Labels:** `metadata`
**Depends on:** None

**Description:**
Add an MIT license file to the repository root. This is a legal prerequisite for open-source publication — without it, the code is technically "all rights reserved" even if the repo is public.

**Acceptance Criteria:**
- [ ] `LICENSE` file exists at repo root
- [ ] Uses standard MIT license text
- [ ] Copyright line reads: `Copyright (c) 2026 Muxara Contributors`
- [ ] `license` field in `package.json` reads `"MIT"`
- [ ] `license` field in `src-tauri/Cargo.toml` reads `"MIT"`

---

### #2 - Update package metadata for public release

**Labels:** `metadata`
**Depends on:** #1

**Description:**
Update metadata fields across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` so the project is properly described in registries, GitHub, and the built application.

**Acceptance Criteria:**

`package.json`:
- [ ] `private` field removed (or set to `false`)
- [ ] `description` field added: `"Desktop control plane for managing parallel Claude Code sessions"`
- [ ] `license` field: `"MIT"`
- [ ] `repository` field: `{ "type": "git", "url": "https://github.com/muxara/muxara.git" }`
- [ ] `author` field added
- [ ] `keywords` field: `["claude-code", "tmux", "developer-tools", "tauri", "session-manager"]`

`src-tauri/Cargo.toml`:
- [ ] `license = "MIT"`
- [ ] `repository = "https://github.com/muxara/muxara"`
- [ ] `homepage = "https://github.com/muxara/muxara"`

`src-tauri/tauri.conf.json`:
- [ ] `bundle.macOS.minimumSystemVersion` set to `"12.0"` (Monterey)
- [ ] DMG layout configured (`windowSize`, `appPosition`, `applicationFolderPosition`)
- [ ] Hardened runtime enabled for notarization compatibility

---

### #3 - Clean up .gitignore for public repo

**Labels:** `metadata`
**Depends on:** None

**Description:**
Ensure `.gitignore` covers common editor files, environment secrets, and build artifacts that shouldn't be in a public repo.

**Acceptance Criteria:**
- [ ] `.env*` patterns added (prevent accidental secret commits)
- [ ] `.vscode/` added (editor-specific settings)
- [ ] `.idea/` added (JetBrains editor settings)
- [ ] `*.swp`, `*.swo` added (vim swap files)
- [ ] Existing entries preserved
- [ ] No overly broad patterns that would exclude legitimate files

---

## Documentation

### #4 - Write README.md

**Labels:** `documentation`
**Depends on:** #1, #2

**Description:**
Create a professional README that serves as the project's landing page. This is the single most important file for first impressions and adoption. It should make someone want to try Muxara within 30 seconds of landing on the repo.

**Structure:**
1. Hero section with logo/icon, one-line tagline, badge row (build status, version, license, macOS)
2. Screenshot or animated GIF of the dashboard in action (placeholder initially, replace before release)
3. "What is Muxara?" - 3-sentence problem/solution pitch
4. Features - bulleted highlights with brief descriptions
5. Requirements - macOS 12+, tmux, iTerm2, Claude Code
6. Installation - Homebrew cask, manual DMG download, build from source
7. Quick Start - launch, click "+", select directory, see sessions
8. Configuration - brief settings overview, link to detailed docs
9. Contributing - link to CONTRIBUTING.md
10. License - MIT with link

**Acceptance Criteria:**
- [ ] README.md exists at repo root
- [ ] All 10 sections present
- [ ] Badge row includes: build status (GitHub Actions), latest release, license, platform
- [ ] Screenshot placeholder with clear TODO comment for replacement
- [ ] Installation section covers all three methods (Homebrew, DMG, source)
- [ ] Requirements section lists minimum versions
- [ ] All internal links work (CONTRIBUTING.md, docs/architecture.md, etc.)
- [ ] No broken badge URLs (update after CI is set up)

---

### #5 - Write CONTRIBUTING.md

**Labels:** `documentation`, `community`
**Depends on:** None

**Description:**
Create a contributor guide that makes it as easy as possible for someone to go from "I want to help" to "I submitted a PR." Lower the barrier to entry as much as possible. This should be welcoming, thorough, and respectful of contributors' time.

**Structure:**
1. Welcome message and project philosophy
2. Prerequisites (Node 20+, Rust stable, tmux, iTerm2, macOS)
3. Development setup (step-by-step from clone to running app)
4. Project structure overview (brief, links to architecture.md)
5. Running tests (`cd src-tauri && cargo test`)
6. Code style and conventions (Rust: `cargo fmt`, `cargo clippy`; TypeScript: existing linting)
7. How to submit changes (fork, branch naming, commit messages, PR process)
8. What makes a good PR (small, focused, docs updated, CHANGELOG entry)
9. Issue triage and labels explained
10. Getting help (where to ask questions - discussions, issues)

**Acceptance Criteria:**
- [ ] `CONTRIBUTING.md` exists at repo root
- [ ] Step-by-step dev setup that a newcomer can follow
- [ ] `npm run tauri dev` explained as the main dev command
- [ ] Test commands documented
- [ ] PR process clear: fork → branch → changes → PR → review
- [ ] Commit message conventions documented
- [ ] Links to CODE_OF_CONDUCT.md
- [ ] Tone is welcoming and encouraging

---

### #6 - Add CODE_OF_CONDUCT.md

**Labels:** `community`
**Depends on:** None

**Description:**
Adopt the Contributor Covenant v2.1, the industry standard code of conduct for open-source projects. This signals that the project is a safe, inclusive space for all contributors.

**Acceptance Criteria:**
- [ ] `CODE_OF_CONDUCT.md` exists at repo root
- [ ] Uses Contributor Covenant v2.1 text
- [ ] Contact method specified for reporting violations (email address or GitHub mechanism)
- [ ] Enforcement guidelines section present

---

### #7 - Add SECURITY.md

**Labels:** `community`
**Depends on:** None

**Description:**
Create a security policy that tells researchers how to responsibly disclose vulnerabilities. This prevents security issues from being filed as public issues.

**Acceptance Criteria:**
- [ ] `SECURITY.md` exists at repo root (or `.github/SECURITY.md`)
- [ ] Clear instructions for reporting vulnerabilities (email or GitHub Security Advisories)
- [ ] Scope defined (Muxara itself, not tmux/iTerm2/Claude Code)
- [ ] Expected response timeline stated (e.g., "acknowledge within 48 hours")
- [ ] Supported versions table (only v0.1.x initially)

---

### #8 - Create CHANGELOG.md seeded from commit history

**Labels:** `documentation`
**Depends on:** None

**Description:**
Create a changelog following the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. Seed the v0.1.0 entry from the existing commit history, grouped by feature area. Include an `[Unreleased]` section at the top for ongoing work.

**Acceptance Criteria:**
- [ ] `CHANGELOG.md` exists at repo root
- [ ] Follows Keep a Changelog format with link to spec
- [ ] States adherence to Semantic Versioning with link
- [ ] `[Unreleased]` section at top
- [ ] `[0.1.0]` section with date, covering all current features grouped by category:
  - Added: session dashboard, status classification, iTerm2 switching, keyboard navigation, new session creation, kill/rename actions, settings panel, worktree isolation, configurable bootstrap command, ANSI color rendering
- [ ] Comparison links at bottom (`[Unreleased]` vs `v0.1.0`, `[0.1.0]` vs initial commit)

---

### #9 - Add spike/README.md documenting the calibration process

**Labels:** `documentation`
**Depends on:** None

**Description:**
The `spike/` directory contains the original research used to understand Claude Code's terminal output formats and build the session status classifier. This is a calibration dataset — if Claude Code's output format drifts in the future, the same process can be reused to recalibrate. Document this clearly so future contributors (or the maintainer) know what the spike is, why it's kept, and how to reuse it.

**Contents to document:**
- Purpose: research artifacts from Phase 0 spikes on tmux capture, state classification, and end-to-end validation
- What's in the directory: `findings.md` (spike results), `capture.sh` (capture script), `fixtures/` (real captured terminal output organized by state), `src/` (spike TypeScript code with classifier prototype and test suite)
- How to use for recalibration: run `capture.sh` against live sessions, compare with existing fixtures, update classifier patterns in `src-tauri/src/tmux/classifier.rs`
- Why it's retained: the classifier's regex patterns are derived from these real-world captures; if Claude Code changes its output format, this process needs to be repeated

**Acceptance Criteria:**
- [ ] `spike/README.md` exists
- [ ] Explains the purpose of the spike directory
- [ ] Documents the contents (findings, fixtures, scripts, prototype code)
- [ ] Describes the recalibration process step-by-step
- [ ] Links to the production classifier (`src-tauri/src/tmux/classifier.rs`)
- [ ] Makes it clear this directory is intentionally retained, not leftover debris

---

## Community & GitHub Config

### #10 - Create GitHub issue templates

**Labels:** `community`
**Depends on:** None

**Description:**
Add YAML-based issue templates that guide reporters to provide structured, actionable information. This reduces back-and-forth and makes triage faster.

**Templates:**

**Bug Report** (`bug_report.yml`):
- Fields: Muxara version, macOS version, tmux version, iTerm2 version, steps to reproduce, expected behavior, actual behavior, terminal output / logs, screenshot (optional)
- Use dropdowns for common states (NeedsInput, Working, Idle, Errored)

**Feature Request** (`feature_request.yml`):
- Fields: problem description ("I'm frustrated when..."), proposed solution, alternatives considered, additional context

**Config** (`config.yml`):
- Blank issue option for anything that doesn't fit templates
- Link to Discussions (if enabled) for questions

**Acceptance Criteria:**
- [ ] `.github/ISSUE_TEMPLATE/bug_report.yml` exists with structured form fields
- [ ] `.github/ISSUE_TEMPLATE/feature_request.yml` exists with structured form fields
- [ ] `.github/ISSUE_TEMPLATE/config.yml` exists with blank issue and links
- [ ] Templates render correctly on GitHub (test by navigating to /issues/new/choose)

---

### #11 - Create PR template

**Labels:** `community`
**Depends on:** None

**Description:**
Add a pull request template that prompts contributors to describe their changes, link related issues, and confirm they've followed the project's conventions.

**Acceptance Criteria:**
- [ ] `.github/PULL_REQUEST_TEMPLATE.md` exists
- [ ] Includes sections: Summary, Related Issues, Test Plan, Checklist
- [ ] Checklist includes: tests pass, docs updated if needed, CHANGELOG updated if user-facing, no sensitive data committed
- [ ] Template is concise enough not to annoy repeat contributors

---

### #12 - Configure GitHub repo settings

**Labels:** `community`
**Depends on:** #13 (CI must exist for branch protection)

**Description:**
Configure the GitHub repository for professional open-source maintenance. This is a manual task performed via GitHub UI or `gh` CLI after the org repo is created.

**Settings to configure:**
- Repository description: "Desktop control plane for managing parallel Claude Code sessions"
- Website: (GitHub releases page or future site)
- Topics: `claude-code`, `tmux`, `developer-tools`, `tauri`, `macos`, `session-manager`
- Branch protection on `main`:
  - Require PR reviews (1 reviewer)
  - Require status checks to pass (CI workflow)
  - Require branches to be up to date
  - No force pushes
- Enable Discussions (for Q&A and community support)
- Disable wiki (docs live in repo)
- Enable "Automatically delete head branches" after merge
- Set default branch to `main`

**Acceptance Criteria:**
- [ ] Description and topics set
- [ ] Branch protection rules active on `main`
- [ ] Discussions enabled
- [ ] Wiki disabled
- [ ] Auto-delete head branches enabled
- [ ] Settings documented in a checklist (since they're manual)

---

## CI/CD

### #13 - Add CI workflow for build, test, and lint

**Labels:** `ci/cd`
**Depends on:** None

**Description:**
Add a GitHub Actions workflow that runs on every push to `main` and every PR. This catches breakage early and gives contributors confidence that the test suite is the source of truth.

**Workflow: `.github/workflows/ci.yml`**
- Trigger: push to `main`, all PRs
- Runner: `macos-latest`
- Steps:
  1. Checkout code
  2. Setup Node 20
  3. Setup Rust stable
  4. `npm ci` (install frontend deps)
  5. `npm run build` (build frontend)
  6. `cargo fmt -- --check` (Rust formatting)
  7. `cargo clippy -- -D warnings` (Rust linting)
  8. `cargo test` (Rust tests)
- Cache: Cargo registry + target directory, npm cache

**Acceptance Criteria:**
- [ ] `.github/workflows/ci.yml` exists
- [ ] Triggers on push to `main` and all PRs
- [ ] Runs on macOS (required for Tauri build)
- [ ] Frontend builds successfully
- [ ] `cargo fmt`, `cargo clippy`, `cargo test` all run
- [ ] Cargo and npm dependencies are cached
- [ ] Workflow badge URL documented for README

---

### #14 - Add release workflow with Tauri builds

**Labels:** `ci/cd`, `distribution`
**Depends on:** #13, #17

**Description:**
Add a GitHub Actions workflow that builds signed and notarized macOS DMGs when a version tag (`v*`) is pushed. Uses the official `tauri-apps/tauri-action` to build, sign, notarize, and upload artifacts to a GitHub Release.

**Workflow: `.github/workflows/release.yml`**
- Trigger: push tag matching `v*`
- Matrix builds:
  - `aarch64-apple-darwin` (Apple Silicon)
  - `x86_64-apple-darwin` (Intel)
- Steps per target:
  1. Checkout code
  2. Setup Node 20
  3. Setup Rust stable with target
  4. `npm ci`
  5. `tauri-apps/tauri-action@v0` with:
     - Code signing env vars from secrets
     - Notarization env vars from secrets
     - `tagName: v__VERSION__`
     - `releaseName: Muxara v__VERSION__`
     - `releaseDraft: true`
     - Target-specific `--target` arg
- Outputs: draft GitHub Release with `.dmg` files for both architectures

**Acceptance Criteria:**
- [ ] `.github/workflows/release.yml` exists
- [ ] Triggers on `v*` tags only
- [ ] Builds for both aarch64 and x86_64
- [ ] Uses `tauri-apps/tauri-action@v0`
- [ ] Code signing secrets referenced (commented out until #17 is done)
- [ ] Creates a draft release (not auto-published)
- [ ] Release name follows `Muxara v{version}` pattern
- [ ] Both `.dmg` files attached to the release

---

### #15 - Add version sync check

**Labels:** `ci/cd`
**Depends on:** #13

**Description:**
Add a CI step (or standalone script) that verifies the version string is consistent across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. A mismatch between these causes confusing build outputs.

**Acceptance Criteria:**
- [ ] Version sync check runs as part of CI
- [ ] Fails the build if any of the three version strings differ
- [ ] Error message clearly states which files are mismatched and what versions they contain
- [ ] Optionally: a `scripts/sync-version.sh` helper that bumps all three files at once

---

## Apple Developer Setup

### #16 - Generate Developer ID certificates and API key

**Labels:** `distribution`
**Depends on:** None (manual task)

**Description:**
Generate the Apple Developer certificates and App Store Connect API key needed for code signing and notarization. This is a manual process done through the Apple Developer portal.

**Steps documented in `docs/apple-signing-guide.md`:**

1. **Developer ID Application certificate:**
   - Log in to developer.apple.com > Certificates, Identifiers & Profiles
   - Create new certificate > "Developer ID Application"
   - Generate a Certificate Signing Request (CSR) from Keychain Access (Certificate Assistant > Request a Certificate from a Certificate Authority)
   - Upload CSR, download the `.cer` file
   - Double-click to import into Keychain Access
   - Right-click the certificate > Export as `.p12` with a strong password

2. **App Store Connect API key:**
   - Log in to App Store Connect > Users and Access > Integrations > App Store Connect API
   - Click "+" to generate a new key with "Developer" role
   - Download the `.p8` file immediately (only downloadable once!)
   - Note the **Key ID** and **Issuer ID** shown on the page

3. **Base64 encode the certificate:**
   ```bash
   base64 -i DeveloperIDApplication.p12 | pbcopy
   ```

**Acceptance Criteria:**
- [ ] `docs/apple-signing-guide.md` exists with step-by-step instructions
- [ ] Guide covers certificate generation, CSR creation, export to `.p12`
- [ ] Guide covers API key generation and `.p8` download
- [ ] Guide covers base64 encoding for GitHub Secrets
- [ ] All required secret names listed (matches what the release workflow expects)
- [ ] Warnings about one-time-download `.p8` file included

---

### #17 - Configure GitHub Secrets for code signing

**Labels:** `distribution`, `ci/cd`
**Depends on:** #16

**Description:**
Add the Apple Developer signing credentials as GitHub repository secrets so the release workflow can sign and notarize builds. This is a manual task done via GitHub UI.

**Secrets to create:**

| Secret Name | Value |
|-------------|-------|
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password used when exporting the `.p12` |
| `KEYCHAIN_PASSWORD` | Any random password (for the CI temporary keychain) |
| `APPLE_API_ISSUER` | Issuer ID from App Store Connect |
| `APPLE_API_KEY` | Key ID from App Store Connect |
| `APPLE_API_KEY_PATH` | Contents of the `.p8` file |

**Acceptance Criteria:**
- [ ] All 6 secrets configured on the repository
- [ ] Release workflow (`release.yml`) references all secrets correctly
- [ ] Code signing env vars uncommented in release workflow
- [ ] Test by pushing a `v0.1.0-rc.1` tag and verifying the DMG is signed
- [ ] Notarization succeeds (check with `spctl --assess --type execute Muxara.app`)

---

## Distribution

### #18 - Create homebrew-muxara tap with cask formula

**Labels:** `distribution`
**Depends on:** #14 (release workflow must produce DMGs)

**Description:**
Create a separate `muxara/homebrew-muxara` repository containing a Homebrew cask formula. This enables users to install Muxara with `brew install --cask muxara/muxara/muxara`.

**Cask formula should include:**
- Architecture-aware URLs pointing to GitHub Release DMGs (ARM vs Intel)
- SHA256 checksums for each DMG
- `depends_on macos: ">= :monterey"`
- `app "Muxara.app"`
- `zap` stanza to clean up preferences on uninstall (`~/Library/Application Support/com.muxara.app`)

**Acceptance Criteria:**
- [ ] `muxara/homebrew-muxara` repository exists
- [ ] `Casks/muxara.rb` formula exists with correct URLs and checksums
- [ ] `brew tap muxara/muxara` succeeds
- [ ] `brew install --cask muxara` downloads and installs the app
- [ ] `brew uninstall --cask muxara` removes the app cleanly
- [ ] `brew uninstall --cask --zap muxara` removes preferences too
- [ ] README in tap repo explains usage

---

### #19 - Add Homebrew tap update automation

**Labels:** `distribution`, `ci/cd`
**Depends on:** #18

**Description:**
Automate the Homebrew cask formula update when a new release is published. After each GitHub Release is published on `muxara/muxara`, a workflow should update the version and SHA256 checksums in the `homebrew-muxara` cask formula.

**Options:**
- GitHub Action in `muxara/muxara` triggered on `release: published` that dispatches to `homebrew-muxara`
- GitHub Action in `homebrew-muxara` triggered by `repository_dispatch`
- Script that downloads the release assets, computes SHA256, and updates the formula via PR

**Acceptance Criteria:**
- [ ] Publishing a release on `muxara/muxara` triggers a formula update
- [ ] Version and SHA256 checksums are updated automatically
- [ ] Formula update is either auto-merged or creates a PR for review
- [ ] The tap remains functional after automated updates

---

## Release

### #20 - Execute v0.1.0 release

**Labels:** `release`
**Depends on:** #1-#19 (all previous tickets)

**Description:**
The final release coordination ticket. Execute the full release checklist to publish Muxara v0.1.0.

**Checklist:**
1. [ ] Verify all 19 preceding tickets are completed
2. [ ] Transfer repo to `muxara` org (or create fresh repo under org and push)
3. [ ] Update all URLs in docs/config to reflect new org (github.com/muxara/muxara)
4. [ ] GitHub Secrets configured (#17)
5. [ ] Final `CHANGELOG.md` review — v0.1.0 date set to release date
6. [ ] Version is `0.1.0` across all three files
7. [ ] `git tag v0.1.0 && git push origin v0.1.0`
8. [ ] Wait for release workflow to complete
9. [ ] Download both DMGs from draft release, test on Apple Silicon and Intel
10. [ ] Verify code signature: `codesign --verify --deep Muxara.app`
11. [ ] Verify notarization: `spctl --assess --type execute Muxara.app`
12. [ ] Edit release notes (copy from CHANGELOG), publish release
13. [ ] Update Homebrew tap formula with release asset URLs and checksums
14. [ ] Test `brew tap muxara/muxara && brew install --cask muxara`
15. [ ] Verify app launches and basic functionality works after Homebrew install
16. [ ] Update README badge URLs to point to real CI/release
17. [ ] Announce release (social media, relevant communities)

**Acceptance Criteria:**
- [ ] v0.1.0 tag exists on `main`
- [ ] GitHub Release published with both DMGs attached
- [ ] Homebrew cask installs successfully
- [ ] App launches, creates sessions, and shows the dashboard
- [ ] All README badges resolve correctly

---

## Dependency Graph

```
#1 LICENSE ──────────┐
#2 Package metadata ─┤ (depends on #1)
#3 .gitignore        │
                     ├──→ #4 README (depends on #1, #2)
#5 CONTRIBUTING      │
#6 CODE_OF_CONDUCT   │
#7 SECURITY          │
#8 CHANGELOG         │
#9 spike/README      │
#10 Issue templates  │
#11 PR template      │
                     │
#13 CI workflow ─────┼──→ #12 Repo settings (depends on #13)
                     ├──→ #15 Version sync (depends on #13)
                     │
#16 Apple certs ─────┼──→ #17 GitHub Secrets (depends on #16)
                     │
#13 + #17 ───────────┼──→ #14 Release workflow (depends on #13, #17)
                     │
#14 ─────────────────┼──→ #18 Homebrew tap (depends on #14)
#18 ─────────────────┼──→ #19 Tap automation (depends on #18)
                     │
#1-#19 ──────────────┴──→ #20 Execute release (depends on all)
```

## Parallelization Notes

Many tickets can be worked on in parallel:
- **Fully independent (can start immediately):** #1, #3, #5, #6, #7, #8, #9, #10, #11, #13, #16
- **Blocked by one ticket:** #2 (by #1), #4 (by #1 & #2), #12 (by #13), #14 (by #13 & #17), #15 (by #13), #17 (by #16), #18 (by #14)
- **Blocked by chain:** #19 (by #18 ← #14 ← #13 & #17 ← #16)
- **Blocked by everything:** #20
