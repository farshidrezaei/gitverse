# GitVerse — Local Development Guide

## Prerequisites

- **Node.js**: v20 or later (or Bun / pnpm)
- **Rust**: stable toolchain 1.82+ (`rustup update stable`)
- **Git**: 2.38+ installed on system PATH
- **Tauri Prerequisites**: Standard platform build packages (`build-essential`, `libwebkit2gtk-4.1-dev`, etc. on Linux)

## Setup Commands

```bash
# 1. Install frontend dependencies
npm install

# 2. Validate translation keys
npm run validate:i18n

# 3. Start development server (Nuxt web mode)
npm run dev

# 4. Start desktop app (Tauri 2 mode)
npm run tauri dev
```
