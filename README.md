# scale-bridge

Rust CLI and library for Avery Weigh-Tronix scales speaking the SCP-01/NCI protocol over serial or Ethernet.

## Tested Hardware

The implementation has been validated against these devices:

| Model | Capacity | Connection | Status Format |
|-------|----------|------------|---------------|
| NCI 6720-30 | 15 kg / 30 lb | USB Serial | ASCII (`S00`, `S10`, `S20`) |
| NCI / Avery Weigh-Tronix 7820-50 | 25 kg / 50 lb | USB Serial adapter | Binary status bytes |

### NCI 6720-30 Notes

Observed device-specific behavior on that unit:

- Serial parity must be `even`. The CLI now defaults to `--parity even`.
- `weight` replies use uppercase units such as `LB`.
- `status` replies may be standalone ASCII frames such as `S00`.
- `weight` may also reply with a standalone ASCII status frame such as `S10` instead of a weight payload when the load is unstable.
- Unsupported commands reply with framed `?` responses like `LF ? CR ETX`.

### NCI / Avery Weigh-Tronix 7820-50 Notes

- Connected via a USB-to-serial adapter (`/dev/ttyUSB0`).
- Uses binary SCP-01 status bytes (not ASCII status codes like the 6720-30).
- Same default serial settings: `9600` baud, even parity, `7` data bits, `1` stop bit.

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- [just](https://github.com/casey/just) task runner — `cargo install just`
- (Optional) [cargo-nextest](https://nexte.st/) for faster tests — `cargo install cargo-nextest`

### Clone and Build

```bash
git clone git@github.com:ye/scale-bridge.git
cd scale-bridge
just build
```

### Run Tests

```bash
just test                              # run all workspace tests
just test-crate scale-bridge-scp01     # run tests for a single crate
just fuzz                              # run fuzz tests for the SCP-01 parser
```

### Lint and Format

```bash
just fmt                               # check formatting
just lint                              # clippy — deny all warnings
just ci                                # full CI check: fmt → lint → test
```

### Install

Install the release binary, man page, and shell completions system-wide:

```bash
just install                           # builds release + installs to /usr/local
```

Or install to a custom prefix:

```bash
PREFIX="$HOME/.local" ./install.sh
```

To uninstall:

```bash
just uninstall
```

### Run With a Physical Scale

Once built or installed, connect your scale via USB serial and run:

```bash
# one-shot weight reading
scale-bridge --serial-port /dev/ttyUSB0 weight

# continuous streaming
scale-bridge --serial-port /dev/ttyUSB0 weight --watch

# check scale status
scale-bridge --serial-port /dev/ttyUSB0 status
```

### Run in Mock Mode (No Hardware)

For development and testing without a physical scale:

```bash
just mock weight
just mock status
just mock weight --watch
```

### Verbose / Debug Mode

Use `--verbose` to see raw wire traffic. This is essential for debugging protocol issues or adding support for new scale models:

```bash
# level 1: debug wire logs (hex bytes on tx/rx)
scale-bridge --serial-port /dev/ttyUSB0 --verbose 1 weight

# level 2: trace-level detail
scale-bridge --serial-port /dev/ttyUSB0 --verbose 2 weight --watch
```

Example output at `--verbose 1`:

```text
DEBUG tx: 57 0D
DEBUG rx: 0A 30 30 31 2E 33 34 4C 42 0D 0A 53 30 30 0D 03
```

That response decodes to `1.34 lb` with ASCII status `S00` (stable, non-zero reading).

When filing a bug or adding support for a new scale model, include the raw `--verbose 1` output so the exact bytes on the wire are visible.

### Available Just Recipes

| Recipe | Description |
|--------|-------------|
| `just build` | Build all crates (debug) |
| `just release` | Build release binary |
| `just test` | Run all tests |
| `just test-crate <name>` | Test a single crate |
| `just fuzz` | Run fuzz tests (SCP-01 parser) |
| `just lint` | Clippy with `-D warnings` |
| `just fmt` | Check formatting |
| `just fmt-fix` | Fix formatting in place |
| `just ci` | Full CI: fmt → lint → test |
| `just mock <args>` | Run CLI in mock mode |
| `just install` | Install to `/usr/local` |
| `just uninstall` | Uninstall |
| `just docs` | Generate and open rustdoc |
| `just generate` | Generate man page + shell completions |
| `just man` | Preview the man page |
| `just size` | Show release binary size |
| `just clean` | Clean build artifacts |

## Git Hooks

A `hooks/pre-push` script is included in the repository. It runs `cargo fmt --check` and `cargo clippy` before every push, blocking if either fails.

To install it, symlink or copy it into `.git/hooks`:

```bash
# symlink (recommended — picks up changes automatically)
ln -sf ../../hooks/pre-push .git/hooks/pre-push

# or copy
cp hooks/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

## Supported Commands On Tested Hardware

Verified working on both tested models:

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

## Serial Defaults

The CLI defaults match the most common NCI/AWT scale configuration:

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
