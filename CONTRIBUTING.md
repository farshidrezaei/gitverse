# Contributing to GitVerse

Thank you for your interest in contributing to **GitVerse**! GitVerse is a modern, cross-platform, local-first Git desktop client combining native Git CLI execution, a custom deterministic commit graph, and GitHub/GitLab code review integration.

All contributors and coding agents must adhere to the permanent engineering constitution defined in [`AGENTS.md`](AGENTS.md).

---

## 1. Code of Engineering Discipline

### 1.1. Zero-Abbreviation Naming Rule
- Never use abbreviations in variable, function, method, class, component, module, or type names unless the abbreviation is an established industry standard protocol (`HTTP`, `SSH`, `API`, `URL`, `JSON`, `IPC`, `UI`, `RTL`, `LTR`, `SHA`, `DAG`, `ID`, `CLI`, `PR`, `MR`, `CI`, `SVG`).
- **Forbidden abbreviations**: `cfg`, `ctx`, `mgr`, `svc`, `repo` (use `repository`), `req`, `res`, `msg` (use `message`), `usr` (use `user`), `val` (use `value`), `env` (use `environment`), `auth` (use `authentication` or `credential`).
- Boolean identifiers must carry explicit prefixes:
  - Rust: `is_`, `has_`, `can_`, `should_`
  - TypeScript: `is`, `has`, `can`, `should`

### 1.2. Zero Hardcoded Values
Every value must belong to one of six categories:
1. **Enum** (`FileChangeStatus`, `CommitGraphLane`, `ProviderKind`).
2. **Configuration setting** (`WorkspaceConfiguration`, `ThemeSettings`).
3. **Named constant** (`DEFAULT_HISTORY_PAGE_SIZE`, `MAX_DIFF_LINE_BYTES`).
4. **Design token** (`--gv-surface`, `--gv-primary`, `@theme`).
5. **Translation key** (`t('git.commit')`).
6. **Domain value** (business entity ID, user input).

### 1.3. UI Standards (Zero-CLS Skeletons & Modal Surfaces)
- **Zero-CLS Skeletons**: Never use generic spinners or "Loading..." text for views, modal bodies, sidebars, cards, panels, or lists. Every loading state must use a high-fidelity, dimension-accurate `USkeleton` matching the exact layout of resolved content.
- **Modal Surface Contract**: Every modal container must feature an opaque surface background (`bg-[var(--gv-surface)]`), border (`border border-default/70`), elevation shadow, and backdrop overlay (`fixed inset-0 bg-black/60`). Never strip the modal surface background.
- **Nuxt UI First**: Always use Nuxt UI v3 primitives before writing custom UI components.
- **Logical CSS**: Use logical properties (`ms-*`, `me-*`, `ps-*`, `pe-*`, `start-*`, `end-*`, `text-start`, `text-end`) to guarantee bidirectional symmetry in English (LTR) and Persian (RTL).

### 1.4. Rust Safety & Process Execution
- Never use `.unwrap()` or `.expect()` in production Rust paths without a mathematically proven, documented invariant.
- Propagate typed domain and infrastructure errors with `thiserror`.
- Never execute Git operations using interpolated shell strings. All Git processes must execute via isolated argument vectors.

---

## 2. Local Development Workflow

### 2.1. Prerequisites
- **Node.js**: v20 or higher
- **Rust & Cargo**: 1.80 or higher
- **Native Git**: 2.30 or higher installed on system `PATH`
- **Tauri 2 Prerequisites**: Platform libraries (`libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `libsecret-1-dev` on Linux)

### 2.2. Commands

```bash
# Install dependencies
npm install

# Start desktop development environment (Vite frontend + Tauri backend)
npm run tauri:dev

# Run type checker
npm run typecheck

# Validate internationalization key parity
npm run validate:i18n

# Run frontend Vitest test suite
npm run test:unit

# Run Rust unit and integration tests
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 3. Mandatory Documentation Synchronization

GitVerse enforces the **Zero Drift Principle**:
- When adding, updating, or removing any IPC command, Rust service/domain model, frontend component, composable, or configuration option, **all corresponding documentation (`README.md`, `docs/ARCHITECTURE.md`, `docs/TESTING.md`, `docs/modules/*`) MUST be updated synchronously within the same pull request**.
- PRs that introduce code changes without updating affected documentation will not pass code review.

---

## 4. Definition of Done (DoD) Checklist

Before submitting a pull request, ensure:
- [ ] Conforms to modular architecture and layer boundaries.
- [ ] Zero prohibited abbreviations across all new identifiers.
- [ ] No hardcoded user-facing strings or magic numbers.
- [ ] Complete `en` and `fa` translations passing `npm run validate:i18n`.
- [ ] Verified in both LTR (`en`) and RTL (`fa`).
- [ ] High-fidelity zero-CLS `USkeleton` implemented for all loading states.
- [ ] Comprehensive unit, integration, or property-based tests added and passing.
- [ ] `npm run typecheck`, `npm run test:unit`, and `cargo test` pass with zero warnings or errors.
- [ ] Documentation updated across all relevant Markdown guides.
