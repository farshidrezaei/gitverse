# AGENTS.md — GitVerse Permanent Engineering Constitution

This document is the permanent engineering contract for **GitVerse**. Every contributor and coding agent must read and obey it before designing, editing, or reviewing code.

Source specification: `implement.md`.

---

## 1. Product Scope & Identity

- **GitVerse** is a modern, cross-platform, local-first, open-source Git desktop client combining powerful Git operations, advanced visual history exploration, and GitHub/GitLab integrations.
- **Tech Stack**: Tauri 2 + Rust + Nuxt 4 + Nuxt UI + Tailwind CSS 4 + SQLite.
- **Local-first (ADR-003)**: Core Git operations must function independently of any network connection or cloud service. Native Git CLI is the single source of truth (ADR-001).
- **Custom Commit Graph (ADR-002)**: Application owns DAG layout, lane assignment, and viewport rendering.
- **Modular Monolith**: Application runs as a clean, single desktop client with structured background tasks.

---

## 2. Non-Negotiable Coding Rules

### 2.1. Naming and Readability
- Never use abbreviations in variable, function, method, class, component, module, or type names unless the abbreviation is an established technical term or protocol name (`HTTP`, `HTTPS`, `SSH`, `MIME`, `API`, `URL`, `URI`, `UUID`, `JSON`, `HTML`, `CSS`, `SQL`, `IPC`, `UI`, `RTL`, `LTR`, `SHA`, `DAG`, `ID`, `CLI`, `PR`, `MR`, `CI`, `SVG`).
- Forbidden abbreviations: `cfg`, `ctx`, `mgr`, `svc`, `repo` (use `repository`), `req`, `res`, `msg` (use `message`), `usr` (use `user`), `val` (use `value`), `env` (use `environment`), `auth` (prefer `authentication` / `credential`), `diff_eng` (use `diff_engine`).
- Boolean identifiers must carry explicit prefixes:
  - Rust: `is_`, `has_`, `can_`, `should_`
  - TypeScript: `is`, `has`, `can`, `should`
- Functions and methods must state intent through their names without cryptic identifiers.

### 2.2. No Hardcoded Values
Every non-structural value must belong to one of six categories:
1. **Enum** — finite, explicitly defined set of states or variants (`ChangeStatus`, `ReferenceType`, `CommitGraphLane`, `ProviderKind`).
2. **Configuration value** — user- or application-configurable setting (`WorkspaceConfiguration`, `ThemeSettings`).
3. **Named constant** — true invariant (`DEFAULT_HISTORY_PAGE_SIZE`, `MAX_DIFF_LINE_BYTES`).
4. **Design token** — visual design decision (`--gv-primary`, `--gv-branch-lane-1`, `@theme` tokens).
5. **Translation key** — any user-facing text (`$t('repository.empty.title')`).
6. **Domain value** — value belonging to a persisted business entity.

Never scatter magic numbers, literal UI colors/hex values, or hardcoded user-facing strings inside components or handlers.

### 2.3. Theme Customization & Token System
- GitVerse must support full user customization of theme colors (primary, accent, background surfaces, diff colors, graph lane colors) alongside built-in default themes (`GitVerse Dark` and `GitVerse Light`).
- Dynamic overrides must flow through CSS custom properties without breaking WCAG contrast standards.

### 2.4. Modularity (No God Files or God Components)
Strictly prohibited:
- God files, God classes, God components, giant composables, giant Rust modules, excessively large functions.
- Business logic or Git CLI invocation inside Vue components.
- Direct database queries or filesystem operations inside Tauri command handlers.
- Duplicated validation or persistence logic.

Split modules by domain responsibility:
- **Rust (`src-tauri/src/`)**: `application/`, `configuration/`, `domain/`, `infrastructure/`, `services/`, `commands/`, `events/`.
- **Frontend (`app/`)**: `components/` (by domain feature), `composables/`, `layouts/`, `pages/`, `stores/`, `types/`, `utils/`, `schemas/`, `assets/css/`.

### 2.5. Rust Safety & Process Execution
- **Never** use `.unwrap()` or `.expect()` in production Rust paths without a documented, mathematically guaranteed invariant.
- Propagate typed domain and infrastructure errors with `thiserror` and map cleanly to safe, structured command responses.
- **Never invoke shell strings via interpolation**. All Git operations must execute via argument arrays with sanitized working directory and cancellation token support (ADR-001).
- Never log credentials, SSH keys, personal access tokens, or sensitive repository payloads.

### 2.6. TypeScript & Frontend Discipline
- TypeScript `strict: true` is mandatory.
- **Nuxt UI first**: Always check official Nuxt UI components before writing custom UI primitives.
- **Minimal custom CSS**: Use Nuxt UI props/variants, Tailwind v4 utilities, and centralized CSS tokens. Use CSS logical properties (`ms-*`, `me-*`, `ps-*`, `pe-*`, `start-*`, `end-*`, `text-start`, `text-end`) for automatic LTR/RTL symmetry.

### 2.7. Zero-CLS Skeleton Loading Standard
- **Never use generic spinners (`UIcon` spinner, CSS spinner) or text labels (e.g. `Loading...`) for views, modal bodies, sidebars, cards, panels, or lists.**
- Every loading state must be a high-fidelity, dimension-accurate skeleton built with `USkeleton`.
- Skeletons must mirror the exact geometry, hierarchy, borders, padding, avatar shapes, badges, and layout of the resolved content to guarantee **zero Cumulative Layout Shift (CLS)** and prevent jarring visual jumps.
- Spinner loaders are strictly confined to inline button action states (`UButton :loading="..."`) during brief in-flight actions.

### 2.8. Modal Architecture & Surface Contract
- **Never render a modal or dialog without an opaque surface background (`bg-[var(--gv-surface)]`) and border (`border-[var(--gv-border)]` / `border-default/70`).**
- Every modal container (`content` slot / root wrapper) must have:
  - Background surface: `bg-[var(--gv-surface)]`
  - Border and elevation: `border border-default/70 shadow-2xl rounded-2xl`
  - Backdrop overlay: `fixed inset-0 bg-black/60`
  - Proper padding, header with title and close button, body, and action footer.
- The default `app.config.ts` modal slot defines `content: 'bg-[var(--gv-surface)] border border-[var(--gv-border)] shadow-2xl rounded-2xl overflow-hidden focus:outline-none'`. Never override it with `bg-transparent` or strip its background surface.

### 2.9. Critical Action Confirmations & Actionable Undo Contract
- **Never trigger a destructive, mutating, or history-altering operation without appropriate tiered confirmation**:
  - **Tier 1 (Standard Dialog)**: Reversible mutating operations (Revert Commit, Cherry-Pick, Detached Checkout, Abort Merge/Rebase, Delete Staged Changes).
  - **Tier 2 (Hold-to-Confirm)**: Irreversible lossy local operations (Hard Reset, Discard All Unstaged Changes).
  - **Tier 3 (Type-to-Confirm Keyword)**: Catastrophic or remote destructive operations (Delete Branch, Force Push).
- **Actionable Undo**: Whenever a mutating operation is reversible (e.g. Revert commit, Delete local branch, Discard file changes), the success notification toast must invoke `notifyUndoable` with an immediate, one-click `Undo` button.
- **Detailed Error Reporting**: All failure notifications must pass `formatApplicationError(error)` to `notifyError` to prevent generic or unhandled error messages from being displayed to users.

### 2.10. Dual-Layer Input Validation Contract (Zod & Rust Domain)
- **Mandatory Dual-Layer Validation**: Every user-configurable input, form field, action argument, or external payload that requires validation **MUST** be validated on **both** the frontend (using strict **Zod** schemas) and the backend (using **Rust domain validators** in `src-tauri/src/domain/validation.rs`).
- **Frontend Zod Standard (`app/schemas/`)**:
  - All form modals, inputs, and command invocations must define or import their schema from `app/schemas/`.
  - Every schema issue must map directly to an internationalized error key (`errorKey: '...'`), never hardcoded strings.
  - UI inputs must perform pre-flight `safeParse()` checks before invoking Tauri IPC commands, showing immediate feedback banners without firing unnecessary IPC calls.
- **Backend Rust Standard (`src-tauri/src/domain/validation.rs`)**:
  - The Rust backend must never trust input from the frontend.
  - All command handlers and domain service methods must invoke the corresponding `validate_*` function before executing Git processes or writing to SQLite/disk.
  - Rust validators must reject null bytes (`\0`), control characters, unescaped path traversals, out-of-range lengths, or illegal Git characters with typed `ApplicationError::InvalidInput(...)`.


---

## 3. Internationalization (i18n) & Theme Matrix

- Mandatory initial locales: **English (`en`, `dir: 'ltr'`)** and **Persian (`fa`, `dir: 'rtl'`)**.
- Mandatory default themes: **Dark (`GitVerse Dark`)** and **Light (`GitVerse Light`)**, plus **User Custom Theme** support.
- Every feature requires both `en` and `fa` translations and verification in both themes and both directions.
- Automated i18n validation script (`scripts/validate-i18n.mjs`) must verify key parity, non-empty values, and placeholder matching.

---

## 4. Testing Constitution

A feature is **not** complete when it compiles. Required coverage:
- **Table-driven tests** for validation, status parsing, diff parsing, and error mapping.
- **Matrix tests** covering operations across repository states (clean, dirty, detached HEAD, merge conflict, shallow clone).
- **Property-based tests** (`proptest`) for graph DAG construction, lane assignment determinism, and commit parsing.
- **Real integration tests** executing real Git operations against temporary scratch repositories.
- **Frontend & E2E tests** verifying rendering, graph interaction, diff viewer, theme switching, and RTL layout.

---

## 5. Mandatory Project Documentation

### 5.1. Mandatory Documentation Synchronization Contract
Every contributor and coding agent is bound to this permanent contract:
- **Zero Drift Principle**: Whenever code is added, updated, or removed (Tauri IPC commands, Rust domain modules/services, frontend components, composables, stores, or configuration), all affected documentation (`AGENTS.md`, `README.md`, `docs/ARCHITECTURE.md`, `docs/TESTING.md`, `docs/modules/*`, etc.) **MUST be updated synchronously within the same task or commit**.
- Submitting code changes without updating corresponding documentation violates the engineering constitution and fails the Definition of Done.
- Documentation must always reflect actual live code symbols, accurate test metrics, and current architectural reality.

### 5.2. Monitored Documentation Suite
Keep these documents synchronized with every architectural or behavioral change:
- `AGENTS.md` — Permanent engineering constitution
- `README.md` — Product overview, architecture, development workflow
- `docs/ARCHITECTURE.md` — System architecture and module boundaries
- `docs/DECISIONS.md` — Architectural Decision Records (ADRs)
- `docs/DESIGN-SYSTEM.md` — Design tokens, customizable themes, and UI standards
- `docs/I18N.md` — Localization structure, RTL/LTR rules, CI validation
- `docs/TESTING.md` — Testing strategy, fixtures, benchmarks, commands
- `docs/DEVELOPMENT.md` — Local environment, build, and contribution workflow
- `docs/modules/` — Deep architectural domain module specifications

---

## 6. Definition of Done Checklist

Before marking any task complete, verify all applicable items:
1. Follows modular architecture and layer boundaries.
2. Zero prohibited abbreviations in identifiers.
3. Zero unexplained hardcoded values or literal user-facing strings.
4. No God files or oversized components.
5. Nuxt UI used first; custom CSS justified if present.
6. Complete `en` and `fa` translations passing validation script.
7. Verified in RTL (`fa`) and LTR (`en`).
8. Verified in Light, Dark, and Custom themes.
9. High-fidelity zero-CLS skeletons (`USkeleton`) implemented for all loading states (strictly no generic content spinners or "Loading..." text), plus complete empty and error states.
10. Table-driven, matrix, property, or integration tests written and passing.
11. Formatting, linting, type checking, and test suites executed after final edits.
12. Strict dual-layer input validation implemented (Zod in frontend `app/schemas/` and Rust domain validators in `domain/validation.rs`) for any new or modified input paths.
13. **Documentation synchronously updated across all relevant docs files for any added, modified, or removed code/features (mandatory contract).**
