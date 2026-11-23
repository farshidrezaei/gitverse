# Portable Distribution & Installation Guide

GitVerse is engineered to support both installed packages and zero-installation portable execution across all major desktop operating systems.

---

## 1. Distribution Matrix

| Operating System | Package Type | Artifact Format | Portable Execution |
| :--- | :--- | :--- | :--- |
| **Linux (x86_64)** | Portable AppImage | `GitVerse_<version>_amd64.AppImage` | :white_check_mark: Yes (Run directly) |
| **Linux (Debian/Ubuntu)** | System Package | `gitverse_<version>_amd64.deb` | :x: No (System install) |
| **macOS (Universal)** | Disk Image | `GitVerse_<version>_universal.dmg` | :white_check_mark: Yes (Run or Drag-to-Applications) |
| **macOS (App Bundle)** | Application Archive | `GitVerse.app.tar.gz` | :white_check_mark: Yes (Standalone bundle) |
| **Windows (x64)** | Portable Executable | `GitVerse_<version>_x64-portable.exe` | :white_check_mark: Yes (Zero install) |
| **Windows (x64)** | Windows Installer | `GitVerse_<version>_x64-setup.msi` | :x: No (System install) |

---

## 2. Automated One-Line Installers (Recommended)

GitVerse provides state-aware, one-line installer scripts that automatically detect your operating system and architecture, check for existing installations, manage updates/reinstalls, configure desktop entries, and set up your system PATH.

### 2.1. Linux & macOS (Bash)

Install or update with a single terminal command:

```bash
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/gitverse/main/scripts/install.sh | bash
```

#### Smart Re-run & Version Awareness:
- **First run**: Installs the latest release to `~/.local/bin/gitverse` and creates an application launcher with official icons.
- **Already installed (up to date)**: Alerts you that GitVerse is current, offering a clean reinstall:
  ```text
  GitVerse is already installed and up to date (v1.0.0).
  Would you like to reinstall GitVerse v1.0.0? [y/N]:
  ```
- **Newer version available**: Prompts for a one-click automatic upgrade:
  ```text
  Update Available!
  Installed version: v1.0.0
  Latest version:    v1.1.0

  Would you like to update GitVerse to v1.1.0 now? [Y/n]:
  ```

#### CLI Options:
```bash
# Bypass prompts (CI / Non-interactive mode)
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/gitverse/main/scripts/install.sh | bash -s -- --force

# Install a specific tagged release
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/gitverse/main/scripts/install.sh | bash -s -- --version v1.0.0

# Force immediate reinstallation
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/gitverse/main/scripts/install.sh | bash -s -- --reinstall

# Uninstall GitVerse and clean up desktop entries & symlinks
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/gitverse/main/scripts/install.sh | bash -s -- --uninstall
```

---

### 2.2. Windows (PowerShell)

Install or update from PowerShell 5.1+ or PowerShell 7+:

```powershell
irm https://raw.githubusercontent.com/farshidrezaei/gitverse/main/scripts/install.ps1 | iex
```

#### Features:
- Silent NSIS or MSI installer execution without administrative prompts.
- Automatically places binary into `%LOCALAPPDATA%\Programs\GitVerse` and updates the user's `PATH`.
- Supports the same update/reinstall detection prompts as the Unix installer.
- Supports `-Force`, `-Version <tag>`, `-Reinstall`, and `-Uninstall` switches.

---

## 3. Manual Installation & Execution Instructions

### 3.1. Linux (AppImage)

The AppImage package requires no installation and can be executed directly from any folder or USB drive:

```bash
# 1. Make the downloaded AppImage executable
chmod +x GitVerse_1.0.0_amd64.AppImage

# 2. Launch GitVerse
./GitVerse_1.0.0_amd64.AppImage
```

> [!NOTE]
> Ensure FUSE (`libfuse2` on Ubuntu 22.04+) is present on your system to launch AppImage files.

### 3.2. Linux (Debian / Ubuntu Package)

```bash
sudo dpkg -i gitverse_1.0.0_amd64.deb
sudo apt-get install -f
```

### 3.3. macOS (Universal DMG)

1. Download `GitVerse_1.0.0_universal.dmg`.
2. Double-click the DMG to mount it.
3. Drag `GitVerse.app` into your `/Applications` directory (or run it directly from the volume).

### 3.4. Windows (Portable)

1. Download `GitVerse_1.0.0_x64-portable.exe`.
2. Move it to your preferred directory (e.g. `C:\Tools\GitVerse` or a portable drive).
3. Double-click `GitVerse_1.0.0_x64-portable.exe` to launch immediately without administrative privileges.

---

## 4. Native Git CLI Runtime Requirement

Operating under **ADR-001 (Native Git CLI)**, GitVerse requires an installed `git` executable on your system:

- GitVerse automatically discovers the Git binary from your system `PATH`.
- If Git is installed in a non-standard location (e.g., portable MinGit or custom Nix profile), you can set the exact binary path in:
  **Settings (`Ctrl+,`) &rarr; Git Engine &rarr; Git Executable Path**.

---

## 5. Local-First Storage Isolation

GitVerse strictly adheres to the principle that **application state never pollutes your repository working tree**:

- **Repository state**: Pure native `.git` directory unmodified by GitVerse.
- **Application configuration & SQLite metadata**:
  - **Linux**: `~/.local/share/com.gitverse.desktop/`
  - **macOS**: `~/Library/Application Support/com.gitverse.desktop/`
  - **Windows**: `%APPDATA%\com.gitverse.desktop\`
- **Credentials & Tokens**: Secured inside your operating system's native keychain (Secret Service, Keychain, Credential Manager).

---

## 6. Verifying Release Integrity

Every official release includes a `SHA256SUMS.txt` checksum file. Verify your downloaded asset:

```bash
# Linux
sha256sum -c SHA256SUMS.txt --ignore-missing

# macOS
shasum -a 256 -c SHA256SUMS.txt

# Windows (PowerShell)
Get-FileHash -Algorithm SHA256 .\GitVerse_1.0.0_x64-portable.exe
```

---

## 7. In-App Updates & Release Automation

GitVerse includes a built-in cryptographic in-app updater powered by Tauri 2 and Minisign:

- **Automatic Check**: Runs silently on application startup (configurable in Settings &rarr; About &rarr; Software Updates).
- **Cryptographic Verification**: Updates are verified against the GitVerse official Minisign public key before execution.
- **Seamless Application**: Download progress is tracked in real-time; one-click restart applies the new version instantly.
- **Telegram Release Broadcast**: Every published GitHub release automatically generates formatted changelog notes and broadcasts to the official Telegram channel via `scripts/send-telegram-release.mjs`.
