# GitVerse — Architectural Decision Records (ADRs)

This document records the foundational architectural decisions made for GitVerse, including their context, decisions, trade-offs, and consequences.

---

## Index of Records

- [ADR-001: Native Git CLI Process Execution](#adr-001-native-git-cli-process-execution)
- [ADR-002: In-House Custom Commit Graph Engine](#adr-002-in-house-custom-commit-graph-engine)
- [ADR-003: Local-First Desktop Architecture](#adr-003-local-first-desktop-architecture)
- [ADR-004: Provider Abstraction for GitHub & GitLab](#adr-004-provider-abstraction-for-github--gitlab)
- [ADR-005: Zero-CLS Skeleton Loading Standard](#adr-005-zero-cls-skeleton-loading-standard)
- [ADR-006: Opaque Modal Surface Contract](#adr-006-opaque-modal-surface-contract)
- [ADR-007: Hierarchical Two-Level Workspace Tabs](#adr-007-hierarchical-two-level-workspace-tabs)

---

## ADR-001: Native Git CLI Process Execution

- **Status:** Accepted
- **Context:** Git desktop clients face a choice between linking against C/Rust libraries (`libgit2` / `git2-rs`) versus executing the system's installed `git` CLI binary. `libgit2` frequently lags behind native Git features (e.g. sparse-checkouts, worktrees, modern interactive rebase, custom credential helpers, SSH signing, GPG signing, and `.gitconfig` conditional includes).
- **Decision:** Execute the installed native Git executable via a safe asynchronous process supervisor:
  - **Zero Shell Interpolation**: Commands are never executed through shell string interpolation (`sh -c`). Process commands are spawned directly using isolated argument arrays (`std::process::Command` / Tokio process).
  - **Structured Formats**: Git output is captured in machine-readable formats (`--porcelain=v2`, `--format=%H%x00...`, `-z`).
  - **Cancellation & Timeouts**: Async process supervisor supports timeouts, progress reporting, and cancellation tokens.
  - **Environment Sanitization**: Clean working directory validation, path traversal prevention, and redaction of credentials from output.
- **Consequences:** 100% parity with user Git configuration, SSH keys, GPG signing, and custom hooks. Complete immunity to shell injection vulnerabilities.

---

## ADR-002: In-House Custom Commit Graph Engine

- **Status:** Accepted
- **Context:** Visualizing Git history with thousands of branches, divergent paths, and merge commits requires deterministic, low-latency rendering. Off-the-shelf graphing libraries either degrade in performance with thousands of DOM nodes or produce chaotic, crossing lane lines ("spaghetti graphs").
- **Decision:** Design and own the complete DAG graph pipeline:
  - **DAG Model & Topological Sort**: Implemented in Rust (`services/graph.rs`).
  - **Deterministic Lane Allocator**: A greedy reservation algorithm that preserves the active branch on lane 0 and minimizes horizontal lane changes.
  - **Virtualized Canvas Rendering**: Vue 3 windowing virtualizer (`useCommitGraph.ts`) coupled with hardware-accelerated SVG cubic Bézier paths (`CommitGraphSvgCanvas.vue`).
- **Consequences:** Smooth 60fps rendering of 5,000+ commits, zero component bloat per row, and complete control over graph styling and theme tokens.

---

## ADR-003: Local-First Desktop Architecture

- **Status:** Accepted
- **Context:** Developers demand fast, offline-capable Git clients that do not rely on remote servers or proprietary cloud backends.
- **Decision:** GitVerse operates as a local-first desktop application:
  - Repository data is queried directly from the local filesystem via Git CLI.
  - User preferences, recent repositories, and theme customizations persist in a local SQLite database (`gitverse.db`).
  - Cloud provider integrations (GitHub and GitLab) are strictly optional enhancements and fail gracefully when offline.
- **Consequences:** User privacy is guaranteed, startup time is under 200ms, and zero network requests occur during local Git operations.

---

## ADR-004: Provider Abstraction for GitHub & GitLab

- **Status:** Accepted
- **Context:** GitHub and GitLab offer similar capabilities (pull/merge requests, code review, CI/CD pipelines) but differ significantly in REST endpoints, data structures, and authorization schemes.
- **Decision:** Implement dedicated provider adapters adhering to a unified domain contract without forced or lossy normalization:
  - **Credential Security**: Access tokens are stored in the operating system's native keychain via `keyring` (Keychain on macOS, Secret Service/libsecret on Linux, Credential Manager on Windows).
  - **Full Enterprise & Self-Hosted Support**: Seamless instance URL configuration for private GitHub Enterprise and GitLab CE/EE instances.
  - **In-App Review & Merge**: Full support for multi-line inline comments, review state submission (`APPROVE`, `REQUEST_CHANGES`, `COMMENT`), and merge strategy execution (`merge`, `squash`, `rebase`).
- **Consequences:** Deep integration with GitHub and GitLab while keeping local Git operations independent.

---

## ADR-005: Zero-CLS Skeleton Loading Standard

- **Status:** Accepted
- **Context:** Asynchronous Git operations and provider queries introduce loading states. Using generic spinners or text labels causes Cumulative Layout Shift (CLS) when content resolves, creating visual jarring and poor user experience.
- **Decision:** Strictly ban generic content spinners and "Loading..." labels across the entire application:
  - Every loading state must be a dimension-accurate, high-fidelity skeleton implemented using `USkeleton`.
  - Skeletons must mirror the exact hierarchy, borders, padding, avatar shapes, badges, and geometry of the final content.
  - Spinners are strictly confined to inline button loading states (`UButton :loading="..."`) during brief in-flight actions.
- **Consequences:** 0 Cumulative Layout Shift across all panels, sidebars, modals, and lists.

---

## ADR-006: Opaque Modal Surface Contract

- **Status:** Accepted
- **Context:** Modals rendered with transparent or semi-transparent content backgrounds cause text and controls to collide visually with underlying workbench islands.
- **Decision:** Enforce an opaque background surface contract across all dialogs:
  - The default Nuxt UI modal slot in `app.config.ts` enforces `bg-[var(--gv-surface)] border border-[var(--gv-border)] shadow-2xl rounded-2xl overflow-hidden`.
  - Every modal must feature an opaque header, scrollable body, and action footer.
  - Modals must never override content with `bg-transparent`.
- **Consequences:** Flawless visual hierarchy, legible typography in all themes, and zero content bleeding through dialog surfaces.

---

## ADR-007: Hierarchical Two-Level Workspace Tabs

- **Status:** Accepted
- **Context:** Developers frequently switch between multiple repositories or need to view large diffs and pull requests without being cramped in small sidebar panels or modals.
- **Decision:** Implement a two-level tab model:
  - **Level 1 (Top Bar)**: Open repositories and worktrees (`WorkspaceRepositoryTabBar.vue`), persisted in `localStorage`.
  - **Level 2 (Scoped Sub-Tab Bar)**: Scoped to the active repository (`WorkspaceDiffTabBar.vue`), providing the main Workbench dashboard, scoped file diff tabs, and full-screen pull request tabs (`WorkspacePullRequestTabContent.vue`).
  - Main workbench island is preserved via `v-show` for instantaneous `0ms` switching.
- **Consequences:** Seamless multi-tasking across repositories, full-screen code reviews, and instant tab restoration upon relaunch.
