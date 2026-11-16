# GitVerse — Architectural Decision Records (ADRs)

## ADR-001: Native Git CLI Process Execution

- **Status:** Accepted
- **Context:** Git clients need high fidelity, wide compatibility with user systems, and complete feature availability without reimplementing complex internals or suffering from libgit2 version drift.
- **Decision:** Execute the installed native Git executable via a safe process execution layer:
  - Never construct shell commands via string interpolation.
  - Always execute processes using argument arrays (`std::process::Command` / Tokio process).
  - Use structured, machine-readable Git output formats (`--porcelain`, `--format=...`, `-z`).
  - Support execution timeouts, progress events, and cancellation tokens.
  - Provide fallback detection and custom executable path configuration.
- **Consequences:** Maximum compatibility with user Git configs, SSH keys, GPG signing, and Git hooks. Shell injection is structurally impossible.

---

## ADR-002: In-House Custom Commit Graph Engine

- **Status:** Accepted
- **Context:** A key differentiator for GitVerse is an intuitive, fast, interactive commit graph. Generic external SVG libraries fail on repositories with thousands of branches, merges, and deep history.
- **Decision:** Build and own the graph pipeline end-to-end:
  - History extraction and commit normalization in Rust/TypeScript.
  - Directed Acyclic Graph (DAG) construction maintaining parent-child edges and multiple merge parents.
  - Deterministic lane assignment algorithm with branch stability across incremental loads.
  - Virtualized custom SVG viewport renderer avoiding per-commit Vue component overhead.
- **Consequences:** Deep performance optimization (smooth 2,000+ commit rendering), seamless theme integration, and custom interactive capabilities (cherry-pick, rebase preview, branch comparison).

---

## ADR-003: Local-First Desktop Architecture

- **Status:** Accepted
- **Context:** Developers frequently work offline or in restricted network environments. Git itself is distributed and local-first.
- **Decision:** The application runs as a modular desktop monolith without requiring an internet connection or proprietary cloud services.
  - Local repository operations operate directly on the filesystem.
  - Application settings and repository workspace history persist in local SQLite.
  - Remote provider integrations (GitHub / GitLab) are optional overlays that gracefully degrade when offline.
- **Consequences:** User data privacy, instant startup, zero cloud downtime vulnerability, and genuine open-source portability.

---

## ADR-004: Provider Abstraction for GitHub & GitLab

- **Status:** Accepted
- **Context:** GitHub and GitLab have divergent data models, authentication methods, and terminology (Pull Request vs Merge Request).
- **Decision:** Implement independent adapters adhering to a unified core contract (`provider-core`), while preserving provider-specific capabilities:
  - Credentials stored in OS Keychain via `keyring`.
  - Rate limiting, token refresh, and pagination handled modularly per provider.
  - Provider features do not artificially restrict or leak abstractions across platforms.
- **Consequences:** Clear provider separation, secure credential handling, and maintainable API integrations.
