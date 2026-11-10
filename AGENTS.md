# AGENTS.md

Notes for anyone — human or otherwise — changing this repository.

## What this project is

A lightweight, terminal-aware JSON pretty printer for Rust, built on console-rs
and Serde. Serialize directly to the caller's writer; keep JSON encoding in
`serde_json` and presentation in `console`. This crate is JSON-only and has no CLI.

## Non-negotiable invariants

Each of these should be enforced by a hook or a test. An invariant nothing
checks is a comment, and it will be violated.

1. **Styling preserves JSON text** — enforced by
   `nested_json_preserves_escaping_unicode_structure_and_values` in
   `tests/rendering.rs`.
2. **Stdout pipes always receive compact, uncolored JSON** — enforced by
   `redirected_stdout_is_always_compact_and_uncolored` in
   `tests/rendering.rs`.
3. **Write errors retain their I/O kind** — enforced by
   `writer_errors_are_preserved_even_inside_colored_tokens` in
   `tests/rendering.rs`.

## Layout

```text
src/
├── lib.rs        the library surface and writer entry points
├── error.rs      the crate's error type
├── formatter.rs  private streaming Serde JSON formatter
└── options.rs    color policy, options, and theme
```

There is no binary. Anything `pub` in `lib.rs` is public API, so keep the
surface small and deliberate.

## Style

**Every non-obvious line carries a comment saying why.** Not what — the code
says what. Ideally naming the failure it prevents. A comment that restates the
code is worse than none.

**Errors are typed and actionable.** Use `thiserror` for the error type and
preserve underlying serialization and I/O errors through source chaining.
Explain what specifically failed and document how callers can recover. Leave
diagnostic rendering to the caller; do not depend on `miette`.

**Test names are sentences.** `an_unchanged_input_produces_no_output`, not
`test_run_2`. The name should say what would be broken if it failed.

## Commits

Conventional Commits, enforced by Commitizen with Emotional on `commit-msg`. The type becomes a
changelog heading, so choose it as if someone will read it in release notes —
because they will.

## Releases

Driven by gh-ship. Never bump a version or push a tag by hand: `cliff.toml`
derives the version from the commit history, `prepare-release` applies it, and
`.github/ship.yml` is the contract between them. See CONTRIBUTING.md.

## Before you push

Follow [CONTRIBUTING.md's validation guidance](CONTRIBUTING.md#before-you-push)
to choose checks relevant to the change and report any skipped checks in your
handoff or PR description.

## This repository is generated from a template

The toolchain, hooks, CI and release workflows come from
[rust.tpl](https://github.com/noirbizarre/rust.tpl) and are updated with
`git tpl update`. Files carrying template-owned content end with a
`# --- project-specific ---` marker: add below it. Global mise tools instead go
between the begin/end project-specific tools markers inside `[tools]`.

Changing template-owned content here fixes it in one repository. Changing it in
the template fixes it in all of them — prefer that.
