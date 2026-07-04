# Changelog

All notable changes to netspd are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `netspd serve`: a built-in speed test server (also answers LibreSpeed
  paths), so one binary measures pod-to-pod, host-to-host and LAN links.
  Pair with the new `--url` flag: `netspd --url http://peer:9516`.
- Environment variable equivalents for the main flags (`NETSPD_URL`,
  `NETSPD_SERVER`, `NETSPD_PROVIDER`, `NETSPD_INTERVAL`, `NETSPD_JSON`,
  `NETSPD_PORT`, `NETSPD_BIND`) for container manifests.
- Headless mode is now automatic when stdout is not a terminal
  (containers, CI, pipes).

- Bufferbloat measurement: latency is sampled during both transfer
  phases and graded A+–F against idle latency (Waveform thresholds).
  Shown on the results screen, in the share card, and exported in
  JSON/CSV as `loaded_down_ms` / `loaded_up_ms` / `bufferbloat`.
- `--compare N`: test the N nearest servers back to back and print a
  ranked table (works with `--json` / `--csv`).
- `--history`: dump stored results without running a test.
- `-4` / `-6`: force IPv4 or IPv6 for measurements.
- `--ascii` and `NO_COLOR` support: plain-character UI with no braille,
  block art or colors for constrained terminals.
- Mouse support: wheel scrolls selection lists, click confirms.
- Editable settings screen: `↑↓` select, `←→` adjust within safe
  bounds, `w` writes `config.toml`; theme changes apply live.
- Trends screen upgraded to a real chart with axes and a per-server
  filter (`←→`).

- Ookla (speedtest.net) and Fast.com providers, selectable with
  `--provider`/`-p` or `provider` in the config file. Both are pure
  `Provider` implementations — the transfer engine is unchanged.
- `custom` provider: test against your own servers (self-hosted
  backends, LAN machines) declared as `[[servers]]` entries, with any
  plain-HTTP endpoints.
- CSV export: `--csv` prints a header plus one row per run — combined
  with `--interval` it becomes a bandwidth logger.
- Shareable results: `y` copies a three-line result card to the
  clipboard (pbcopy / wl-copy / xclip / clip.exe).
- Scheduled repeat testing with `--interval`/`-i` (e.g. `15m`) or
  `repeat_interval` in config: the TUI auto-restarts with a countdown on
  the results screen; headless mode becomes a watch loop that logs every
  run and keeps going through transient failures.

### Fixed

- Headless mode no longer drops the final transfer summary line when the
  engine finishes before the last events are consumed.

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
