# Security Policy

GitVerse takes the security, privacy, and integrity of your source code and development environment seriously. Because GitVerse operates directly on local repositories and interacts with cloud provider credentials, our architectural security posture is built on strict local-first isolation.

---

## 1. Supported Versions

Only the latest stable release line receives security updates and vulnerability fixes.

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

---

## 2. Core Security Architecture

GitVerse adheres to strict security standards codified in our engineering constitution (`AGENTS.md`):

### 2.1. Local-First & Zero Telemetry
- GitVerse does not transmit your repository files, commit contents, diffs, branch names, or commit graphs to any third-party or proprietary GitVerse cloud server.
- All Git operations run locally using your machine's native `git` CLI executable.
- Network traffic is strictly limited to:
  - Explicit provider actions you configure (e.g., GitHub and GitLab API endpoints for Pull Requests, Merge Requests, and CI status checks).
  - Explicit or background application update checks querying official GitHub Releases endpoints (configured via `tauri.conf.json` updater endpoints; update checks transmit no repository code or personal telemetry and can be configured in app settings).

### 2.2. Zero Shell Interpolation Standard
- All Git CLI process execution is performed via direct, structured argument arrays (`std::process::Command` / `tokio::process::Command`).
- Shell string interpolation (e.g., `sh -c "git checkout $BRANCH"`) is strictly forbidden across the codebase, preventing command injection and shell escape attacks.

### 2.3. Secure Credential Storage
- Personal Access Tokens (PATs) and OAuth secrets for GitHub and GitLab are never stored in plaintext SQLite databases, local storage, or application config files.
- Credentials are encrypted and persisted using native operating system keyrings:
  - **Linux**: Secret Service API (via `libsecret` / FreeDesktop D-Bus)
  - **macOS**: Apple Keychain Services
  - **Windows**: Windows Credential Manager
- If a secure keyring daemon is unavailable, memory-only session storage is used as a secure transient fallback.

### 2.4. Automatic Secret Redaction in Diagnostics
- The application diagnostic logger automatically intercepts and redacts sensitive data before writing to disk or exporting bundles:
  - Bearer tokens and Personal Access Tokens (`ghp_...`, `glpat-...`).
  - Embedded HTTP Basic Authentication credentials in remote URLs (`https://user:password@host/repo.git`).
  - SSH private keys and passphrase payloads.

---

## 3. Reporting a Security Vulnerability

If you discover a security vulnerability or potential attack vector in GitVerse:

1. **Do not open a public GitHub issue**.
2. Report the vulnerability privately via GitHub Security Advisories or by emailing:
   - **Email**: `security@gitverse.app` (or contact repository maintainers directly).
3. Please include:
   - A detailed description of the vulnerability and attack scenario.
   - Operating system and Git CLI version.
   - Minimal reproduction steps, proof-of-concept scripts, or sample commands.
4. You will receive an initial response within 48 hours acknowledging receipt of your disclosure.
5. We will coordinate a fix, verification, and a responsible public advisory release.
