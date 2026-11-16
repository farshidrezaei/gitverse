# GitVerse

> Modern, cross-platform, local-first Git client desktop application with an in-house commit graph engine, advanced diff visualizer, and GitHub/GitLab integrations.

---

## Highlights

- **Native Git CLI Core (ADR-001)**: Safe process execution via argument arrays without shell interpolation.
- **Custom Commit Graph Engine (ADR-002)**: In-house DAG construction, deterministic branch lane assignment, and high-performance virtualized SVG rendering.
- **Local-First Architecture (ADR-003)**: Functions 100% offline. SQLite metadata storage, OS Keychain credential security.
- **Provider Abstraction (ADR-004)**: Clean GitHub and GitLab adapters without artificial normalization.
- **Customizable Developer Design System**: Built-in `GitVerse Dark` (Deep Graphite) and `GitVerse Light` (Crisp Clean White) with real-time color personalization.
- **Full Bilingual Parity**: English (`en`, LTR) and Persian (`fa`, RTL with Vazirmatn).
- **Living Style Guide**: Interactive component and token verification route at `/design-system`.

---

## Technical Stack

| Layer | Technology |
|---|---|
| Desktop Runtime | Tauri 2 |
| Frontend Framework | Nuxt 4 (Vue 3 + TypeScript) |
| Styling & UI | Tailwind CSS 4 + Nuxt UI 4 |
| Native Core | Rust (tokio, rusqlite, thiserror) |
| Persistence | SQLite |
| Credential Storage | OS Keychain (`keyring`) |
| Testing | Rust unit/integration + Vitest |

---

## Quick Start

```bash
# 1. Install dependencies
npm install

# 2. Validate translation keys
npm run validate:i18n

# 3. Start web development preview
npm run dev

# 4. Start Tauri desktop app
npm run tauri dev
```

---

## Engineering Constitution

All development is strictly governed by [`AGENTS.md`](./AGENTS.md) and [`implement.md`](./implement.md).
