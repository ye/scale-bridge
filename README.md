# scale-bridge

Rust CLI and library for Avery Weigh-Tronix scales speaking the SCP-01/NCI protocol over serial or Ethernet.

## Current Hardware Notes

The implementation has been validated against this device:

- Model: `NCI 6720-30`
- Capacity: `15 kg / 30 lb`

Observed device-specific behavior on that unit:

- Serial parity must be `even`. The CLI now defaults to `--parity even`.
- `weight` replies use uppercase units such as `LB`.
- `status` replies may be standalone ASCII frames such as `S00`.
- `weight` may also reply with a standalone ASCII status frame such as `S10` instead of a weight payload when the load is unstable.
- Unsupported commands reply with framed `?` responses like `LF ? CR ETX`.

## Supported Commands On Tested Hardware

Verified working:

- `weight`
- `status`
- `zero`

Verified unsupported on the tested `NCI 6720-30`:

- `tare`
- `metrology`
- `about`
- `diagnostic`

Unsupported commands were observed returning the framed `?` response on the wire.

## Operational Notes For NCI 6720-30

- The tested unit uses ASCII status codes in weight responses and standalone status frames.
- `S00` means the reading is stable and non-zero.
- `S10` means the weight is not ready because the load is unstable or changing.
- `S20` means the scale is at zero. On the tested unit, this matches the illuminated zero indicator on the LCD.
- `zero` is recognized by the scale, but it may not actually zero the displayed weight if the current load is outside the scale's allowed zero window.
- In testing, issuing `zero` with about `1.32 lb` present returned a status reply and the follow-up weight remained non-zero.
- Treat this as device behavior or configuration/calibration state, not a transport or parser failure.

Representative raw responses observed on the tested unit:

- Stable non-zero reading with `S00`:
  `LF 002.98LB CR LF S00 CR ETX`
- Unstable dynamic load with `S10`:
  `LF S10 CR ETX`
- Zero reading with `S20`:
  `LF 000.00LB CR LF S20 CR ETX`

## Serial Usage

Typical invocation for the tested unit:

```bash
./target/debug/scale-bridge --serial-port /dev/ttyUSB0 --baud 9600 weight
```

The default serial settings used by the CLI are:

- `--baud 9600`
- `--parity even`
- `7 data bits`
- `1 stop bit`

## Windows PowerShell

Use the Windows executable from a PowerShell prompt:

```powershell
.\scale-bridge.exe --serial-port COM3 weight
```

Useful notes:

- Windows serial ports are typically named `COM3`, `COM4`, and so on.
- USB serial adapters usually appear in Device Manager under `Ports (COM & LPT)`.
- If the scale is connected through a USB-to-serial adapter, note the assigned `COM` number and pass it to `--serial-port`.
- The same default serial settings still apply: `9600` baud, even parity, `7` data bits, `1` stop bit.

Example HTTPS server invocation from PowerShell:

```powershell
.\scale-bridge.exe --serial-port COM3 serve --https-port 8443 --bind 127.0.0.1 --cert .\cert.pem --key .\key.pem
```

## macOS

Use the macOS executable from Terminal:

```bash
./scale-bridge --serial-port /dev/cu.usbserial-0001 weight
```

Useful notes:

- USB serial devices on macOS commonly appear as `/dev/cu.usbserial-*`, `/dev/cu.usbmodem*`, `/dev/tty.usbserial-*`, or `/dev/tty.usbmodem*`.
- Prefer the `/dev/cu.*` device for initiating outbound serial connections from the CLI.
- To inspect available serial devices before and after plugging in the adapter, run `ls /dev/cu.* /dev/tty.*`.
- The same default serial settings still apply: `9600` baud, even parity, `7` data bits, `1` stop bit.

Example HTTPS server invocation on macOS:

```bash
./scale-bridge --serial-port /dev/cu.usbserial-0001 serve --https-port 8443 --bind 127.0.0.1 --cert ./cert.pem --key ./key.pem
```

## Release Builds

Tagged releases use GitHub Actions to build downloadable archives for Linux, macOS, and Windows.

- Create and push a version tag such as `v0.2.0`:

```bash
git tag v0.2.0
git push origin v0.2.0
```

- The `Release` workflow in `.github/workflows/release.yml` runs on tags matching `v*`.
- It builds release binaries for Linux, macOS, and Windows, packages them, and attaches the archives to the GitHub Release page for that tag.
- Each archive includes `scale-bridge`, `scale-bridge-generate`, and `README.md`.

## CLI Help

The CLI has two levels of help:

- `scale-bridge --help` shows top-level flags such as `--serial-port`, `--baud`, `--parity`, `--host`, and `--tcp-port`.
- `scale-bridge <subcommand> --help` shows options specific to that subcommand.

For the HTTPS server, use:

```bash
scale-bridge serve --help
```

That is where listener-specific options such as `--https-port` and `--bind` are documented.

## Debugging Wire Responses

Enable wire logging with `--verbose 1` or higher:

```bash
./target/debug/scale-bridge --serial-port /dev/ttyUSB0 --verbose 1 weight
```

Example observed traffic on the tested unit:

```text
DEBUG tx: 57 0D
DEBUG rx: 0A 30 30 31 2E 33 34 4C 42 0D 0A 53 30 30 0D 03
```

That response decodes to `1.34 lb` with ASCII status `S00`, meaning a stable non-zero reading.
