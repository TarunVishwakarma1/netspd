# Changelog

All notable changes to netspd are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Releases now refresh `packaging/` manifests automatically: the
  workflow recomputes scoop/winget/AUR checksums and the GitHub
  Action's default version from the published assets and commits them
  back.

## [0.1.4] - 2026-07-04

### Added

- Plain-language verdict on the results screen, share card and headless
  output: what the connection is actually good for.
- Prometheus integration: `--prom-textfile <path>` writes node_exporter
  gauges (speeds, latency, loss, loaded latency, bufferbloat grade)
  atomically after every completed run.
- `--fail-below <MBPS>`: exit code 2 when download misses the
  threshold, for CI checks and alerting.
- `netspd completions <shell>` and `netspd man`; release archives now
  bundle completions and the man page, and the Homebrew formula
  installs them.
- `.deb` and `.rpm` packages built on every release.
- Packaging kit under `packaging/`: winget manifests, Scoop manifest,
  AUR PKGBUILD, a publishable GitHub Action; plus a Nix flake
  (`nix run github:TarunVishwakarma1/netspd`).
- One-page website (`web/`) deployed via GitHub Pages.
- `examples/embed.rs`: using the UI-free engine as a library.

### Fixed

- Upload speeds against Ookla servers were wildly underreported
  (~50x): uploads were streamed with chunked transfer encoding, which
  Ookla's `upload.php` never answers, stalling every request until the
  phase deadline. Upload bodies now carry an explicit `Content-Length`
  (and default to 2 MB so round trips don't cap high-latency links).
  Verified at parity with speedtest.net on the same connection.

## [0.1.3] - 2026-07-04

### Added

- Bufferbloat measurement: latency is sampled during both transfer
  phases and graded A+–F against idle latency (Waveform thresholds).
  Shown on the results screen, in the share card, and exported in
  JSON/CSV as `loaded_down_ms` / `loaded_up_ms` / `bufferbloat`.
- `netspd serve`: a built-in speed test server (also answers LibreSpeed
  paths), so one binary measures pod-to-pod, host-to-host and LAN links.
  Pair with the new `--url` flag: `netspd --url http://peer:9516`.
- Scheduled repeat testing with `--interval`/`-i` (e.g. `15m`) or
  `repeat_interval` in config: the TUI auto-restarts with a countdown on
  the results screen; headless mode becomes a watch loop that logs every
  run and keeps going through transient failures.
- CSV export: `--csv` prints a header plus one row per run — combined
  with `--interval` it becomes a bandwidth logger.
- Shareable results: `y` copies a result card to the clipboard
  (pbcopy / wl-copy / xclip / clip.exe).
- `--compare N`: test the N nearest servers back to back and print a
  ranked table (works with `--json` / `--csv`).
- `--history`: dump stored results without running a test.
- Automatic failover: when a test fails, up to two next-nearest servers
  are tried before the error screen (headless tries up to three unless
  `--server` pins one).
- `-4` / `-6`: force IPv4 or IPv6 for measurements.
- `--ascii` and `NO_COLOR` support: plain-character UI with no braille,
  block art or colors for constrained terminals.
- Mouse support: wheel scrolls selection lists, click confirms.
- Editable settings screen: `↑↓` select, `←→` adjust within safe
  bounds, `w` writes `config.toml`; theme changes apply live.
- Trends screen upgraded to a real chart with axes and a per-server
  filter (`←→`).
- Environment variable equivalents for the main flags (`NETSPD_URL`,
  `NETSPD_SERVER`, `NETSPD_PROVIDER`, `NETSPD_INTERVAL`, `NETSPD_JSON`,
  `NETSPD_PORT`, `NETSPD_BIND`) for container manifests.
- Headless mode is now automatic when stdout is not a terminal
  (containers, CI, pipes).
- Kubernetes and container documentation: embed the static binary in
  any image, pod-to-pod and CronJob recipes.

### Fixed

- The ignition sweep now plays *before* data moves: the engine pauses
  for a configurable lead-in after announcing a transfer phase, so the
  needle sweep and the measurement no longer overlap. Headless mode
  skips the pause entirely.
- Headless mode no longer drops the final transfer summary line when
  the engine finishes before the last events are consumed.

## [0.1.2] - 2026-07-04

### Added

- Ookla (speedtest.net) and Fast.com providers, selectable with
  `--provider`/`-p` or `provider` in the config file. Both are pure
  `Provider` implementations — the transfer engine is unchanged.
- `custom` provider: test against your own servers (self-hosted
  backends, LAN machines) declared as `[[servers]]` entries, with any
  plain-HTTP endpoints.
## [0.1.1] - 2026-07-04

### Added

- Real packet loss measurement: a burst of ICMP echoes runs alongside
  the HTTP ping phase and refines the loss figure. Falls back to the
  HTTP-based estimate wherever ICMP sockets are unavailable or
  filtered.
- CLI flags via clap: `--server`/`-s` (pick by name or host substring),
  `--list-servers`, `--duration`/`-d`, `--connections`/`-c`, plus
  generated `--help`.
- Trends screen (`g`): download/upload sparklines and last/avg/best
  figures across all stored runs.
- Homebrew tap: `brew tap TarunVishwakarma1/tap && brew install
  netspd`, kept up to date automatically by the release pipeline.
- Release guard: tagging a version that doesn't match `Cargo.toml`
  fails the release before anything is published.

### Changed

- Minimum supported Rust version is 1.88 (required by the ratatui 0.30
  dependency family).

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

[Unreleased]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/TarunVishwakarma1/netspd/releases/tag/v0.1.0
