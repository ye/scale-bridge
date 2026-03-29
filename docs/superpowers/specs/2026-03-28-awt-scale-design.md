# scale-bridge Design Spec
# Avery Weigh-Tronix(AWT) Scale CLI + Library

**Date:** 2026-03-28

## Problem

Avery Weigh-Tronix Digital Bench Scales communicate via the SCP-01 (NCI) serial protocol over RS-232/USB. No cross-platform Rust tool exists to query them from scripts, automation pipelines, or web applications. The architecture must support future integration of other scale brands, connection types (Serial, USB, Ethernet), and a local HTTPS REST server for web app access.

## Goals

- Cross-platform Rust library + CLI utility for AWT scales via SCP-01/NCI protocol
- One-shot commands + `--watch` streaming mode
- Text, JSON, and CSV output formats
- Architecture that supports new brands/protocols without refactoring
- Synchronous I/O, no tokio runtime
- `rust_decimal::Decimal` for all weight values (no f64 precision loss)
- Systemd-compatible for Linux daemon operation
- Complete test suite requiring no hardware

---

## Architecture

### Workspace Layout

```
scale-bridge/
├── Cargo.toml                         ← workspace root
├── Justfile                           ← cross-platform task runner
├── scale-bridge.service               ← systemd unit file
├── .github/workflows/ci.yml           ← Linux + Windows + macOS CI
├── docs/superpowers/specs/            ← this file
├── docs/superpowers/plans/            ← implementation plans
└── crates/
    ├── scale-bridge-core/             ← traits + shared types + transport impls
    ├── scale-bridge-scp01/            ← SCP-01/NCI protocol implementation
    ├── scale-bridge-server/           ← HTTPS REST server (stubbed)
    └── scale-bridge-cli/              ← binary entry point
```

### Dependency Direction (no cycles)

```
scale-bridge-cli
    ├── scale-bridge-core
    ├── scale-bridge-scp01  →  scale-bridge-core
    └── scale-bridge-server →  scale-bridge-core
```

### The Four Trait Layers

**Layer 1 — Transport** (`scale-bridge-core/src/transport/mod.rs`)
```rust
pub trait Transport: Read + Write {
    fn set_timeout(&mut self, timeout: Duration) -> Result<(), ScaleError>;
    fn flush(&mut self) -> Result<(), ScaleError>;
}
```
Implementations: `SerialTransport`, `TcpTransport`, `MockTransport`

**Layer 2 — Codec** (`scale-bridge-core/src/codec/mod.rs`)
```rust
pub trait Codec {
    fn encode(&self, raw: &[u8]) -> Vec<u8>;
    fn decode(&mut self, buf: &mut Vec<u8>) -> Result<Option<Vec<u8>>, ScaleError>;
}
```
Implementation: `EtxCodec` (buffers until ETX byte 0x03)

**Layer 3 — Protocol** (`scale-bridge-core/src/protocol.rs`)
```rust
pub trait Protocol {
    type Command: Command;
    type Response;
    fn encode_command(&self, cmd: &Self::Command) -> Vec<u8>;
    fn decode_response(&self, cmd: &Self::Command, frame: &[u8]) -> Result<Self::Response, ScaleError>;
}
pub trait Command {
    fn command_byte(&self) -> u8;
}
```

**Layer 4 — Scale** (`scale-bridge-core/src/scale.rs`)
```rust
pub struct Scale<T: Transport, C: Codec, P: Protocol> {
    transport: T, codec: C, protocol: P,
}
impl<T, C, P> Scale<T, C, P> {
    pub fn send(&mut self, cmd: P::Command) -> Result<P::Response, ScaleError>;
}
```

---

## SCP-01 / NCI Protocol Reference

**Serial config:** 7 data bits, 1 start, 1 stop, configurable parity, 1200–19200 baud

**Commands (all append `<CR>` = 0x0D):**

| Command | Byte | Type |
|---------|------|------|
| Weight | `W` | Mandatory |
| Status | `S` | Mandatory |
| Zero | `Z` | Mandatory |
| High-Resolution | `H` | Optional (10x) |
| Units | `U` | Optional |
| Metrology | `M` | Optional |
| Tare | `T` | Optional |
| About | `A` | Optional (7600 series) |
| Diagnostic | `D` | Optional (7600 series) |

**Response frame format:**
```
<LF>[DATA]<CR><LF>[STATUS_BYTES]<CR><ETX>
```
Where `<ETX>` = 0x03, `<LF>` = 0x0A, `<CR>` = 0x0D

**Weight data formats:**
- Decimal: `  1234.56lb` (space-padded, unit suffix)
- Lb-Oz: `  10lb  2.3oz`
- Over capacity: `^^^^^^^uu`
- Under capacity: `_______uu`
- Zero error: `-------uu`

**Status bytes:**
- Minimum 2 bytes; 3 if bit 6 of byte 2 is set
- Bit 4 and 5 always 1; bit 7 is odd parity over bits 0–6

| Bit | Byte 1 | Byte 2 | Byte 3 (opt) |
|-----|--------|--------|--------------|
| 0 | motion | under capacity | range LSB |
| 1 | at zero | over capacity | net weight |
| 2 | RAM error | ROM error | initial zero error |
| 3 | EEPROM error | faulty calibration | reserved |
| 4 | always 1 | always 1 | always 1 |
| 5 | always 1 | always 1 | always 1 |
| 6 | always 0 | 1=byte follows | 1=byte follows |
| 7 | parity | parity | parity |

Scale replies `?<CR><LF><ETX>` for unrecognized commands.

---

## Data Types (`scale-bridge-scp01`)

```rust
pub enum NciCommand { Weight, Status, Zero, HighResolution, Units, Metrology, Tare, About, Diagnostic }

pub struct WeightReading {
    pub value: Decimal,
    pub unit: WeightUnit,
    pub format: WeightFormat,
    pub display: DisplayState,
    pub status: ScaleStatus,
}
pub struct ScaleStatus {
    pub motion: bool, pub at_zero: bool,
    pub under_capacity: bool, pub over_capacity: bool,
    pub ram_error: bool, pub rom_error: bool,
    pub eeprom_error: bool, pub faulty_calibration: bool,
    pub net_weight: bool, pub initial_zero_error: bool,
    pub range: WeightRange,
}
pub enum WeightUnit   { Lb, Kg, Oz, G, LbOz }
pub enum WeightFormat { Decimal, PoundsOunces }
pub enum DisplayState { Normal, OverCapacity, UnderCapacity, ZeroError }
pub enum WeightRange  { Low, High }
```

## Error Type (`scale-bridge-core`)

```rust
pub enum ScaleError {
    Transport(std::io::Error),
    Timeout,
    FramingError(String),
    ParseError(String),
    UnrecognizedCommand,
    SerialPort(String),
}
```

---

## CLI Design

```bash
# One-shot serial
scale-bridge --serial-port /dev/ttyUSB0 --baud 9600 weight
scale-bridge --serial-port COM3 status
scale-bridge --serial-port /dev/ttyUSB0 zero
scale-bridge --serial-port /dev/ttyUSB0 tare
scale-bridge --serial-port /dev/ttyUSB0 high-resolution
scale-bridge --serial-port /dev/ttyUSB0 metrology
scale-bridge --serial-port /dev/ttyUSB0 about
scale-bridge --serial-port /dev/ttyUSB0 diagnostic

# TCP/Ethernet (scale has built-in Ethernet)
scale-bridge --host 192.168.1.50 --tcp-port 3001 weight

# Streaming
scale-bridge --serial-port /dev/ttyUSB0 weight --watch
scale-bridge --serial-port /dev/ttyUSB0 weight --watch --interval 500ms

# Output formats
scale-bridge --serial-port /dev/ttyUSB0 weight --output text
scale-bridge --serial-port /dev/ttyUSB0 weight --output json
scale-bridge --serial-port /dev/ttyUSB0 weight --output csv

# Server (stubbed)
scale-bridge --serial-port /dev/ttyUSB0 serve --https-port 8443 --bind 127.0.0.1 --cert cert.pem --key key.pem
```

**Exit codes:** 0=success, 1=scale error, 2=transport error, 3=parse error

---

## Testing Strategy

- All tests run without hardware via `MockTransport`
- `cargo nextest run` locally via `just test`
- `just ci` = fmt + lint + test (mirrors GitHub Actions)
- Unit tests for codec edge cases, status byte parsing, weight parsing
- Integration tests for every NCI command round-trip
- `proptest` fuzz tests for parser (must never panic)
- CLI integration tests via `assert_cmd` with `--mock` feature flag
- GitHub Actions CI: ubuntu-latest + windows-latest + macos-latest in parallel

## Key Dependencies

| Crate | Purpose |
|---|---|
| `serialport` | Cross-platform serial/USB |
| `rust_decimal` | Exact decimal weight values |
| `clap` (derive) | CLI argument parsing |
| `serde` + `serde_json` | JSON output |
| `tracing` | Structured logging |
| `proptest` | Fuzz testing |
| `assert_cmd` + `predicates` | CLI integration testing |
| `cargo-nextest` | Fast parallel test runner |
| `just` | Cross-platform task runner |
