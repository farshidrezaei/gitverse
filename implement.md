# Git Client — Technical Specification & Implementation Backlog

**Document status:** Initial Technical Baseline  
**Product stage:** Pre-development  
**Release target:** v1.0  
**Distribution:** Portable Desktop Application  
**License:** To be selected before public release  
**Primary platforms:** Linux, Windows, macOS

---

# 1. Product Definition

## 1.1 Vision

Build a modern, cross-platform, open-source Git client that combines powerful Git operations, advanced visual history exploration, and GitHub/GitLab integrations in one cohesive desktop experience.

The product must be approachable for developers learning Git while providing advanced workflows for experienced engineers.

## 1.2 Product Principles

| Principle | Requirement |
|---|---|
| Local-first | Git operations must work without an internet connection |
| Git-native | Native Git behavior is the source of truth |
| Progressive complexity | Advanced features must not overwhelm new users |
| Open-source | Core functionality must remain fully open-source |
| Portable | No mandatory account, backend, or cloud service |
| Cross-platform | Linux, Windows, and macOS |
| Safety-first | Destructive operations must be explicit and recoverable where possible |
| Performance | Large repositories must not freeze the UI |

## 1.3 Target Users

- Developers new to Git
- Individual software engineers
- Senior and staff engineers
- Open-source contributors
- Teams using GitHub and GitLab
- Developers managing multiple repositories

## 1.4 Product Differentiators

1. Custom, interactive Commit Graph.
2. Advanced Diff and Staging experience.
3. Professional Git workflows without terminal dependency.
4. Unified GitHub and GitLab experience.
5. Futuristic visual identity with excellent usability.
6. Fully local-first and genuinely open-source architecture.

## 1.5 Non-Goals for v1.0

- Git server implementation
- Built-in code editor or IDE
- Cloud synchronization
- Hosted collaboration backend
- Team account management
- Plugin marketplace
- AI-powered code review
- Bitbucket and other provider integrations
- Git implementation from scratch

---

# 2. Technical Stack

| Layer | Technology | Responsibility |
|---|---|---|
| Desktop Runtime | Tauri 2 | Native desktop application |
| Frontend | Nuxt 4 | Application UI |
| Framework | Vue 3 | Component system |
| Language | TypeScript | Frontend type safety |
| Styling | Tailwind CSS 4 | Design system implementation |
| UI Components | Nuxt UI | Base components |
| Native Backend | Rust | Application core |
| Git Engine | Native Git CLI | Git operations |
| Persistence | SQLite | Application metadata |
| Credentials | OS Keychain | Secure credential storage |
| Provider APIs | REST / GraphQL | GitHub and GitLab |
| Graph Rendering | Custom SVG renderer | Commit graph visualization |
| Testing | Rust tests + Vitest + Playwright | Automated quality assurance |

## 2.1 Architectural Decisions

### ADR-001: Native Git CLI

Native Git CLI is the source of truth for repository operations.

Reasons:

- Compatibility with installed Git versions.
- Access to advanced Git functionality.
- Predictable behavior matching terminal workflows.
- Reduced implementation complexity.
- No need to recreate Git internals.

Implementation requirements:

- Never construct shell command strings from user-controlled values.
- Execute processes using argument arrays.
- Use structured Git output wherever possible.
- Handle process cancellation and exit codes.
- Support Git executable discovery across operating systems.
- Never parse localized, human-oriented output when a machine-readable alternative exists.

### ADR-002: Custom Commit Graph

The application will own the graph layout and rendering implementation.

The graph is not a wrapper around a third-party Git graph component.

Responsibilities:

- Commit node representation
- Parent-child relationships
- Branch and tag decorations
- Lane assignment
- Merge visualization
- Zoom and pan
- Selection and navigation
- Virtualized rendering
- Graph-to-commit interaction

Git remains responsible for retrieving commit history and relationships.

### ADR-003: Local-first Architecture

No application-owned backend is required.

All local repository operations must function independently of GitHub and GitLab.

Provider integrations are optional capabilities layered on top of local Git repositories.

### ADR-004: Provider Abstraction

GitHub and GitLab must use independent adapters implementing shared application contracts.

Provider-specific capabilities must not be artificially normalized into features that do not exist on every provider.

---

# 3. System Architecture

```text
┌──────────────────────────────────────────┐
│             Nuxt 4 Application           │
│                                          │
│ Workspace · Changes · History · Remote  │
│ Command Palette · Settings · Components │
└────────────────────┬─────────────────────┘
                     │
               Tauri IPC
                     │
┌────────────────────▼─────────────────────┐
│               Rust Core                  │
│                                          │
│  Application Commands                    │
│  Repository Manager                      │
│  Git Engine                              │
│  Diff Engine                             │
│  Commit Graph Engine                     │
│  Provider Core                           │
│  Credential Manager                      │
│  Workspace Store                         │
│  Operation Manager                       │
│  Event Dispatcher                        │
└─────────────┬──────────────────┬─────────┘
              │                  │
      ┌───────▼────────┐  ┌──────▼──────────┐
      │ Native Git CLI │  │ Provider APIs   │
      │                │  │                │
      │ Git Repository │  │ GitHub         │
      │ SSH            │  │ GitLab         │
      │ Filesystem     │  │ OAuth          │
      └────────────────┘  └─────────────────┘
              │
      ┌───────▼────────┐
      │ Local Storage  │
      │                │
      │ SQLite         │
      │ OS Keychain    │
      │ Preferences    │
      └────────────────┘
```

## 3.1 Repository Structure

```text
git-client/
├── apps/
│   └── desktop/
│       ├── app/
│       │   ├── components/
│       │   ├── composables/
│       │   ├── features/
│       │   │   ├── workspace/
│       │   │   ├── repositories/
│       │   │   ├── changes/
│       │   │   ├── history/
│       │   │   ├── graph/
│       │   │   ├── remote/
│       │   │   └── settings/
│       │   ├── layouts/
│       │   ├── pages/
│       │   ├── stores/
│       │   └── types/
│       │
│       └── src-tauri/
│           ├── src/
│           │   ├── commands/
│           │   ├── events/
│           │   ├── application/
│           │   └── main.rs
│           └── capabilities/
│
├── crates/
│   ├── git-engine/
│   ├── repository-manager/
│   ├── diff-engine/
│   ├── graph-engine/
│   ├── provider-core/
│   ├── provider-github/
│   ├── provider-gitlab/
│   ├── credential-manager/
│   ├── workspace-store/
│   └── operation-manager/
│
├── packages/
│   ├── ui/
│   ├── git-types/
│   └── provider-types/
│
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── development/
│   └── release/
│
├── scripts/
├── .github/
└── README.md
```

Note: This is a logical module structure. Avoid splitting everything into independently published packages before there is a practical need.

---

# 4. Core Domain Models

These are initial conceptual contracts. Final Rust and TypeScript types should be defined during implementation.

## 4.1 Repository

```ts
interface Repository {
  id: string
  path: string
  name: string
  currentBranch: string | null
  headCommit: string | null
  isBare: boolean
  isShallow: boolean
  remotes: Remote[]
  lastOpenedAt: number
}
```

## 4.2 Commit

```ts
interface Commit {
  sha: string
  parents: string[]
  author: {
    name: string
    email: string
    timestamp: number
  }
  committer: {
    name: string
    email: string
    timestamp: number
  }
  subject: string
  body: string
}
```

## 4.3 Reference

```ts
type ReferenceType =
  | 'local-branch'
  | 'remote-branch'
  | 'tag'
  | 'head'

interface GitReference {
  name: string
  fullName: string
  commitSha: string
  type: ReferenceType
}
```

## 4.4 File Change

```ts
type ChangeStatus =
  | 'added'
  | 'modified'
  | 'deleted'
  | 'renamed'
  | 'copied'
  | 'untracked'
  | 'conflicted'

interface FileChange {
  path: string
  originalPath?: string
  status: ChangeStatus
  staged: boolean
  unstaged: boolean
  isBinary: boolean
}
```

## 4.5 Provider Account

```ts
interface ProviderAccount {
  id: string
  provider: 'github' | 'gitlab'
  username: string
  displayName?: string
  avatarUrl?: string
  instanceUrl: string
}
```

---

# 5. Functional Requirements

## 5.1 Repository Management

- Open existing repository.
- Add repository to workspace.
- Remove repository from workspace without deleting files.
- Show recent repositories.
- Detect invalid or unavailable repository paths.
- Detect repository state changes.
- Support multiple open repositories.
- Support repository groups or workspaces.

## 5.2 Git Operations

### Essential

- Status
- Fetch
- Pull
- Push
- Commit
- Branch creation and deletion
- Branch checkout
- Merge
- Remote management
- Tag management
- Stash and apply stash

### Advanced

- Interactive Rebase
- Cherry-pick
- Revert
- Amend
- Squash
- Reset
- Reflog
- Worktree
- Bisect

All operations must return structured results, not raw command output.

## 5.3 Changes and Diff

- Staged and unstaged changes.
- File-level staging.
- Hunk-level staging.
- Line-level staging.
- Inline and side-by-side diff.
- Binary file detection.
- Rename detection.
- Syntax highlighting.
- Word-level differences.
- Conflict visualization.
- Image comparison as a later enhancement.

## 5.4 Commit Graph

- Render commit history.
- Show parent relationships.
- Support merge commits.
- Display local and remote branches.
- Display tags.
- Display HEAD.
- Select commits.
- Show commit details.
- Zoom and pan.
- Filter by branch.
- Navigate to file history.
- Compare selected commits.
- Incrementally load older history.

## 5.5 GitHub Integration

- Authentication.
- Account management.
- Repository discovery.
- Pull request list.
- Pull request details.
- Review comments.
- Submit reviews.
- Create pull requests.
- CI status.
- Issue list and creation.
- Release information.

## 5.6 GitLab Integration

- Authentication.
- Account management.
- Project discovery.
- Merge request list.
- Merge request details.
- Discussions.
- Submit reviews or approvals where supported.
- Create merge requests.
- Pipeline status.
- Issue list and creation.
- Release information.

## 5.7 Application Experience

- Command Palette.
- Keyboard shortcuts.
- Dark and light themes.
- Configurable sidebar.
- Multiple repository tabs.
- Notifications.
- Search.
- Settings.
- Portable configuration option.
- Native OS integrations.

---

# 6. Custom Commit Graph Specification

This is one of the most important engineering components.

## 6.1 Data Pipeline

```text
Git Repository
      │
      ▼
History Extraction
      │
      ▼
Commit Normalization
      │
      ▼
DAG Construction
      │
      ▼
Lane Assignment
      │
      ▼
Graph Layout
      │
      ▼
Viewport Calculation
      │
      ▼
Custom Renderer
```

## 6.2 Graph Data

The graph engine should operate independently of the UI framework.

```ts
interface GraphNode {
  sha: string
  parents: string[]
  lane: number
  row: number
  references: GraphReference[]
  isHead: boolean
}

interface GraphEdge {
  from: string
  to: string
  type: 'first-parent' | 'merge-parent'
}

interface GraphReference {
  name: string
  type: 'branch' | 'tag' | 'remote'
}
```

## 6.3 Layout Requirements

- Stable lane assignment.
- Correct representation of merge commits.
- First-parent relationships must be visually distinguishable.
- Additional merge parents must be visible.
- Branch decorations must not overlap unnecessarily.
- Graph layout must be deterministic for the same input.
- New history must not unnecessarily rearrange existing visible nodes.

## 6.4 Rendering Strategy

Initial approach:

- Custom SVG renderer.
- Viewport-aware rendering.
- Avoid mounting every commit as a Vue component.
- Render nodes and edges in a controlled rendering layer.
- Keep selection, hover and accessibility controls separate from graph geometry.

If profiling demonstrates that SVG cannot meet performance requirements, evaluate Canvas or WebGL for the rendering layer without changing the graph data model.

## 6.5 Performance Requirements

Initial targets:

- 2,000 commits: smooth interaction.
- 10,000 commits: incremental loading and virtualization.
- No synchronous graph computation on the UI thread.
- Pan and zoom must remain responsive.
- Selecting a commit must not trigger a full graph rebuild.

These are engineering targets, not guarantees. Benchmark using generated repositories and real-world large repositories.

---

# 7. Security and Reliability

## 7.1 Process Execution

- Never invoke shell commands through string interpolation.
- Use argument arrays.
- Validate repository paths.
- Handle timeouts and cancellation.
- Sanitize logs.
- Avoid leaking credentials through process arguments.
- Handle unusual filenames, spaces and Unicode paths.
- Handle process failures and partial output.

## 7.2 Credentials

- Store tokens in the operating system's secure credential store.
- Never store plaintext tokens in SQLite.
- Support token revocation and account disconnection.
- Avoid logging authorization headers.
- Use appropriate OAuth flows for native applications.
- Respect GitHub and GitLab API rate limits.

## 7.3 Destructive Operations

Operations such as Reset, Clean, Force Push and branch deletion must:

- Explain their impact.
- Show affected files or references when feasible.
- Require explicit confirmation.
- Never silently discard user changes.
- Offer recovery guidance when Git provides a recovery mechanism.

## 7.4 Repository Integrity

- Git is always the source of truth.
- Detect external Git operations.
- Refresh state after external modifications.
- Avoid concurrent conflicting mutations.
- Never assume that a previously fetched repository state is still current.

---

# 8. Portable Distribution Specification

Portable distribution must not be confused with zero-dependency execution.

The application should support:

| Platform | Initial distribution |
|---|---|
| Windows | Portable executable package |
| Linux | AppImage and archive |
| macOS | Portable application bundle and archive |

Requirements:

- Detect installed Git.
- Allow manual Git executable configuration.
- Display Git version.
- Provide clear errors when Git is unavailable.
- Keep application data separate from repository data.
- Support configurable portable data location.
- Never write application settings into repository directories by default.
- Support offline operation.

macOS notarization and signing, and platform-specific security requirements, must be addressed in release engineering. An application bundle is not automatically free of platform trust requirements.

---

# 9. Non-Functional Requirements

| Category | Requirement |
|---|---|
| Performance | No blocking Git operations on the UI thread |
| Reliability | Operation failures must not corrupt application state |
| Accessibility | Keyboard navigation and accessible labels |
| Security | Secure credential storage |
| Maintainability | Typed IPC contracts |
| Testing | Automated tests for critical Git operations |
| Portability | Platform-aware process and filesystem handling |
| UX | Consistent keyboard and mouse interaction |
| Observability | Structured logs without secrets |
| Localization | English first; extensible i18n architecture |

---

# 10. Implementation Backlog

Issue labels:

- `type:feature`
- `type:infra`
- `type:bug`
- `type:design`
- `type:security`
- `type:docs`
- `area:frontend`
- `area:backend`
- `area:git`
- `area:graph`
- `area:provider`
- `priority:p0`
- `priority:p1`
- `priority:p2`

Effort estimates use relative engineering days, not guaranteed calendar durations.

---

# EPIC 00 — Brand & Product Foundation

## GC-001 — Define Product Brand Identity

**Labels:** `type:design`, `priority:p0`

**Estimate:** 2–4 days

### Scope

- Define product name.
- Define logo and icon system.
- Define color palette.
- Define typography.
- Define visual direction.
- Define design principles.
- Create brand assets.

### Acceptance Criteria

- Product name is approved.
- Logo works in light and dark environments.
- Application icon is available in required platform sizes.
- Brand colors and typography are documented.
- Design assets are available in the repository.

## GC-002 — Establish UI Design System

**Labels:** `type:design`, `area:frontend`, `priority:p0`

**Estimate:** 3–5 days

### Scope

- Define spacing and sizing tokens.
- Define semantic colors.
- Define typography scale.
- Define component states.
- Define dark and light themes.
- Define motion and transition guidelines.

### Acceptance Criteria

- Shared design tokens are implemented.
- Theme switching works.
- Buttons, inputs, dialogs, menus and tooltips follow consistent patterns.
- Keyboard focus states are visible.
- Reduced-motion preferences are respected.

## GC-003 — Initialize Monorepo and Development Standards

**Labels:** `type:infra`, `priority:p0`

**Estimate:** 2–3 days

### Scope

- Initialize Nuxt and Tauri.
- Initialize Rust workspace.
- Establish frontend and backend linting.
- Configure formatting.
- Configure development scripts.
- Establish contribution guidelines.

### Acceptance Criteria

- Application runs in development mode.
- Rust workspace builds.
- Type checking works.
- Formatting and lint commands are documented.
- CI validates basic build and quality checks.

---

# EPIC 01 — Desktop Core

## GC-004 — Implement Native Git Discovery

**Labels:** `type:feature`, `area:backend`, `area:git`, `priority:p0`

**Estimate:** 2 days

### Scope

- Discover Git executable.
- Support platform-specific discovery.
- Detect Git version.
- Support manual executable configuration.

### Acceptance Criteria

- Git is discovered on supported platforms.
- Missing Git produces an actionable error.
- Invalid executable paths are rejected.
- Git version is exposed to the frontend.

## GC-005 — Implement Safe Process Execution Layer

**Labels:** `type:infra`, `area:backend`, `area:security`, `priority:p0`

**Estimate:** 3–4 days

### Scope

- Process execution using argument arrays.
- Working-directory management.
- Standard output and error handling.
- Cancellation.
- Timeouts.
- Exit code handling.
- Structured execution results.

### Acceptance Criteria

- No shell interpolation is used.
- Paths containing spaces and Unicode work.
- Processes can be cancelled.
- Errors are structured.
- Sensitive data is excluded from logs.

## GC-006 — Implement Typed Tauri IPC Contracts

**Labels:** `type:infra`, `area:frontend`, `area:backend`, `priority:p0`

**Estimate:** 2–3 days

### Scope

- Define command conventions.
- Define response envelopes.
- Define error contracts.
- Define event payload conventions.
- Establish frontend/backend type synchronization.

### Acceptance Criteria

- No untyped application command responses.
- Errors have stable codes and user-facing messages.
- Long-running operations can emit progress events.
- IPC contracts are documented.

## GC-007 — Implement Repository Discovery and Management

**Labels:** `type:feature`, `area:git`, `area:backend`, `priority:p0`

**Estimate:** 3–4 days

### Scope

- Open repository by path.
- Validate repository.
- Detect bare repositories.
- Discover repository metadata.
- Maintain recent repositories.
- Remove repository from application workspace.

### Acceptance Criteria

- Valid repositories can be opened.
- Invalid paths are handled safely.
- Removing a repository does not delete its files.
- Recent repositories survive application restarts.

## GC-008 — Implement Application Persistence

**Labels:** `type:infra`, `area:backend`, `priority:p0`

**Estimate:** 2–3 days

### Scope

- Initialize SQLite.
- Store application preferences.
- Store repository metadata.
- Store recent repository history.
- Implement schema migrations.

### Acceptance Criteria

- Database initialization is reliable.
- Schema migrations are versioned.
- Repository data is not duplicated unnecessarily.
- Database corruption produces a recoverable error.

---

# EPIC 02 — Git Essentials

## GC-009 — Implement Repository Status

**Labels:** `type:feature`, `area:git`, `priority:p0`

**Estimate:** 2–3 days

### Scope

- Current branch.
- HEAD state.
- Staged changes.
- Unstaged changes.
- Untracked files.
- Conflict state.
- Ahead/behind information where available.

### Acceptance Criteria

- Status is obtained using machine-readable Git output.
- Renamed and conflicted files are represented correctly.
- Unusual filenames are handled.
- External repository changes can be detected.

## GC-010 — Implement Changes and Staging UI

**Labels:** `type:feature`, `area:frontend`, `area:git`, `priority:p0`

**Estimate:** 3–5 days

### Scope

- File change list.
- Staged and unstaged sections.
- Stage and unstage file.
- Stage and unstage hunk.
- Refresh status.

### Acceptance Criteria

- File status is visually distinguishable.
- Staging operations update the UI.
- Partial staging is supported.
- Conflicted files cannot accidentally be committed as resolved.

## GC-011 — Implement Diff Engine

**Labels:** `type:feature`, `area:backend`, `area:frontend`, `priority:p0`

**Estimate:** 4–6 days

### Scope

- Parse Git diff output.
- Represent files and hunks.
- Support staged and unstaged diffs.
- Detect binary files.
- Handle renamed files.
- Support line-level operations.

### Acceptance Criteria

- Diff output is structured.
- Large diffs do not block the UI.
- Binary files are identified.
- Hunk and line information is available to the frontend.
- Edge cases involving newlines and unusual paths are tested.

## GC-012 — Build Advanced Diff Viewer

**Labels:** `type:feature`, `type:design`, `area:frontend`, `priority:p0`

**Estimate:** 4–6 days

### Scope

- Inline diff.
- Side-by-side diff.
- Syntax highlighting.
- Word-level highlighting.
- File navigation.
- Diff filtering.

### Acceptance Criteria

- Both diff modes work.
- Large files remain usable.
- File navigation is keyboard-accessible.
- Theme colors are consistent with the design system.

## GC-013 — Implement Commit Creation

**Labels:** `type:feature`, `area:git`, `priority:p0`

**Estimate:** 2–3 days

### Scope

- Commit message editor.
- Commit creation.
- Amend support.
- Git identity validation.
- Commit result handling.

### Acceptance Criteria

- Commit can be created from staged changes.
- Empty staged changes are handled.
- Missing Git identity is explained.
- Commit details are refreshed after success.

## GC-014 — Implement Branch and Remote Management

**Labels:** `type:feature`, `area:git`, `priority:p0`

**Estimate:** 3–4 days

### Scope

- List local and remote branches.
- Create and delete branches.
- Checkout branches.
- Rename branches.
- List and configure remotes.
- Fetch, pull and push.

### Acceptance Criteria

- Branch changes are reflected immediately.
- Unsafe branch deletion is prevented or confirmed.
- Push and pull failures are explained.
- Ahead/behind information is available where applicable.

---

# EPIC 03 — Custom Commit Graph

## GC-015 — Implement Commit History Extraction

**Labels:** `type:feature`, `area:graph`, `area:backend`, `priority:p0`

**Estimate:** 3–4 days

### Scope

- Extract commit SHA.
- Extract parent SHAs.
- Extract author and committer metadata.
- Extract subject and body.
- Extract reference decorations.
- Support pagination or incremental history loading.

### Acceptance Criteria

- Merge parents are preserved.
- History is correctly represented for multiple branches.
- Shallow repositories are supported.
- Large repositories can load history incrementally.
- Commit parsing does not depend on localized output.

## GC-016 — Implement Graph Domain Model and DAG Builder

**Labels:** `type:feature`, `area:graph`, `area:backend`, `priority:p0`

**Estimate:** 3–5 days

### Scope

- Build a directed acyclic graph from commit data.
- Maintain parent-child relationships.
- Resolve references.
- Identify HEAD.
- Detect missing parents in partial history.

### Acceptance Criteria

- Graph relationships are correct.
- Merge commits have all parents.
- Missing parents do not crash graph construction.
- The model is independent of Vue components.

## GC-017 — Implement Graph Lane Assignment

**Labels:** `type:feature`, `area:graph`, `priority:p0`

**Estimate:** 4–6 days

### Scope

- Assign lanes to commit nodes.
- Route parent-child edges.
- Represent merges.
- Preserve deterministic layout.
- Minimize unnecessary lane changes.

### Acceptance Criteria

- Linear histories render correctly.
- Divergent branches render correctly.
- Merge histories render correctly.
- Layout is deterministic for identical inputs.
- Unit tests cover complex branching structures.

## GC-018 — Build Custom Graph Renderer

**Labels:** `type:feature`, `area:graph`, `area:frontend`, `priority:p0`

**Estimate:** 5–8 days

### Scope

- Custom SVG graph renderer.
- Node and edge rendering.
- Branch decorations.
- Tag decorations.
- HEAD indicator.
- Commit selection.
- Hover states.
- Theme integration.

### Acceptance Criteria

- Graph accurately reflects the DAG.
- Commit nodes are selectable.
- Branch and tag labels are readable.
- Merge edges are visually distinguishable.
- Rendering is separated from graph calculation.

## GC-019 — Implement Graph Navigation and Virtualization

**Labels:** `type:feature`, `area:graph`, `area:frontend`, `priority:p0`

**Estimate:** 4–6 days

### Scope

- Pan and zoom.
- Viewport culling.
- Incremental rendering.
- Commit selection synchronization.
- Keyboard navigation.
- History expansion.

### Acceptance Criteria

- 2,000-commit benchmark is usable.
- UI remains responsive during history expansion.
- Zoom and pan do not trigger unnecessary full graph recalculation.
- Selected commit remains synchronized with details panel.

## GC-020 — Implement Commit Details and Comparison

**Labels:** `type:feature`, `area:graph`, `area:git`, `priority:p1`

**Estimate:** 3–4 days

### Scope

- Commit details panel.
- Changed file list.
- Parent information.
- Compare two commits.
- Compare branch with base.

### Acceptance Criteria

- Selecting a commit displays metadata.
- Changed files can be inspected.
- Two commits can be compared.
- Graph selection and comparison state are synchronized.

---

# EPIC 04 — GitHub and GitLab

## GC-021 — Implement Provider Core Contracts

**Labels:** `type:infra`, `area:provider`, `priority:p0`

**Estimate:** 2–3 days

### Scope

- Shared provider interface.
- Provider capability discovery.
- Authentication contract.
- Error normalization.
- Pagination contract.
- Rate limit handling.

### Acceptance Criteria

- GitHub and GitLab adapters are independent.
- Provider-specific features can be exposed.
- API errors are normalized.
- Rate limits are represented.

## GC-022 — Implement Secure Credential Management

**Labels:** `type:feature`, `type:security`, `area:provider`, `priority:p0`

**Estimate:** 2–4 days

### Scope

- OS Keychain integration.
- Save credentials.
- Retrieve credentials.
- Revoke credentials.
- Disconnect accounts.

### Acceptance Criteria

- Credentials are not stored in plaintext.
- Credentials are not included in logs.
- Account disconnection removes stored credentials.
- Keychain failures are handled gracefully.

## GC-023 — Implement GitHub Authentication and Adapter

**Labels:** `type:feature`, `area:provider`, `priority:p0`

**Estimate:** 4–5 days

### Scope

- Authentication.
- Account profile.
- Repository discovery.
- Pagination.
- Rate limit handling.
- Repository mapping.

### Acceptance Criteria

- User can connect a GitHub account.
- User can disconnect it.
- Repositories can be discovered.
- API failures are recoverable.
- Multiple accounts can be supported.

## GC-024 — Implement GitLab Authentication and Adapter

**Labels:** `type:feature`, `area:provider`, `priority:p0`

**Estimate:** 4–5 days

### Scope

- GitLab.com authentication.
- Account profile.
- Project discovery.
- Pagination.
- Rate limit handling.
- Instance URL abstraction for future self-managed support.

### Acceptance Criteria

- User can connect GitLab.com.
- User can disconnect the account.
- Projects can be discovered.
- Provider instance configuration does not require changing provider-core contracts.

## GC-025 — Implement Pull Request and Merge Request Views

**Labels:** `type:feature`, `area:provider`, `area:frontend`, `priority:p1`

**Estimate:** 4–6 days

### Scope

- PR/MR lists.
- Detail view.
- Status and metadata.
- Changed files.
- Review status.
- Open external links.

### Acceptance Criteria

- GitHub PRs and GitLab MRs are displayed.
- Provider-specific fields are preserved.
- Loading and error states are implemented.
- Relevant local repositories can be associated with remote requests.

## GC-026 — Implement Review and Pipeline Views

**Labels:** `type:feature`, `area:provider`, `priority:p1`

**Estimate:** 4–6 days

### Scope

- Review comments.
- Submit review where supported.
- Display pipeline status.
- Display CI check results.
- Link to provider details.

### Acceptance Criteria

- Reviews can be submitted where permissions allow.
- Pipeline states are visually distinguishable.
- Unsupported actions are hidden or explicitly disabled.
- Permission errors are explained.

---

# EPIC 05 — Advanced Git

## GC-027 — Implement Stash Management

**Labels:** `type:feature`, `area:git`, `priority:p1`

**Estimate:** 2 days

## GC-028 — Implement Cherry-pick and Revert

**Labels:** `type:feature`, `area:git`, `priority:p1`

**Estimate:** 2–3 days

## GC-029 — Implement Interactive Rebase UI

**Labels:** `type:feature`, `area:git`, `priority:p1`

**Estimate:** 4–6 days

### Scope

- Pick.
- Reword.
- Edit.
- Squash.
- Fixup.
- Drop.
- Reorder commits.
- Abort and continue rebase.

### Acceptance Criteria

- Rebase plan can be edited visually.
- Conflicts are surfaced.
- Abort and continue actions work.
- User is warned about rewriting published history.

## GC-030 — Implement Conflict Resolution UI

**Labels:** `type:feature`, `area:git`, `area:frontend`, `priority:p1`

**Estimate:** 5–7 days

### Scope

- Conflict file discovery.
- Three-way merge view.
- Base, ours and theirs.
- Manual resolution.
- Mark as resolved.
- Continue or abort merge/rebase.

### Acceptance Criteria

- All conflict stages are represented.
- Resolved content can be staged.
- Git remains responsible for merge state.
- Abort and recovery flows are tested.

## GC-031 — Implement Worktree Management

**Labels:** `type:feature`, `area:git`, `priority:p2`

**Estimate:** 3–4 days

### Scope

- List worktrees.
- Create worktree.
- Remove worktree.
- Switch between worktrees.

---

# EPIC 06 — Product Experience

## GC-032 — Implement Command Palette

**Labels:** `type:feature`, `area:frontend`, `priority:p1`

**Estimate:** 2–3 days

### Scope

- Global search.
- Git command discovery.
- Keyboard shortcut integration.
- Context-aware commands.

### Acceptance Criteria

- Commands are searchable.
- Commands reflect repository state.
- Keyboard shortcuts are documented.
- Unavailable actions are not silently executed.

## GC-033 — Implement Multi-Repository Workspace

**Labels:** `type:feature`, `area:frontend`, `priority:p1`

**Estimate:** 3–4 days

### Scope

- Repository tabs.
- Repository switching.
- Recent repositories.
- Workspace persistence.
- Per-repository state.

## GC-034 — Implement Application Settings

**Labels:** `type:feature`, `area:frontend`, `priority:p1`

**Estimate:** 2–3 days

### Scope

- Theme.
- Git executable path.
- Default branch behavior.
- Diff preferences.
- Keyboard shortcuts.
- Portable data directory preferences.

## GC-035 — Implement File Watcher and External Change Detection

**Labels:** `type:feature`, `area:backend`, `priority:p1`

**Estimate:** 3–4 days

### Scope

- Watch repository changes.
- Detect external Git operations.
- Debounce events.
- Refresh affected views.
- Handle large repository directories.

### Acceptance Criteria

- Changes made in Terminal appear in the application.
- `.git` changes are monitored appropriately.
- Watcher does not create infinite refresh loops.
- Large file changes do not freeze the UI.

---

# EPIC 07 — Quality and Release

## GC-036 — Establish Git Integration Test Suite

**Labels:** `type:infra`, `priority:p0`

**Estimate:** 3–5 days

### Scope

Create fixture repositories covering:

- Linear history.
- Multiple branches.
- Merge commits.
- Rebase.
- Conflicts.
- Renames.
- Binary files.
- Shallow repositories.
- Unusual filenames.
- Detached HEAD.

### Acceptance Criteria

- Core Git operations have integration tests.
- Graph engine has deterministic test fixtures.
- Critical regressions are covered.

## GC-037 — Implement Portable Build Pipeline

**Labels:** `type:infra`, `priority:p0`

**Estimate:** 3–5 days

### Scope

- Linux builds.
- Windows builds.
- macOS builds.
- Portable artifacts.
- Artifact checksums.
- Release automation.

### Acceptance Criteria

- All target platforms have build jobs.
- Build artifacts are published.
- Checksums are generated.
- Installation and portable execution are documented.

## GC-038 — Implement Application Logging and Diagnostics

**Labels:** `type:feature`, `area:backend`, `type:security`, `priority:p1`

**Estimate:** 2–3 days

### Scope

- Structured logs.
- Log rotation.
- Diagnostic export.
- Secret redaction.
- Crash diagnostics.

### Acceptance Criteria

- No credentials appear in diagnostic exports.
- Users can export logs voluntarily.
- Logs include operation context.
- Logs do not contain unnecessary repository file contents.

## GC-039 — Prepare Public v1.0 Release

**Labels:** `type:docs`, `priority:p0`

**Estimate:** 3–4 days

### Scope

- README.
- Screenshots.
- Installation instructions.
- Feature documentation.
- Known limitations.
- Contribution guide.
- License.
- Security policy.
- Release notes.

### Acceptance Criteria

- Fresh installation is documented.
- Supported platforms are listed.
- Known limitations are explicit.
- Project contribution workflow is documented.
- License and third-party dependencies are reviewed.

---

# 11. Milestone Plan

| Milestone | Included Issues | Expected Outcome |
|---|---|---|
| M0 — Foundation | GC-001 to GC-008 | Branded application foundation |
| M1 — Git Essentials | GC-009 to GC-014 | Daily-use Git client |
| M2 — Visual History | GC-015 to GC-020 | Custom Commit Graph |
| M3 — Provider Integration | GC-021 to GC-026 | GitHub and GitLab |
| M4 — Professional Workflows | GC-027 to GC-031 | Advanced Git operations |
| M5 — Product Experience | GC-032 to GC-035 | Polished desktop UX |
| M6 — Release | GC-036 to GC-039 | Public v1.0 |

## Release Gate

v1.0 must not be released until:

- Git Essentials are stable.
- Graph handles merges and branches correctly.
- GitHub and GitLab authentication works.
- Credential storage is secure.
- No known critical data-loss bugs remain.
- Portable builds work on all target platforms.
- Critical Git integration tests pass.

---

# 12. Testing Strategy

## Rust

- Unit tests for domain logic.
- Process execution tests.
- Git integration tests.
- Graph construction tests.
- Graph layout tests.
- Persistence tests.
- Provider adapter tests.

## Frontend

- Component tests.
- State management tests.
- Graph interaction tests.
- Diff rendering tests.
- Keyboard navigation tests.

## End-to-End

- Open repository.
- Stage changes.
- Create commit.
- Create branch.
- Fetch and push.
- Inspect history.
- Authenticate provider.
- View PR/MR.
- Resolve conflict.

## Performance Benchmarks

Create generated repositories with:

- 2,000 commits.
- 10,000 commits.
- Multiple branches.
- Merge-heavy histories.
- Large diffs.
- Large numbers of changed files.

Track:

- History extraction time.
- Graph layout time.
- First render time.
- Memory usage.
- Pan and zoom responsiveness.

---

# 13. Definition of Done

An issue is considered complete when:

1. Implementation matches the stated scope.
2. Acceptance criteria pass.
3. Relevant tests exist.
4. Errors are handled.
5. Loading, empty and failure states are implemented where relevant.
6. No secrets or sensitive data are logged.
7. Documentation is updated.
8. UI follows the design system.
9. Cross-platform behavior is considered.
10. Changes pass CI.

---

# 14. Development Rules

- Prefer explicit types over implicit behavior.
- Keep Git operations in the Rust backend.
- Keep provider APIs isolated from Git Engine.
- Do not expose raw Git process output as application domain data.
- Do not let UI components execute Git commands directly.
- Do not store repository state as a second source of truth.
- Avoid premature abstractions.
- Keep graph algorithms independently testable.
- Every destructive operation must have an explicit safety contract.
- Do not introduce a dependency without evaluating its maintenance, license and platform compatibility.
- Keep the application usable without GitHub or GitLab authentication.

---

# 15. Immediate Development Order

The recommended first implementation sequence is:

1. GC-001 — Brand Identity
2. GC-002 — Design System
3. GC-003 — Monorepo Foundation
4. GC-004 — Git Discovery
5. GC-005 — Safe Process Execution
6. GC-006 — Typed IPC
7. GC-007 — Repository Management
8. GC-008 — Persistence
9. GC-009 — Repository Status
10. GC-015 — Commit History Extraction
11. GC-016 — DAG Builder
12. GC-017 — Lane Assignment
13. GC-018 — Custom Graph Renderer

Do not wait until the entire Git feature set is implemented before building the graph. The custom graph is one of the product's defining engineering investments and should be validated early.

---

**Final product direction:**

A genuinely open-source, local-first, futuristic Git desktop client with a custom history visualization engine, powerful native Git workflows, and first-class GitHub and GitLab integrations.

**Primary engineering priority:** correctness and reliability.

**Primary UX priority:** making complex Git workflows understandable without hiding their power.

**Primary product priority:** a small set of exceptionally polished core experiences rather than a large collection of mediocre features.