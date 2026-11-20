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

## 2. Installation & Execution Instructions

### 2.1. Linux (AppImage)

The AppImage package requires no installation and can be executed directly from any folder or USB drive:

```bash
# 1. Make the downloaded AppImage executable
chmod +x GitVerse_1.0.0_amd64.AppImage

# 2. Launch GitVerse
./GitVerse_1.0.0_amd64.AppImage
```

> [!NOTE]
> Ensure FUSE (`libfuse2` on Ubuntu 22.04+) is present on your system to launch AppImage files.

### 2.2. Linux (Debian / Ubuntu Package)

```bash
sudo dpkg -i gitverse_1.0.0_amd64.deb
sudo apt-get install -f
```

### 2.3. macOS (Universal DMG)

1. Download `GitVerse_1.0.0_universal.dmg`.
2. Double-click the DMG to mount it.
3. Drag `GitVerse.app` into your `/Applications` directory (or run it directly from the volume).

### 2.4. Windows (Portable)

1. Download `GitVerse_1.0.0_x64-portable.exe`.
2. Move it to your preferred directory (e.g. `C:\Tools\GitVerse` or a portable drive).
3. Double-click `GitVerse_1.0.0_x64-portable.exe` to launch immediately without administrative privileges.

---

## 3. Native Git CLI Runtime Requirement

Operating under **ADR-001 (Native Git CLI)**, GitVerse requires an installed `git` executable on your system:

- GitVerse automatically discovers the Git binary from your system `PATH`.
- If Git is installed in a non-standard location (e.g., portable MinGit or custom Nix profile), you can set the exact binary path in:
  **Settings (`Ctrl+,`) &rarr; Git Engine &rarr; Git Executable Path**.

---

## 4. Local-First Storage Isolation

GitVerse strictly adheres to the principle that **application state never pollutes your repository working tree**:

- **Repository state**: Pure native `.git` directory unmodified by GitVerse.
- **Application configuration & SQLite metadata**:
  - **Linux**: `~/.local/share/com.gitverse.desktop/`
  - **macOS**: `~/Library/Application Support/com.gitverse.desktop/`
  - **Windows**: `%APPDATA%\com.gitverse.desktop\`
- **Credentials & Tokens**: Secured inside your operating system's native keychain (Secret Service, Keychain, Credential Manager).

---

## 5. Verifying Release Integrity

Every official release includes a `SHA256SUMS.txt` checksum file. Verify your downloaded asset:

```bash
# Linux
sha256sum -c SHA256SUMS.txt --ignore-missing

# macOS
shasum -a 256 -c SHA256SUMS.txt

# Windows (PowerShell)
Get-FileHash -Algorithm SHA256 .\GitVerse_1.0.0_x64-portable.exe
```
