## Summary of Changes

<!-- Provide a concise description of what this PR introduces, fixes, or refactors. -->

## Motivation & Context

<!-- Closes #issue_number or explains the architectural rationale. -->

## Checklist per `AGENTS.md` Constitution

- [ ] **Naming Discipline**: Zero forbidden abbreviations (`cfg`, `ctx`, `mgr`, `svc`, `repo`, `req`, `res`, `msg`, `usr`, `val`, `env`, `auth`). Boolean flags carry clear prefixes (`is*`, `has*`, `can*`, `should*`).
- [ ] **Dual-Layer Validation**: All user inputs validated on both frontend (Zod schema in `app/schemas/`) and backend (Rust domain validator in `domain/validation.rs`).
- [ ] **Zero Hardcoded Strings**: All user-facing text localized via `$t(...)` / `t(...)` with complete parity across `en.json` and `fa.json` (`npm run validate:i18n` passes).
- [ ] **Bidirectional Design**: Verified in both LTR (English) and RTL (Persian) with logical Tailwind utilities (`ms-*`, `me-*`, `ps-*`, `pe-*`).
- [ ] **Zero-CLS Skeletons**: Dimension-accurate `USkeleton` used for all asynchronous views (no generic spinners or "Loading..." labels).
- [ ] **Modal Surface Contract**: All modals use opaque surfaces (`bg-[var(--gv-surface)]`), borders, shadows, and backdrop overlays.
- [ ] **Testing Standards**: New or updated behavior is verified by unit, table-driven, or integration tests (`npm run test:unit` and `cargo test` pass).
- [ ] **Code Formatting & Linting**: Rust code formatted (`cargo fmt --check`) and clippy warnings clean (`cargo clippy -- -D warnings`). TypeScript typecheck passes (`npm run typecheck`).
- [ ] **Documentation Sync**: Any changes to architecture, domain models, or CLI commands are reflected in `docs/` and `README.md`.
