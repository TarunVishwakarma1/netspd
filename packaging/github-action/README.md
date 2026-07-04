# netspd GitHub Action

Measure network speed, latency and bufferbloat from CI — runner
uplink, or your own endpoints via `netspd serve`.

> To publish: copy this directory into its own repository (e.g.
> `TarunVishwakarma1/netspd-action`), tag `v1`, and publish it to the
> GitHub Marketplace from the repo page.

## Usage

```yaml
- name: Network baseline
  uses: TarunVishwakarma1/netspd-action@v1
  with:
    duration: 5
    fail-below: 50        # fail the job under 50 Mbps download

- name: Use the result
  run: echo '${{ steps.speed.outputs.json }}' | jq .download_mbps
```

Test a staging endpoint instead of the public internet:

```yaml
- uses: TarunVishwakarma1/netspd-action@v1
  with:
    url: https://speed.staging.example.com:9516
```

## Inputs

| Input | Default | Description |
| --- | --- | --- |
| `version` | latest release | netspd release tag |
| `url` | – | test one specific server (`netspd serve` peer or LibreSpeed backend) |
| `provider` | `librespeed` | `librespeed` / `ookla` / `fast` / `custom` |
| `duration` | `5` | seconds per transfer phase |
| `fail-below` | – | fail when download Mbps is below this |

## Outputs

| Output | Description |
| --- | --- |
| `json` | the full report (`download_mbps`, `upload_mbps`, `ping_ms`, `bufferbloat`, …) |
