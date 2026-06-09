---
name: "Context Build"
description: Analyze the project and generate CLAUDE.md files optimized for Claude tooling
category: Project Setup
tags: [context, claude-md, onboarding]
---

Analyze this project and generate CLAUDE.md files optimized for Claude tooling.

I'll scan the codebase structure, detect the tech stack and conventions, show you a plan, then generate concise context files.

**Input**: Optionally specify a subdirectory path to scope the analysis. Defaults to workspace root.

---

## Phase 1 — Detect

Scan the project to build a complete picture. Use Glob, Grep, and Read tools — not Bash. Run detection in parallel where possible.

### 1.1 Root manifest scan

Read these files if they exist at the workspace root (use Read tool, skip gracefully if missing):

**Language/runtime manifests:**
- `package.json` — extract: `name`, `scripts`, `workspaces`, key `dependencies`/`devDependencies`
- `Cargo.toml` — extract: `[workspace].members`, `[package].name`, `[dependencies]`
- `go.mod` — extract: `module` path, key `require` entries
- `pyproject.toml` — extract: `[project].name`, `[tool.poetry]`, `[build-system]`
- `Gemfile` — extract: key gems
- `build.gradle` / `build.gradle.kts` — extract: plugins, dependencies

**Monorepo orchestrators:**
- `nx.json` — extract: `plugins`, `targetDefaults`
- `turbo.json` — extract: `pipeline` keys
- `pnpm-workspace.yaml` — extract: `packages` globs
- `lerna.json` — extract: `packages`
- `go.work` — extract: `use` directives

**Build/command sources:**
- `Makefile` — extract: target names (lines matching `^[a-zA-Z_-]+:`)
- `.github/workflows/*.yml` — extract: job names and `run` steps (glob first, read if found)

### 1.2 Monorepo enumeration

If any workspace/monorepo config was detected, enumerate sub-projects:
- Resolve workspace member globs to actual directories
- For each sub-project directory, read its `package.json`, `Cargo.toml`, or `project.json`
- Extract: name, available scripts/targets, language, key dependencies
- Note which sub-projects have a meaningfully different stack from the root (different language, different framework)

### 1.3 Pattern discovery

Use Glob to find:
- Test configs: `**/vitest.config.*`, `**/jest.config.*`, `**/playwright.config.*` (maxdepth 3)
- Test files: sample of `**/*.test.*`, `**/*.spec.*`, `**/*_test.*` (head 10)
- Existing context: `**/CLAUDE.md`, `**/README.md` (maxdepth 2)
- Containers: `**/Dockerfile`, `**/docker-compose.*` (maxdepth 2)
- Lint/format configs: `.eslintrc*`, `biome.json`, `.prettierrc*`, `rustfmt.toml`, `clippy.toml`

Use Bash `ls` at depth 1-2 from root to understand the directory layout.

### 1.4 Existing context check

If any `CLAUDE.md` files exist:
- Read them fully
- Identify which sections match the standard template headings (Tech Stack, Project Structure, Build & Run, Testing, Key Conventions, Architecture)
- Identify hand-written sections (headings that don't match template) — these will be preserved

---

## Phase 2 — Plan

Present findings to the user before writing anything.

**Show:**
1. Detected tech stack summary (languages, frameworks, build tools, test frameworks)
2. Workspace layout (monorepo with N sub-projects, or single project)
3. List of CLAUDE.md files to generate/update:
   - Root `CLAUDE.md` — always
   - Sub-project `CLAUDE.md` — only for sub-projects with meaningfully different stacks (e.g., Rust app in a TypeScript monorepo). NOT every sub-project.
4. For each file: which sections will be included
5. If updating existing CLAUDE.md: which sections will be refreshed vs preserved

**Use AskUserQuestion** with options:
- "Looks good, generate"
- "Add/remove files" (open-ended follow-up)
- "Skip" (abort)

**Do NOT proceed to Phase 3 without explicit user approval.**

---

## Phase 3 — Generate

Write CLAUDE.md files using these templates. Conciseness is a hard constraint.

### Root CLAUDE.md template (target: under 80 lines)

```markdown
# <project-name>

<1-2 sentence description: what this is and what it does>

## Tech Stack

- **Languages**: <e.g., Rust, TypeScript>
- **Frameworks**: <e.g., Axum, React, Astro>
- **Build**: <e.g., Cargo, Nx, Vite>
- **Test**: <e.g., cargo test, Vitest, Playwright>
- **Lint**: <e.g., Clippy, ESLint, Prettier>

## Project Structure

\```
<directory tree at appropriate depth>
<monorepo: show apps/ and libs/ with 1-line descriptions>
<single repo: show src/ layout>
\```

## Build & Run

| Task | Command |
|------|---------|
| Build | `<extracted from manifests>` |
| Dev | `<extracted from manifests>` |
| Test | `<extracted from manifests>` |
| Lint | `<extracted from manifests>` |

## Testing

- **Unit**: <framework>, `<command>`, tests in `<location>`
- **Integration**: <framework>, `<command>`
- **E2E**: <framework>, `<command>` (if applicable)

## Key Conventions

- <non-obvious convention 1>
- <non-obvious convention 2>
- ...

## Architecture

\```
<ASCII diagram showing how components interact>
<only include if 3+ interacting components>
\```
```

### Sub-project CLAUDE.md template (target: under 40 lines)

```markdown
# <name>

<1 sentence: role within the larger system>

## Commands

| Task | Command |
|------|---------|
| Build | `<command>` |
| Test | `<command>` |

## Conventions

- <only what differs from root>
```

### Generation rules

- **Extract commands from files** — read `package.json` scripts, `Makefile` targets, `project.json` targets. Never guess commands.
- **If no build commands found**, use AskUserQuestion: "I couldn't detect build commands. What commands do you use to build, test, and run this project?"
- **Existing CLAUDE.md**: preserve hand-written sections (headings that don't match template). Update template-matching sections with fresh data.
- **Never include**: secrets, env var values, credentials, lengthy API docs, changelog, installation guides for end users
- **Omit empty sections** — if there's nothing meaningful for a section, skip it entirely
- **Sub-project CLAUDE.md**: only generate for sub-projects with a meaningfully different stack. A React app in a React monorepo doesn't need its own file. A Rust API in a TypeScript monorepo does.
- **Large monorepos (20+ sub-projects)**: group by category in root structure section, only generate sub-project files the user explicitly requests

---

## Phase 4 — Verify

After writing:
1. Read back each generated file to confirm it was written correctly
2. Show summary: files created/updated with line counts
3. Note: "Re-run `/context-build` after major structural changes to keep context fresh."

---

## Guardrails

- **Never write without showing the plan first** — Phase 2 approval is mandatory
- **Never duplicate README.md verbatim** — CLAUDE.md is for Claude, README is for humans
- **Check .gitignore** — if `CLAUDE.md` is gitignored, warn the user
- **Idempotent** — safe to re-run. Preserves hand-written sections, refreshes detected sections.
- **No emojis** in generated files unless the user explicitly requests them
