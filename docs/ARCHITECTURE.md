# GitVerse — System Architecture

```text
┌────────────────────────────────────────────────────────┐
│                   Nuxt 4 Desktop UI                    │
│                                                        │
│  Workspace · Staging · Diff · Commit Graph · Remotes   │
│  Command Palette · Settings · Design System Showcase   │
└───────────────────────────┬────────────────────────────┘
                            │
                      Tauri 2 IPC
                            │
┌───────────────────────────▼────────────────────────────┐
│                    Rust Core Layer                     │
│                                                        │
│  Commands · Repository Engine · Safe Process Runner    │
│  Diff Parser · Commit Graph DAG · Provider Adapters    │
│  Persistence (SQLite) · Credential Manager (Keychain)  │
└─────────────┬────────────────────────────┬─────────────┘
              │                            │
      ┌───────▼────────┐           ┌───────▼────────┐
      │ Native Git CLI │           │ Provider APIs  │
      │                │           │                │
      │ Git Repos      │           │ GitHub (REST)  │
      │ SSH / GPG      │           │ GitLab (REST)  │
      │ Filesystem     │           │ OAuth / Tokens │
      └────────────────┘           └────────────────┘
```

## 1. Directory Structure

```text
gitverse/
├── app/                        # Frontend Nuxt 4 application
│   ├── assets/css/main.css     # Tailwind v4, design tokens & theme variables
│   ├── components/             # Domain components (workspace, staging, graph, diff)
│   ├── composables/            # State & controllers (locale, theme, repository)
│   ├── layouts/                # Desktop layouts (default, clean)
│   ├── pages/                  # Views (index, design-system, settings)
│   ├── schemas/                # Zod v4 validation schemas
│   ├── types/                  # Shared TypeScript interfaces
│   └── utils/                  # Pure helper functions
├── src-tauri/                  # Tauri 2 Desktop & Rust Core
│   ├── src/
│   │   ├── application/        # Application state & coordinators
│   │   ├── commands/           # Typed Tauri command handlers
│   │   ├── configuration/      # Config structs & environment loaders
│   │   ├── domain/             # Pure models (Commit, Ref, Change, Lane)
│   │   ├── infrastructure/     # Git CLI runner, SQLite store, Keychain
│   │   ├── services/           # Diff service, Graph builder, Auth service
│   │   └── lib.rs              # Tauri initialization & plugin registration
│   ├── capabilities/           # Tauri permissions
│   └── Cargo.toml              # Rust dependencies
├── docs/                       # Architecture & design documentation
├── i18n/locales/               # Bilingual locale dictionaries (en.json, fa.json)
├── scripts/                    # Validation & dev automation scripts
├── AGENTS.md                   # Permanent engineering constitution
├── implement.md                # Technical implementation backlog & epics
└── package.json                # Frontend package dependencies & scripts
```

## 2. Layer Boundaries & Responsibilities

1. **Frontend (`app/`)**:
   - Manages UI presentation, keyboard shortcuts, virtual scrolling, and SVG viewport interaction.
   - Strictly delegates Git commands and persistence to Tauri IPC.
   - Strictly applies localized strings and design tokens.

2. **Rust Core (`src-tauri/src/`)**:
   - Executes Git CLI commands via safe argument arrays.
   - Builds DAG and calculates deterministic lane assignments off the UI thread.
   - Manages SQLite database and OS Keychain securely.
   - Exposes typed error payloads with stable error codes and human-readable messages.
