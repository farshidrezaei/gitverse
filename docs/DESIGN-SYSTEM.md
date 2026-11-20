# GitVerse — Design System Specification (`Git Ember & Cosmic Slate v1.0`)

## 1. Visual Identity & Principles

GitVerse uses a developer-focused, technical, high-contrast visual identity:
- **Cosmic Slate foundations** (Dark: `#0D1117` background, `#161B22` surface; Light: `#F6F8FA` background, `#FFFFFF` surface).
- **Git Ember accents** (`#EA580C` / `#FB923C` brand primary) reflecting the fiery energy of Git branches, commits, and version trees.
- **Customizable Themes**: Full user color personalization supported through CSS custom property overrides (`--ui-primary`, `--gv-accent`, `--gv-lane-*`), allowing developers to craft their own visual workspaces.
- **Dual-theme parity**: Light and Dark palettes are independently calibrated. Dark mode is **never** an inverted light mode.
- **Bilingual parity**: English (`en`, LTR) and Persian (`fa`, RTL) receive equal typographic care using `IBM Plex Sans`, `Vazirmatn`, and `JetBrains Mono`.
- **Nuxt UI First**: Official Nuxt UI 4 components (`UApp`, `UButton`, `UBadge`, `UInput`, `USelect`, `UTable`, `UModal`, `UTabs`, `UDropdownMenu`, etc.) mapped via Tailwind CSS v4 `@theme` and `app.config.ts`.

---

## 2. Semantic Color Tokens

### Brand Roles (`app.config.ts` & `app/assets/css/main.css`)
- **Brand Primary / Git Ember**:
  - Light theme: `primary` `#EA580C`, `primary-hover` `#C2410C`, `primary-active` `#9A3412`, `primary-soft` `#FFF7ED`, `secondary` `#F97316`, `accent` `#0284C7`, `accent-soft` `#F0F9FF`
  - Dark theme: `primary` `#FB923C`, `primary-hover` `#FDBA74`, `primary-active` `#F97316`, `primary-soft` `#381C06`, `secondary` `#F97316`, `accent` `#38BDF8`, `accent-soft` `#082F49`

### Surfaces & Borders
| Role | Light (`GitVerse Light`) | Dark (`GitVerse Dark`) | Token |
| --- | --- | --- | --- |
| Application Background | `#F6F8FA` | `#0D1117` | `--gv-background` |
| Sidebar Background | `#EFF2F5` | `#090D13` | `--gv-sidebar` |
| Primary Surface | `#FFFFFF` | `#161B22` | `--gv-surface` |
| Elevated Surface | `#FFFFFF` | `#21262D` | `--gv-surface-elevated` |
| Hover Surface | `#F3F4F6` | `#30363D` | `--gv-surface-hover` |
| Active Surface | `#E5E7EB` | `#383F47` | `--gv-surface-active` |
| Modal Overlay | `#0D1117` | `#010409` | `--gv-overlay` |
| Border / Subtle / Strong | `#D0D7DE` / `#E1E4E8` / `#AFB8C1` | `#30363D` / `#21262D` / `#484F58` | `--ui-border`, `--gv-border-subtle`, `--gv-border-strong` |
| Focus Indicator | `#EA580C` | `#FB923C` | `--gv-border-focus` |

### Git Status Indicators
| Status | Light Theme | Dark Theme | Token |
| --- | --- | --- | --- |
| Staged Changes | `#1A7F37` | `#3FB950` | `--gv-git-staged` |
| Unstaged Changes | `#9A6700` | `#D29922` | `--gv-git-unstaged` |
| Conflicted Files | `#CF222E` | `#F85149` | `--gv-git-conflicted` |
| Untracked Files | `#57606A` | `#8B949E` | `--gv-git-untracked` |
| Current HEAD | `#EA580C` | `#FB923C` | `--gv-git-head` |
| Tag Decoration | `#8250DF` | `#A371F7` | `--gv-git-tag` |

### Diff Highlighting
| Element | Light Theme | Dark Theme | Token |
| --- | --- | --- | --- |
| Added Line Background | `#DAFBE1` | `rgba(46, 160, 67, 0.18)` | `--gv-diff-add-bg` |
| Added Line Text / Border | `#1A7F37` / `#ACEEBB` | `#3FB950` / `rgba(46, 160, 67, 0.40)` | `--gv-diff-add-text` |
| Removed Line Background | `#FFEBE9` | `rgba(248, 81, 73, 0.18)` | `--gv-diff-remove-bg` |
| Removed Line Text / Border | `#CF222E` / `#FFC1C0` | `#F85149` / `rgba(248, 81, 73, 0.40)` | `--gv-diff-remove-text` |
| Hunk Header Background | `#DDF4FF` | `rgba(56, 139, 253, 0.15)` | `--gv-diff-hunk-bg` |
| Hunk Header Text | `#0969DA` | `#58A6FF` | `--gv-diff-hunk-text` |

### Commit Graph Lane Palette (Deterministic Visualization)
| Lane | Light Theme | Dark Theme | Role | Token |
| --- | --- | --- | --- | --- |
| Lane 0 | `#EA580C` | `#FB923C` | Main / Primary branch | `--gv-lane-0` |
| Lane 1 | `#0891B2` | `#22D3EE` | Feature branch 1 | `--gv-lane-1` |
| Lane 2 | `#16A34A` | `#4ADE80` | Feature branch 2 | `--gv-lane-2` |
| Lane 3 | `#8B5CF6` | `#A78BFA` | Feature branch 3 | `--gv-lane-3` |
| Lane 4 | `#E11D48` | `#FB7185` | Hotfix / Release | `--gv-lane-4` |
| Lane 5 | `#D97706` | `#FBBF24` | Staging / Pre-release | `--gv-lane-5` |
| Lane 6 | `#2563EB` | `#60A5FA` | Long-lived branch | `--gv-lane-6` |
| Lane 7 | `#4F46E5` | `#818CF8` | Integration branch | `--gv-lane-7` |

---

## 3. Typography System

- **UI Sans (LTR)**: `'IBM Plex Sans Variable', 'Vazirmatn Variable', system-ui, sans-serif`
- **UI Sans (RTL / Persian)**: `'Vazirmatn Variable', 'IBM Plex Sans Variable', system-ui, sans-serif`
- **Monospace (Commit SHAs, Diffs, Branch Names, File Paths)**: `'JetBrains Mono Variable', ui-monospace, monospace`
- **Scale**: Display (`1.5rem` / 24px), Heading (`1.25rem` / 20px), Section (`1rem` / 16px), Body (`0.875rem` / 14px), Label (`0.8125rem` / 13px), Caption/Meta (`0.75rem` / 12px).

---

## 4. Spacing, Radius, Shadow, Sizing & Motion Tokens

- **Spacing Scale (4px base)**: Steps `0, 1, 2, 3, 4, 6, 8, 10, 12, 16, 20, 24` (`0px` through `96px`).
- **Border Radius Scale**:
  - `none`: `0px` (`--gv-radius-none`)
  - `sm`: `6px` (`--gv-radius-sm`)
  - `md`: `10px` (`--gv-radius-md`)
  - `lg`: `14px` (`--gv-radius-lg`)
  - `xl`: `16px` (`--gv-radius-xl`)
  - `2xl`: `20px` (`--gv-radius-2xl`)
  - `full`: `9999px` (`--gv-radius-full`)
- **Shadow Scale**: Independent Light & Dark calibrations (`--gv-shadow-none`, `--gv-shadow-sm`, `--gv-shadow-md`, `--gv-shadow-lg`, `--gv-shadow-xl`, `--gv-shadow-focus`).
- **Component Dimensions**:
  - Control heights: `sm` (`30px`), `md` (`36px`), `lg` (`42px`)
  - Input height: `38px` (`--gv-input-height`)
  - Sidebar width: `256px` (`--gv-sidebar-width`), collapsed: `64px` (`--gv-sidebar-collapsed-width`)
- **Motion Tokens**:
  - `--gv-duration-fast: 120ms`, `--gv-duration-standard: 180ms`, `--gv-duration-slow: 280ms`
  - Mandatory `@media (prefers-reduced-motion: reduce)` override.

---

## 5. Zero-CLS Skeleton Loading Standard

- **No Generic Spinners for Content**: Modal bodies, sidebars, viewport panels, cards, and list views must never render plain spinner icons or "Loading..." text strings.
- **High-Fidelity Skeletons**: Every asynchronous loading state must be rendered using `USkeleton` primitives configured to exact dimensions, borders, and margins matching the resolved content.
- **Layout Shift Immunity**: Skeletons must prevent Cumulative Layout Shift (CLS), visual flicker, and structural jumps when async data resolves.
- **Button-Only Spinners**: Spinners are strictly permitted only inside action buttons (`UButton :loading="isSubmitting"`) during active user-triggered submissions.

---

## 6. Living Showcase (`/design-system`)

The `/design-system` route serves as the living source of truth displaying:
1. Color tokens (brand, surfaces, Git statuses, diff highlights, commit graph lanes).
2. Live interactive theme customizer with real-time CSS variable re-binding.
3. Typography scale (`en` + `fa` + monospace).
4. Spacing, Radius, Shadow, and Motion demos.
5. Core Nuxt UI components across states (default, hover, focus, active, disabled, loading, error, success).
6. Live interactive **Light / Dark** and **English (LTR) / Persian (RTL)** verification switchers.
