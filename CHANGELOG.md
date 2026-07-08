# Changelog

All notable changes to netspd are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/).

## [0.1.6] - 2026-07-07

### Added

- **Wallpaper support** (`[wallpaper]` in `config.toml`): a vertical
  colour gradient can be rendered behind all UI elements. `kind =
  "gradient"` activates it; `from` and `to` (both `#rrggbb`, optional)
  set the top and bottom colours — unset values default to the theme's
  background and a 50 %-darkened version of it.
- **Theme selector swatches**: every entry shows a five-colour preview
  bar (accent, download, upload, success, warning) beside its name.
- **Mbps ↔ MB/s toggle**: press `u` at any time (during test or on
  results) to flip speed display between Mbps and MB/s (divide by 8).
  Unit persists across screens and is reflected in all gauges, dials
  and the results summary.
- **Composite score**: results screen now shows a weighted 0–100 score
  and letter grade (A+–F). Weights: download 30 %, upload 20 %, ping
  25 %, jitter 10 %, bufferbloat 15 % (redistributed proportionally
  when bufferbloat data is unavailable).
- **Desktop notification**: a system notification fires when a test
  completes (Unix only, via `notify-rust`). Shows download, upload and
  ping in the body. Disable with `notify = false` in `config.toml`.
- **One-line mode** (`--one-line`): prints a single tmux/status-bar
  friendly line to stdout — `↓93.7 ↑66.1 ~12ms A+` — then exits.
  Incompatible with `--json`, `--csv` and `--no-tui`.
- **Ping histogram**: the ping card on the results screen now shows an
  8-bucket Unicode block histogram (`▁▂▃▄▅▆▇█`) with a min–max range
  label. Rendered only when ≥ 3 samples with > 0.5 ms spread exist.
- **Server latency in server list**: each row in the server picker now
  shows the measured probe latency (`~14ms`), coloured in the latency
  accent. Servers are already sorted nearest-first; this makes the
  ordering transparent.

### Fixed

- **Upload progress bar snapped to half**: the bar was gated on the first
  counted byte, but an upload counts bytes only when a whole request
  completes — several seconds on a slow uplink. The bar sat empty, then
  jumped to the elapsed-time ratio the instant the first body landed. The
  bar now tracks elapsed time for the whole phase, and each upload worker
  sends a small warm-up body first so the gauge reads a real speed within
  the first sampling intervals instead of after the first full body.
- **Terminal cursor desync after a test**: the dial canvas and progress
  bar push many escape sequences per frame; dropped sequences could leave
  the terminal cursor out of sync with ratatui's model, landing text in
  the wrong cells on the results screen. The screen is now cleared on the
  transition off the testing screen. The clear sends `ESC[2J` +
  `MoveTo(0, 0)` directly and invalidates ratatui's diff buffer via
  `Terminal::resize`, rather than `Terminal::clear()` which queries the
  cursor position (`ESC[6n`) and could time out while the pty was still
  draining — the timeout previously crashed the process.
- **Duplicate footer bar on results screen**: two keyhint rows were
  visible (one from the screen, one from the global renderer). The
  per-screen footer was removed; all hint/notice/countdown logic now
  lives in `footer.rs` and renders exactly one row.

### Changed (internal)

- `app/state.rs` split into `screen.rs` (`Screen` enum) and `views.rs`
  (`PingView`, `TransferView`) to keep each file focused on one concern.
- Footer hint tables and the results-screen notice/countdown override
  moved from `renderer.rs` into `footer.rs` (`render_frame`).
- `download_card.rs` and `upload_card.rs` (19-line near-identical
  wrappers) deleted; replaced by `transfer_card::render_download` /
  `render_upload` in the existing `transfer_card.rs`.
- Wallpaper rendering extracted to `tui/wallpaper.rs`; the renderer's
  background pass is now a single `wallpaper::render` call that handles
  both solid and gradient modes.

## [0.1.5] - 2026-07-04

### Fixed

- Added providers in the config for the netspeed
  after ``` netspd ``` command in the TUI press the 
  ``` c ``` to open up the config and set the providers.


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

[Unreleased]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.6...HEAD
[0.1.6]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/TarunVishwakarma1/netspd/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/TarunVishwakarma1/netspd/releases/tag/v0.1.0
