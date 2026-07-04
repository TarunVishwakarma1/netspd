# Packaging

Ready-to-submit packaging for the ecosystems netspd doesn't publish to
automatically. **Versions and checksums in this directory are updated
by CI on every release** (the `packaging` job in
[`release.yml`](../.github/workflows/release.yml) recomputes hashes
from the published assets and pushes back to `main`) — don't edit them
by hand. Each ecosystem still needs a one-time manual submission,
after which most of it self-updates.

| Directory / file | Ecosystem | How to submit |
| --- | --- | --- |
| `scoop/netspd.json` | Scoop (Windows) | Copy into a `bucket/` folder of a repo named e.g. `scoop-bucket`; users run `scoop bucket add netspd https://github.com/TarunVishwakarma1/scoop-bucket && scoop install netspd`. The `autoupdate` block keeps it current. |
| `winget/*.yaml` | winget (Windows) | `wingetcreate submit` or a PR to `microsoft/winget-pkgs` under `manifests/t/TarunVishwakarma1/netspd/0.1.3/`. New versions: `wingetcreate update`. |
| `aur/PKGBUILD` | Arch AUR | Push to `ssh://aur@aur.archlinux.org/netspd.git` (needs an AUR account). Update `pkgver`/`sha256sums` per release, or hand to a community maintainer. |
| `../flake.nix` | Nix | Already usable: `nix run github:TarunVishwakarma1/netspd`. Bump `version` on release. |
| `github-action/` | GitHub Marketplace | Copy into its own repo (`netspd-action`), tag `v1`, publish to the marketplace from the repo page. |

Checksums always match the latest tagged release; CI keeps them
current. Manual fallback: `shasum -a 256 <asset>`.
