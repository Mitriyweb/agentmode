---
description: Review code changes and create a commit following project standards
---

# Review & Commit Workflow

Review changed code for quality and compliance, then commit if all checks pass.

## Phase 0: Detect Changes

```bash
git status
git diff --name-only
git diff --stat
```

1. Identify all changed files (`.rs`, `Cargo.toml`, `.md`, `.yml`, `.sh`)
2. Read full content of each changed file
3. Categorize by type and apply matching rules:
   - `.rs` files -> Rust formatting (`cargo fmt`), clippy linting (`cargo clippy`), panic-free error handling, proper RAII & IOKit handle safety
   - `Cargo.toml` / `Cargo.lock` -> valid manifest structure, accurate dependency versions
   - `.md` files -> clean Markdown syntax, valid headings and links
   - `.yml` / `.yaml` files -> valid GitHub Actions workflow syntax
   - Agent definitions / workflows (`.md` in `.agents/`) -> valid YAML frontmatter, H1 title, clear instructions
   - All files -> no hardcoded secrets, credentials, or sensitive bot tokens (`TELEGRAM_TOKEN`, etc.)

## Phase 1: Code Review

For each changed file, check for violations and categorize:

- **Critical** - must fix before commit (broken build/syntax, secrets/tokens exposed, unhandled panics in core safety code)
- **High** - should fix before commit (clippy warnings, `cargo fmt` diffs, missing doc comments on public APIs, test failures)
- **Medium** - nice to fix (code structure, minor naming improvements)

If critical/high issues found -> fix them before proceeding.

### Run Validation Pipeline

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

All validation steps must pass cleanly before proceeding to commit.

## Phase 2: Prepare Commit Message

**Format** (conventional commits, matching repo standards):

```text
type: brief description

Optional body with details
- Bullet point 1
- Bullet point 2
```

**Types:** `feat`, `fix`, `refactor`, `test`, `docs`, `style`, `chore`, `perf`

**Rules:**

- Subject line: lowercase `type:` prefix, max 72 chars
- No period at the end of subject line
- Body wrapped at 72 chars (optional, for complex changes)

## Phase 3: Stage & Commit

```bash
git add [specific-files]
git status
git diff --cached
```

Verify:

- All intended files are staged
- No accidental files included (`target/`, `.env`, temporary trace logs, binary outputs)

### Create Commit

```bash
git commit -m "type: description"
```

**NEVER** use `--no-verify` or `-n` — commit hooks MUST run.

If this change includes a version bump, verify `Cargo.toml` (and `Cargo.lock` if updated) are correct before committing.

### Handle Hook / Build Failures

If validation or commit hooks fail:

1. Read the error output carefully
2. Fix the issue:
   - Formatting errors: `cargo fmt`
   - Clippy warnings: `cargo clippy --fix` or manual fix
   - Test failures: fix failing tests or underlying logic (`cargo test`)
3. Stage fixed files: `git add [fixed-files]`
4. Create a **new** commit (never `--amend` unless explicitly requested)

### Verify Commit

```bash
git log --oneline -n 1
git diff HEAD~1 --stat
```

Confirm the commit message and changed files match intent.

## Review Summary

After commit, output:

```text
Commit: [hash] [message]
Files: [count] | LOC: [additions+deletions]
Checks: cargo fmt [pass/fail] | clippy [pass/fail] | tests [pass/fail] | build [pass/fail]
Issues: [count critical] / [count high] / [count medium]
```

## Key Rules

1. **Never `--no-verify`** — git hooks must run
2. **All Rust checks pass** before commit (`cargo fmt --check && cargo clippy && cargo test && cargo build`)
3. **Use `cargo`** — standard toolchain for this Rust codebase (never `npm` or `bun`)
4. **Conventional commits** — `type: description` format
5. **New commits only** — never amend unless user explicitly requests it