# GitVerse — Testing Strategy & Quality Assurance

## 1. Testing Pyramid

- **Rust Unit & Property Tests**:
  - `proptest` for Git DAG normalization, lane assignment stability, and hash parsing.
  - Table-driven unit tests for status porcelain v2 parsing and diff hunk parsing.
  - Process runner timeout, cancellation, and argument array isolation tests.
- **Rust Integration Tests**:
  - Real Git tests using `tempfile` scratch repositories creating linear, merge, and conflicted histories.
- **Frontend Unit Tests (`Vitest`)**:
  - Component rendering in LTR and RTL.
  - Theme switching and custom palette CSS injection.
  - Form validation with Zod schemas.
- **E2E Tests (`Playwright`)**:
  - Full desktop flow: repository opening, staging, commit creation, graph selection.

## 2. Testing Commands

```bash
# Frontend unit tests
npm run test:unit

# i18n parity check
npm run validate:i18n

# Type checking
npm run typecheck

# Rust unit & integration tests
cargo test --manifest-path src-tauri/Cargo.toml
```
