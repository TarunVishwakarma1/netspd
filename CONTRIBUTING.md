# Contributing to netspd

Thanks for your interest! netspd aims for the polish of tools like
`lazygit`, `btop` and `gitui` — contributions are held to the same bar the
existing code is.

## Development setup

```sh
git clone https://github.com/TarunVishwakarma1/netspd
cd netspd
cargo run
```

Rust 1.88+ is required. No other dependencies.

## Quality gates

Every PR must pass all three; CI enforces them on Linux, macOS and Windows:

```sh
cargo test
cargo clippy --all-targets   # zero warnings; unwrap/expect/panic are denied
cargo fmt --check
```

`unsafe` code is forbidden and every public item needs a rustdoc comment
(`missing_docs` warns).

## Architecture rules

The layering is strict and one-way — PRs that violate it will be asked to
restructure:

- `engine/` never imports Ratatui, Crossterm or anything from `tui/` or
  `app/`. It must stay usable as a plain library.
- `tui/` reads application state and draws it; it never talks to the
  network and never mutates state.
- State changes only happen in reducers (`app/state.rs`) and the
  controller; widgets and screens are pure render functions.
- New speed test providers implement the `Provider` trait in
  `engine/providers/` and touch nothing else.
- Metrics (`engine/metrics/`) stay pure and deterministic: explicit time
  inputs, no I/O, unit tests required.

## Pull requests

1. Fork, branch from `main`.
2. Keep PRs focused; one change per PR.
3. Add tests for new behavior (integration tests live in `tests/`).
4. Update `CHANGELOG.md` under `[Unreleased]`.
5. Describe *why*, not just *what*, in the PR body.

## Reporting bugs

Use the bug report issue template. The output of `netspd --no-tui` and
your terminal emulator + size help a lot; TUI rendering issues ideally
come with a screenshot.
