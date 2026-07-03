# netspd

A beautiful, modern, minimalistic network speed testing terminal application, written entirely in Rust.

netspd measures **ping, jitter, download and upload** against LibreSpeed-compatible servers and renders them live in a smooth, themeable TUI — in the spirit of tools like `btop`, `gitui` and `lazygit`.

```
  netspd  v0.1.0                                LibreSpeed · Frankfurt, Germany

    ⠹ Download            Frankfurt, Germany                00:07  eta 4s

                                DOWNLOAD

                     ▄█ ▄▀▀▄ ▄▀▀▄   █▀▀▀
                      █  █  █ ▄▄▀   ▀▀▀▄  Mbps
                     ▄█▄ ▀▄▄▀ █▄▄▄  ▄▄▄▀
                        ▂▃▅▆▇█▇▆▇█▇▇▆▇█

           ██████████████████████████▌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌

   ╭ ⇄ Ping ─────────╮  ╭ ↓ Download ──────╮  ╭ ↑ Upload ────────╮
   │ 12.4 ms          │  │ 142.5 Mbps       │  │ —                │
   │ jitter  1.2 ms   │  │ avg   138.1 Mbps │  │ avg   —          │
   │ range   11 – 15  │  │ peak  151.0 Mbps │  │ peak  —          │
   │ loss    0%       │  │ data  171.2 MB   │  │ data  —          │
   ╰──────────────────╯  ╰──────────────────╯  ╰──────────────────╯

              q quit · r restart · s servers · t theme · ? help
```

> Screenshots coming soon.

## Features

- **Hypercar tachometer** — braille-canvas dial with a heat-gradient band, hatched redline, spring-loaded needle with afterglow, ignition sweep on phase start, session-peak ghost notch and a latency sub-dial; twin instrument cluster on wide terminals
- **Live metrics** — current, average and peak speed, transferred bytes, ETA, elapsed time
- **Latency analysis** — multiple samples, outlier trimming, jitter, packet-loss ready
- **Streaming transfers** — nothing is buffered in memory; parallel connections with EMA smoothing
- **Headless mode** — `--no-tui` for scripts and cron, `--json` for machine-readable reports
- **Result history** — every run appended as JSON lines to your data directory
- **Provider architecture** — LibreSpeed today; Ookla, Fast.com or self-hosted backends are one trait away
- **Themes** — Default, Nord, Dracula, Catppuccin, Gruvbox, plus your own TOML themes without recompiling
- **Responsive layout** — adapts from 80×24 up to 4K terminals
- **Smooth animation** — spring-physics needles, interpolated counters, gradient progress bars, braille spinners, capped at 60 FPS
- **Graceful everywhere** — cancellation, retries, timeouts; network failures never panic

## Installation

### From source

```sh
git clone https://github.com/tarunvishwakarma/netspd
cd netspd
cargo install --path .
```

Requires Rust 1.80 or newer.

## Usage

```sh
netspd            # full TUI
netspd --no-tui   # headless: progress on stderr, summary on stdout
netspd --json     # headless: report as one JSON object on stdout
```

The test starts automatically: ping → download → upload, then a results summary. Every completed run is appended to `<data dir>/netspd/history.jsonl` (macOS: `~/Library/Application Support/netspd/`), one JSON object per line:

```sh
netspd --json | jq .download_mbps
```

### Keyboard

| Key | Action |
| --- | --- |
| `q` / `Esc` | Quit (Esc closes overlays first) |
| `r` | Restart the test |
| `s` | Server selection |
| `t` | Theme selector |
| `c` | View configuration |
| `?` | Help |
| `↑↓` / `jk`, `Enter` | Navigate and confirm in lists |

## Configuration

netspd reads the first `config.toml` found in:

1. `$XDG_CONFIG_HOME/netspd/config.toml` (macOS: `~/Library/Application Support/netspd/config.toml`)
2. `./config/config.toml`

Every key is optional; see [`config/config.toml`](config/config.toml) for the annotated reference. Highlights:

```toml
theme = "catppuccin"       # default | nord | dracula | catppuccin | gruvbox
refresh_rate = 30          # UI frames per second (1..=60)
provider = "librespeed"
animation_speed = 1.0

[engine]
duration_secs = 10         # length of each transfer phase
connections = 4            # parallel connections
ping_samples = 10

[[servers]]                # optional: pin your own backend
name = "My Server"
url = "https://speedtest.example.com/backend/"
```

Custom themes go in `~/.config/netspd/themes/*.toml` — copy any file from [`assets/themes/`](assets/themes) as a starting point.

## Architecture

netspd follows clean architecture with strict one-way dependencies:

```
┌──────────────────────────────────────────────┐
│ tui/          presentation (Ratatui only)    │
│   theme · animation · widgets · screens      │
├──────────────────────────────────────────────┤
│ app/          application                    │
│   events → controller → state → renderer    │
├──────────────────────────────────────────────┤
│ engine/       domain (no UI imports)         │
│   Engine → scheduler → providers/network     │
│   metrics: EMA · sampler · throughput ·      │
│            latency · statistics              │
├──────────────────────────────────────────────┤
│ config/ · errors/ · utils/   infrastructure  │
└──────────────────────────────────────────────┘
```

- The **engine** emits strongly-typed `EngineEvent`s over a channel and never imports Ratatui — it is directly reusable from a CLI, GUI, REST API or as a library.
- **Providers** implement one trait (`Provider`); adding a new speed test network touches nothing else.
- **State** is plain data mutated only by reducers; the renderer just draws it.
- **Metrics** are pure, deterministic calculators, each independently unit-tested.

## Development

```sh
cargo test            # unit + integration tests
cargo clippy          # lint-clean, unwrap/expect/panic denied
cargo fmt --check
```

## Roadmap

- [x] JSON result export and `--no-tui` CLI mode
- [x] Result history (JSON lines)
- [ ] Trend sparklines over the stored history
- [ ] Packet loss via ICMP (the reporting model is already in place)
- [ ] Ookla and Fast.com providers
- [ ] Scheduled repeat testing

## Contributing

Contributions are welcome!

1. Fork and create a feature branch.
2. Keep the layering rules: engine code never imports UI code.
3. Add tests for new behavior; `cargo test`, `cargo clippy` and `cargo fmt --check` must pass.
4. Open a pull request with a clear description.

## License

[MIT](LICENSE)
