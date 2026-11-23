# GitVerse — Local Development & Contribution Guide

> Comprehensive guide for setting up your local environment, building, debugging, and contributing to GitVerse.

---

## 1. Prerequisites & Toolchain Setup

### Required Tools

- **Node.js**: `20.x`, `22.x`, or `24.x` (LTS recommended)
- **Rust Toolchain**: Stable `1.80+` (`rustup default stable`)
- **Native Git**: `2.30+` installed on system `$PATH`
- **Tauri CLI**: Installed automatically via project dependencies (`@tauri-apps/cli`)

### Platform-Specific Dependencies

#### Linux (Debian / Ubuntu)
```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### macOS
- Xcode Command Line Tools: `xcode-select --install`

#### Windows
- Microsoft C++ Build Tools
- WebView2 Runtime (preinstalled on Windows 10/11)

---

## 2. Quick Setup & Development Commands

```bash
# 1. Clone the repository
git clone https://github.com/farshidrezaei/gitverse.git
cd gitverse

# 2. Install Node dependencies
npm install

# 3. Validate translations
npm run validate:i18n

# 4. Start frontend preview in browser (http://localhost:3000)
npm run dev

# 5. Launch Tauri desktop app in dev mode (Hot Module Reload enabled)
npm run tauri dev

# 6. Test one-line installer script locally
./scripts/install.sh --help
```

---

## 3. Contribution & Coding Contract

Every contributor and agent must obey **[`AGENTS.md`](../AGENTS.md)**:

1. **Naming Standard**:
   - Zero prohibited abbreviations (`repo`, `req`, `res`, `msg`, `auth`, `env`, `svc`, `val`).
   - Boolean variables must use explicit prefixes (`is`, `has`, `can`, `should`).
2. **No Hardcoded Magic Numbers or Literals**:
   - Every color, duration, or dimension must originate from design tokens (`--gv-*`).
   - Every string must be localizable via `$t('...')`.
3. **Zero-CLS Skeletons**:
   - Never use generic loading spinners or text for panels or lists. Use `USkeleton` matching the exact layout.
4. **Opaque Modal Surface Contract**:
   - Modals must have `bg-[var(--gv-surface)]`, elevation border, and header/footer structure.
5. **Rust Safety**:
   - Never use `.unwrap()` or `.expect()` in production paths. Propagate typed errors with `thiserror`.
   - Never execute interpolated shell strings. Use argument vectors.

---

## 4. Git Commit Conventions

GitVerse follows [Conventional Commits](https://www.conventionalcommits.org/):

- `feat(scope): ...` — New user-facing feature or enhancement
- `fix(scope): ...` — Bug fix or regression patch
- `docs(scope): ...` — Documentation updates
- `refactor(scope): ...` — Code restructuring without behavior changes
- `test(scope): ...` — Adding or updating test suites
- `chore(scope): ...` — Dependency bumps, build script changes

---

## 5. Pre-Commit Verification Checklist

Before pushing any branch or submitting a pull request, run:

```bash
# Validate translation keys
npm run validate:i18n

# Run frontend test suite
npm run test

# Run Rust unit and integration tests
cargo test --manifest-path src-tauri/Cargo.toml

# Verify production build
npm run build
```
