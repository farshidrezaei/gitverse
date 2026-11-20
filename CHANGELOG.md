# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] - 2026-11-21

### Added

#### Desktop Core & Process Execution
- Native Git CLI discovery, version checking, and execution orchestration (`ADR-001`).
- Safe argument vector executor preventing command injection vulnerabilities.
- Strongly-typed Tauri IPC contracts with structured `ApplicationError` mappings.
- Native SQLite database persistence for repository history, tabs, and provider accounts.
- Cross-platform secure credential storage using OS keyrings (Secret Service, Keychain, Credential Manager).

#### Commit Graph & Visual History
- Custom deterministic DAG layout engine with branch lane assignment (`ADR-002`).
- Virtualized SVG commit graph canvas rendering up to 10,000 commits in under 30ms.
- Two-commit comparison inspector with unified diff calculation.
- Cryptographic commit signature verification badge (GPG, SSH, S/MIME) with signer and key details.
- Interactive File Blame viewer with line-by-line commit authorship, timestamps, and commit inspection navigation.

#### Git Operations & Workspace
- Complete working tree changes status tracker with both Flat and Hierarchical Tree views.
- Line-by-line and hunk-by-hunk staging, unstaging, and discarding.
- Intra-line character-level diff highlighting for added and removed spans.
- Stash management with full diff inspection and file preview modal (`StashPreviewModal`).
- Git hooks bypass toggle (`--no-verify`) for commits.
- Branch management with local/remote grouping, filtering, checkout, merge, and rebase.
- Interactive rebase interface with pick, squash, fixup, reword, edit, and drop action ordering.
- 3-way visual conflict resolution modal with "Accept Current", "Accept Incoming", or "Accept Both".
- Git worktree manager for creating, listing, checking out, and pruning worktrees.

#### Cloud Provider Integrations (GitHub & GitLab)
- Multi-account connection support for GitHub (Personal Access Token & OAuth) and GitLab (Cloud & Self-Hosted instances).
- Pull Request and Merge Request explorer with state filtering (Open, Merged, Closed), author search, and labels.
- Code review interface supporting line-level comments, submission of reviews (Approve, Request Changes, Comment), and merging (Merge Commit, Squash and Merge, Rebase and Merge).
- CI/CD pipeline summaries and automated check run inspect panels.

#### Desktop Experience & Integration
- Multi-repository workspace tabs with independent staging and graph state.
- Scoped full-page Diff tabs and Pull Request review tabs.
- Quick Command Palette (`Ctrl+K` / `Cmd+K`) for keyboard-driven navigation across repositories, branches, and commands.
- External tool launchers for instant launching into VS Code, Cursor, Zed, Sublime Text, IntelliJ IDEA, System Terminal, and File Manager.
- Application settings dialog with custom theme color overrides, typography scaling, border radius, and Git binary configurations.

#### Security, Logging & Diagnostics
- Sanitized circular logging buffer with automatic redaction of bearer tokens, PATs, and URL passwords (`GC-038`).
- One-click Diagnostic Bundle export for voluntary issue reporting.
- Global application error boundary (`NuxtErrorBoundary`) preventing unhandled desktop crashes.

#### Internationalization & Design System
- Full bidirectional support: English (`en`, LTR) and Persian (`fa`, RTL) with 100% key parity.
- Dark and Light theme presets with dynamic CSS token customization compliant with WCAG contrast standards.
- High-fidelity zero-CLS `USkeleton` loading states across all panels and dialogs.
