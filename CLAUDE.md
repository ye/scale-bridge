# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

```bash
just build                        # debug build
just release                      # release build
just test                         # all workspace tests (uses cargo-nextest if available)
just test-crate scale-bridge-scp01  # test a single crate
just lint                         # clippy -D warnings
just fmt                          # check formatting
just fmt-fix                      # fix formatting in place
just ci                           # full CI: fmt → lint → test
just mock weight                  # run CLI against mock scale (no hardware needed)
just fuzz                         # property-based fuzz tests for the SCP-01 parser
just docs                         # generate and open rustdoc
```

To run a single test by name:
```bash
cargo test -p scale-bridge-scp01 parse_ascii_status
```

## Workspace Layout

Four crates with a strict dependency chain:

```
scale-bridge-core        # Transport/Codec/Protocol traits + generic Scale orchestrator
    └── scale-bridge-scp01   # SCP-01/NCI protocol implementation (commands, parsers, types)
            ├── scale-bridge-server  # Axum HTTPS REST API wrapping the protocol
            └── scale-bridge-cli    # CLI binary + scale-bridge-generate helper binary
```

## Architecture

### Core Abstraction (scale-bridge-core)

Three traits compose to form the `Scale<T, C, P>` generic struct:

- **Transport** (`Read + Write + set_timeout + flush_output`) — `SerialTransport`, `TcpTransport`, `MockTransport`
- **Codec** (`encode`/`decode`) — `EtxCodec` handles ETX-framed wire bytes
- **Protocol** (`encode_command`/`decode_response`) — `NciProtocol` implements SCP-01/NCI

`Scale::send(cmd)` wires them together: encode → frame → write → read → deframe → decode. Use `MockTransport` in tests to inject raw byte responses without hardware.

### Protocol Layer (scale-bridge-scp01)

- `NciCommand` enum covers: Weight, Status, Zero, Tare, Units, About, Diagnostic, Metrology, HighResolution
- `NciResponse` wraps: `WeightReading`, `ScaleStatus`, raw bytes for diagnostics/metrology
- **Key parser behaviour:** a Weight or Status request can legitimately receive a `Status`-only reply (unstable load, zero condition). The parser detects standalone status frames and returns `NciResponse::Status`. Callers must handle this case.

### Server (scale-bridge-server)

- Built on Axum with spawn-blocking for the synchronous transport I/O
- `ServerTransport` enum dispatches to Serial/Tcp/Mock at runtime (no generic parameter — avoids monomorphisation complexity in async handlers)
- Error mapping: `ScaleError::UnrecognizedCommand` → 501, `Timeout` → 504, `ParseError`/`FramingError` → 502, status-instead-of-weight conflict → 409

### Serial Feature Gate

`scale-bridge-core` gates `SerialTransport` behind the `serial` default feature. On Linux CI, `libudev-dev` must be installed for the feature to compile.

## Key ScaleError Variants

| Variant | Meaning |
|---|---|
| `UnrecognizedCommand` | Scale replied with framed `?` — command not supported by this device |
| `Timeout` | No response within the configured timeout |
| `FramingError` | ETX framing violated |
| `ParseError` | Response bytes did not match expected format |

## Git Hooks

Install the pre-push hook (runs fmt check + clippy before every push):
```bash
ln -sf ../../hooks/pre-push .git/hooks/pre-push
```

## Device Quirks (Reference)

- **NCI 6720-30**: requires even parity; ASCII status codes (`S00`/`S10`/`S20`); unsupported commands return framed `?`
- **NCI/AWT 7820-50**: binary status bytes (not ASCII); USB-to-serial adapter
- `zero` may not take effect if the load is outside the scale's allowed zero window — this is device behaviour, not a protocol failure
