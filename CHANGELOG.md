# Changelog

All notable changes to netspd are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Real packet loss measurement: a burst of ICMP echoes runs alongside the
  HTTP ping phase and refines the loss figure. Falls back to the
  HTTP-based estimate wherever ICMP sockets are unavailable or filtered.

- Trends screen (`g`): download/upload sparklines and last/avg/best
  figures across all stored runs.
- CLI flags via clap: `--server`/`-s` (pick by name or host substring),
  `--list-servers`, `--duration`/`-d`, `--connections`/`-c`, plus
  generated `--help`.
- Automatic failover: when a test fails, up to two next-nearest servers
  are tried before the error screen (TUI) or the run aborts (headless
  tries up to three unless `--server` pins one).

## [0.1.0] - 2026-07-03

### Added

- Full speed test flow: HTTP ping (trimmed mean, jitter, loss), streaming
  download and upload over parallel connections with EMA smoothing.
- Hypercar tachometer dial: heat-gradient value band, hatched redline,
  spring-physics needle with afterglow trail, ignition sweep before each
  transfer phase, session-peak ghost notch, latency sub-dial, twin
  instrument cluster on wide terminals.
- Server discovery against the public LibreSpeed list with concurrent
  health probing; dead servers are filtered and the nearest is selected
  automatically.
- Headless mode: `--no-tui` for scripts and `--json` for a single
  machine-readable report on stdout.
- Result history appended as JSON lines to the platform data directory.
- Client IP/ISP shown in the header.
- Five built-in themes (Default, Nord, Dracula, Catppuccin, Gruvbox) plus
  user themes from TOML files, hot-selectable at runtime.
- Screens: splash, testing, results, help, settings, server selection,
  theme selection, error — responsive from 80×24 up to 4K terminals.
- Configuration via `config.toml` with sensible defaults and clamping.

[Unreleased]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/TarunVishwakarma1/netspd/releases/tag/v0.1.0
