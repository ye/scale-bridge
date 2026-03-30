# scale-bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform Rust library + CLI for Avery WeighTronix scales via SCP-01/NCI protocol, with a layered architecture supporting future brands, connection types, and a REST server.

**Architecture:** Four trait layers (Transport → Codec → Protocol → Scale) in a Cargo workspace. `scale-bridge-core` owns traits and transports; `scale-bridge-scp01` implements NCI; `scale-bridge-cli` wires everything into a binary. All tests use `MockTransport` — no hardware required.

**Tech Stack:** Rust stable, `serialport 4`, `rust_decimal`, `clap 4` (derive), `serde_json`, `tracing`, `proptest`, `assert_cmd`, `cargo-nextest`, `just`

---

## File Map

```
scale-bridge/
├── Cargo.toml
├── Justfile
├── scale-bridge.service
├── .github/workflows/ci.yml
├── docs/superpowers/specs/2026-03-28-awt-scale-design.md   [already written]
├── docs/superpowers/plans/2026-03-28-scale-bridge.md       [this file]
└── crates/
    ├── scale-bridge-core/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs
    │       ├── protocol.rs          ← Protocol + Command traits
    │       ├── scale.rs             ← Scale<T,C,P> struct
    │       ├── transport/
    │       │   ├── mod.rs           ← Transport trait
    │       │   ├── mock.rs          ← MockTransport (test utility)
    │       │   ├── serial.rs        ← SerialTransport
    │       │   └── tcp.rs           ← TcpTransport
    │       └── codec/
    │           ├── mod.rs           ← Codec trait
    │           └── etx.rs           ← EtxCodec
    ├── scale-bridge-scp01/
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── command.rs           ← NciCommand enum
    │       ├── types.rs             ← WeightReading, ScaleStatus, enums
    │       ├── response.rs          ← NciResponse enum
    │       ├── parser/
    │       │   ├── mod.rs           ← parse_frame() dispatcher
    │       │   ├── status.rs        ← status byte parser
    │       │   └── weight.rs        ← weight/display state parser
    │       └── protocol.rs          ← NciProtocol impl
    ├── scale-bridge-server/
    │   ├── Cargo.toml
    │   └── src/lib.rs               ← stub only
    └── scale-bridge-cli/
        ├── Cargo.toml
        └── src/
            ├── main.rs
            ├── args.rs              ← clap structs
            ├── transport_builder.rs ← builds T from CLI args
            ├── runner.rs            ← executes subcommands
            └── output/
                ├── mod.rs
                ├── text.rs
                ├── json.rs
                └── csv.rs
```

---

### Task 1: Workspace Cargo.toml + crate scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `crates/scale-bridge-core/Cargo.toml`
- Create: `crates/scale-bridge-scp01/Cargo.toml`
- Create: `crates/scale-bridge-server/Cargo.toml`
- Create: `crates/scale-bridge-cli/Cargo.toml`

- [ ] **Step 1: Write workspace Cargo.toml**

```toml
[workspace]
members = [
    "crates/scale-bridge-core",
    "crates/scale-bridge-scp01",
    "crates/scale-bridge-server",
    "crates/scale-bridge-cli",
]
resolver = "2"

[workspace.dependencies]
rust_decimal = { version = "1", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
serialport = "4"
clap = { version = "4", features = ["derive"] }
proptest = "1"
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 2: Write scale-bridge-core Cargo.toml**

```toml
[package]
name = "scale-bridge-core"
version = "0.2.0"
edition = "2021"

[dependencies]
tracing = { workspace = true }
serialport = { workspace = true, optional = true }

[features]
default = ["serial"]
serial = ["dep:serialport"]

[dev-dependencies]
```

- [ ] **Step 3: Write scale-bridge-scp01 Cargo.toml**

```toml
[package]
name = "scale-bridge-scp01"
version = "0.2.0"
edition = "2021"

[dependencies]
scale-bridge-core = { path = "../scale-bridge-core" }
rust_decimal = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }
rust_decimal = { workspace = true, features = ["macros"] }
```

- [ ] **Step 4: Write scale-bridge-server Cargo.toml**

```toml
[package]
name = "scale-bridge-server"
version = "0.2.0"
edition = "2021"

[dependencies]
scale-bridge-core = { path = "../scale-bridge-core" }
tracing = { workspace = true }
```

- [ ] **Step 5: Write scale-bridge-cli Cargo.toml**

```toml
[package]
name = "scale-bridge-cli"
version = "0.2.0"
edition = "2021"

[[bin]]
name = "scale-bridge"
path = "src/main.rs"

[dependencies]
scale-bridge-core = { path = "../scale-bridge-core", features = ["serial"] }
scale-bridge-scp01 = { path = "../scale-bridge-scp01" }
scale-bridge-server = { path = "../scale-bridge-server" }
clap = { workspace = true }
rust_decimal = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = "0.3"

[dev-dependencies]
assert_cmd = { workspace = true }
predicates = { workspace = true }

[features]
mock = []
```

- [ ] **Step 6: Create all src/lib.rs and src/main.rs stubs**

```bash
mkdir -p crates/scale-bridge-core/src/{transport,codec}
mkdir -p crates/scale-bridge-scp01/src/parser
mkdir -p crates/scale-bridge-server/src
mkdir -p crates/scale-bridge-cli/src/output

# Each lib.rs starts empty:
for crate in scale-bridge-core scale-bridge-scp01 scale-bridge-server; do
  echo "// placeholder" > crates/$crate/src/lib.rs
done
echo 'fn main() {}' > crates/scale-bridge-cli/src/main.rs
```

- [ ] **Step 7: Verify workspace compiles**

```bash
cargo build --workspace
```
Expected: compiles with 0 errors (warnings about unused ok)

- [ ] **Step 8: Commit**

```bash
git init
git add Cargo.toml crates/
git commit -m "feat: initialize workspace with four crates"
```

---

### Task 2: ScaleError type

**Files:**
- Create: `crates/scale-bridge-core/src/error.rs`
- Modify: `crates/scale-bridge-core/src/lib.rs`

- [ ] **Step 1: Write failing test**

In `crates/scale-bridge-core/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_io_error_converts_to_timeout_variant() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timed out");
        let scale_err: ScaleError = io_err.into();
        assert!(matches!(scale_err, ScaleError::Timeout));
    }

    #[test]
    fn non_timeout_io_error_converts_to_transport_variant() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken pipe");
        let scale_err: ScaleError = io_err.into();
        assert!(matches!(scale_err, ScaleError::Transport(_)));
    }

    #[test]
    fn display_formats_all_variants() {
        assert!(ScaleError::Timeout.to_string().contains("timeout"));
        assert!(ScaleError::UnrecognizedCommand.to_string().contains("recognize"));
        assert!(ScaleError::FramingError("bad".into()).to_string().contains("bad"));
        assert!(ScaleError::ParseError("oops".into()).to_string().contains("oops"));
        assert!(ScaleError::SerialPort("port gone".into()).to_string().contains("port gone"));
    }
}
```

- [ ] **Step 2: Run test — expect compile failure**

```bash
cargo test -p scale-bridge-core 2>&1 | head -20
```
Expected: error — `ScaleError` not defined

- [ ] **Step 3: Implement ScaleError**

```rust
use std::fmt;

#[derive(Debug)]
pub enum ScaleError {
    Transport(std::io::Error),
    Timeout,
    FramingError(String),
    ParseError(String),
    UnrecognizedCommand,
    SerialPort(String),
}

impl fmt::Display for ScaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaleError::Transport(e) => write!(f, "transport error: {e}"),
            ScaleError::Timeout => write!(f, "scale communication timeout"),
            ScaleError::FramingError(msg) => write!(f, "framing error: {msg}"),
            ScaleError::ParseError(msg) => write!(f, "parse error: {msg}"),
            ScaleError::UnrecognizedCommand => write!(f, "scale did not recognize command"),
            ScaleError::SerialPort(msg) => write!(f, "serial port error: {msg}"),
        }
    }
}

impl std::error::Error for ScaleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ScaleError::Transport(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ScaleError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::TimedOut {
            ScaleError::Timeout
        } else {
            ScaleError::Transport(e)
        }
    }
}
```

- [ ] **Step 4: Export from lib.rs**

```rust
// crates/scale-bridge-core/src/lib.rs
mod error;
pub use error::ScaleError;
```

- [ ] **Step 5: Run tests — expect pass**

```bash
cargo test -p scale-bridge-core -- error
```
Expected: 3 tests pass

- [ ] **Step 6: Commit**

```bash
git add crates/scale-bridge-core/src/
git commit -m "feat(core): add ScaleError type with From<io::Error>"
```

---

### Task 3: Transport trait + MockTransport

**Files:**
- Create: `crates/scale-bridge-core/src/transport/mod.rs`
- Create: `crates/scale-bridge-core/src/transport/mock.rs`
- Modify: `crates/scale-bridge-core/src/lib.rs`

- [ ] **Step 1: Write failing tests in mock.rs**

```rust
// crates/scale-bridge-core/src/transport/mock.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn mock_returns_preset_response_bytes() {
        let response = b"\x0a  1234.56lb\x0d\x0a\xb0\xb0\x0d\x03".to_vec();
        let mut t = MockTransport::with_response(response.clone());
        let mut buf = vec![0u8; response.len()];
        t.read_exact(&mut buf).unwrap();
        assert_eq!(buf, response);
    }

    #[test]
    fn mock_captures_written_bytes() {
        let mut t = MockTransport::with_response(vec![]);
        t.write_all(b"W\r").unwrap();
        assert_eq!(t.written(), b"W\r");
    }

    #[test]
    fn mock_set_timeout_succeeds() {
        let mut t = MockTransport::with_response(vec![]);
        t.set_timeout(std::time::Duration::from_secs(1)).unwrap();
    }
}
```

- [ ] **Step 2: Run — expect compile failure**

```bash
cargo test -p scale-bridge-core -- transport 2>&1 | head -10
```

- [ ] **Step 3: Write Transport trait in mod.rs**

```rust
// crates/scale-bridge-core/src/transport/mod.rs
use std::io::{Read, Write};
use std::time::Duration;
use crate::ScaleError;

pub trait Transport: Read + Write {
    fn set_timeout(&mut self, timeout: Duration) -> Result<(), ScaleError>;
    fn flush_output(&mut self) -> Result<(), ScaleError>;
}

pub mod mock;
pub use mock::MockTransport;
```

- [ ] **Step 4: Implement MockTransport**

```rust
// crates/scale-bridge-core/src/transport/mock.rs
use std::io::{self, Cursor, Read, Write};
use std::time::Duration;
use crate::ScaleError;
use super::Transport;

pub struct MockTransport {
    reader: Cursor<Vec<u8>>,
    written: Vec<u8>,
}

impl MockTransport {
    pub fn with_response(response: Vec<u8>) -> Self {
        Self {
            reader: Cursor::new(response),
            written: Vec::new(),
        }
    }

    pub fn written(&self) -> &[u8] {
        &self.written
    }
}

impl Read for MockTransport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Write for MockTransport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl Transport for MockTransport {
    fn set_timeout(&mut self, _timeout: Duration) -> Result<(), ScaleError> { Ok(()) }
    fn flush_output(&mut self) -> Result<(), ScaleError> { Ok(()) }
}

#[cfg(test)]
mod tests { ... } // (paste tests from Step 1 here)
```

- [ ] **Step 5: Export from lib.rs**

```rust
// crates/scale-bridge-core/src/lib.rs
mod error;
pub mod transport;
pub use error::ScaleError;
pub use transport::{Transport, MockTransport};
```

- [ ] **Step 6: Run tests — expect pass**

```bash
cargo test -p scale-bridge-core -- transport
```
Expected: 3 tests pass

- [ ] **Step 7: Commit**

```bash
git add crates/scale-bridge-core/src/
git commit -m "feat(core): add Transport trait and MockTransport"
```

---

### Task 4: Codec trait + EtxCodec

**Files:**
- Create: `crates/scale-bridge-core/src/codec/mod.rs`
- Create: `crates/scale-bridge-core/src/codec/etx.rs`

ETX = 0x03 (End of Text). `EtxCodec::decode` accumulates bytes into an internal buffer until it sees 0x03, then returns the complete frame (including all bytes up to and including ETX) and clears the buffer.

- [ ] **Step 1: Write failing tests in etx.rs**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::Codec;

    fn codec() -> EtxCodec { EtxCodec::new() }

    #[test]
    fn returns_none_when_no_etx_yet() {
        let mut c = codec();
        let mut buf = b"hello".to_vec();
        assert!(c.decode(&mut buf).unwrap().is_none());
        assert!(buf.is_empty()); // consumed into internal buffer
    }

    #[test]
    fn returns_frame_when_etx_received() {
        let mut c = codec();
        let mut buf = b"\x0ahello\x0d\x03".to_vec();
        let frame = c.decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame, b"\x0ahello\x0d\x03");
        assert!(buf.is_empty());
    }

    #[test]
    fn handles_data_split_across_two_decode_calls() {
        let mut c = codec();
        let mut part1 = b"\x0ahel".to_vec();
        assert!(c.decode(&mut part1).unwrap().is_none());
        let mut part2 = b"lo\x0d\x03".to_vec();
        let frame = c.decode(&mut part2).unwrap().unwrap();
        assert_eq!(frame, b"\x0ahello\x0d\x03");
    }

    #[test]
    fn handles_two_frames_in_one_buffer() {
        let mut c = codec();
        let mut buf = b"\x0afirst\x03\x0asecond\x03".to_vec();
        let frame1 = c.decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame1, b"\x0afirst\x03");
        // remaining bytes for second frame
        let frame2 = c.decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame2, b"\x0asecond\x03");
    }

    #[test]
    fn encode_appends_cr() {
        let c = codec();
        assert_eq!(c.encode(b"W"), b"W\r");
    }

    #[test]
    fn empty_buffer_returns_none() {
        let mut c = codec();
        let mut buf = vec![];
        assert!(c.decode(&mut buf).unwrap().is_none());
    }
}
```

- [ ] **Step 2: Run — expect compile failure**

```bash
cargo test -p scale-bridge-core -- codec 2>&1 | head -10
```

- [ ] **Step 3: Write Codec trait in mod.rs**

```rust
// crates/scale-bridge-core/src/codec/mod.rs
use crate::ScaleError;

pub trait Codec {
    fn encode(&self, raw: &[u8]) -> Vec<u8>;
    fn decode(&mut self, buf: &mut Vec<u8>) -> Result<Option<Vec<u8>>, ScaleError>;
}

pub mod etx;
pub use etx::EtxCodec;
```

- [ ] **Step 4: Implement EtxCodec**

```rust
// crates/scale-bridge-core/src/codec/etx.rs
use super::Codec;
use crate::ScaleError;

const ETX: u8 = 0x03;
const CR: u8 = 0x0D;

pub struct EtxCodec {
    internal: Vec<u8>,
}

impl EtxCodec {
    pub fn new() -> Self {
        Self { internal: Vec::new() }
    }
}

impl Default for EtxCodec {
    fn default() -> Self { Self::new() }
}

impl Codec for EtxCodec {
    fn encode(&self, raw: &[u8]) -> Vec<u8> {
        let mut out = raw.to_vec();
        out.push(CR);
        out
    }

    fn decode(&mut self, buf: &mut Vec<u8>) -> Result<Option<Vec<u8>>, ScaleError> {
        self.internal.append(buf); // drain buf into internal

        if let Some(pos) = self.internal.iter().position(|&b| b == ETX) {
            let frame: Vec<u8> = self.internal.drain(..=pos).collect();
            // leftover bytes go back to buf for next call
            *buf = std::mem::take(&mut self.internal);
            self.internal.clear();
            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests { ... } // paste tests from Step 1
```

- [ ] **Step 5: Export from lib.rs**

```rust
// crates/scale-bridge-core/src/lib.rs
mod error;
pub mod codec;
pub mod transport;
pub use error::ScaleError;
pub use codec::{Codec, EtxCodec};
pub use transport::{Transport, MockTransport};
```

- [ ] **Step 6: Run tests — expect pass**

```bash
cargo test -p scale-bridge-core -- codec
```
Expected: 6 tests pass

- [ ] **Step 7: Commit**

```bash
git add crates/scale-bridge-core/src/
git commit -m "feat(core): add Codec trait and EtxCodec"
```

---

### Task 5: Protocol + Command traits + Scale struct

**Files:**
- Create: `crates/scale-bridge-core/src/protocol.rs`
- Create: `crates/scale-bridge-core/src/scale.rs`

- [ ] **Step 1: Write failing test for Scale::send round-trip**

```rust
// crates/scale-bridge-core/src/scale.rs  (bottom, cfg test)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockTransport, EtxCodec};

    struct EchoProtocol;

    #[derive(Clone)]
    struct EchoCommand(u8);

    impl crate::Command for EchoCommand {
        fn command_byte(&self) -> u8 { self.0 }
    }

    impl crate::Protocol for EchoProtocol {
        type Command = EchoCommand;
        type Response = Vec<u8>;

        fn encode_command(&self, cmd: &EchoCommand) -> Vec<u8> {
            vec![cmd.0]
        }
        fn decode_response(&self, _cmd: &EchoCommand, frame: &[u8]) -> Result<Vec<u8>, crate::ScaleError> {
            Ok(frame.to_vec())
        }
    }

    #[test]
    fn send_writes_encoded_command_and_returns_decoded_response() {
        // Response: some data + ETX
        let response = b"hello\x03".to_vec();
        let transport = MockTransport::with_response(response.clone());
        let mut scale = Scale::new(transport, EtxCodec::new(), EchoProtocol);
        let result = scale.send(EchoCommand(b'W')).unwrap();
        assert_eq!(result, b"hello\x03");
        assert_eq!(scale.transport.written(), b"W\r");
    }
}
```

- [ ] **Step 2: Run — expect compile failure**

```bash
cargo test -p scale-bridge-core -- scale 2>&1 | head -10
```

- [ ] **Step 3: Write Protocol and Command traits**

```rust
// crates/scale-bridge-core/src/protocol.rs
use crate::ScaleError;

pub trait Command {
    fn command_byte(&self) -> u8;
}

pub trait Protocol {
    type Command: Command;
    type Response;
    fn encode_command(&self, cmd: &Self::Command) -> Vec<u8>;
    fn decode_response(&self, cmd: &Self::Command, frame: &[u8]) -> Result<Self::Response, ScaleError>;
}
```

- [ ] **Step 4: Write Scale struct**

```rust
// crates/scale-bridge-core/src/scale.rs
use std::io::Read;
use crate::{Codec, Protocol, ScaleError, Transport};

pub struct Scale<T: Transport, C: Codec, P: Protocol> {
    pub(crate) transport: T,
    codec: C,
    protocol: P,
}

impl<T: Transport, C: Codec, P: Protocol> Scale<T, C, P> {
    pub fn new(transport: T, codec: C, protocol: P) -> Self {
        Self { transport, codec, protocol }
    }

    pub fn send(&mut self, cmd: P::Command) -> Result<P::Response, ScaleError> {
        // encode and write
        let bytes = self.protocol.encode_command(&cmd);
        let frame_out = self.codec.encode(&bytes);
        std::io::Write::write_all(&mut self.transport, &frame_out)?;
        self.transport.flush_output()?;

        // read until complete frame
        let frame_in = self.read_frame()?;
        self.protocol.decode_response(&cmd, &frame_in)
    }

    fn read_frame(&mut self) -> Result<Vec<u8>, ScaleError> {
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match self.transport.read(&mut byte) {
                Ok(0) => return Err(ScaleError::FramingError("connection closed before ETX".into())),
                Ok(_) => {
                    buf.push(byte[0]);
                    if let Some(frame) = self.codec.decode(&mut buf)? {
                        return Ok(frame);
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}
```

- [ ] **Step 5: Export from lib.rs**

```rust
// crates/scale-bridge-core/src/lib.rs
mod error;
pub mod codec;
pub mod protocol;
pub mod scale;
pub mod transport;
pub use error::ScaleError;
pub use codec::{Codec, EtxCodec};
pub use protocol::{Command, Protocol};
pub use scale::Scale;
pub use transport::{Transport, MockTransport};
```

- [ ] **Step 6: Run tests — expect pass**

```bash
cargo test -p scale-bridge-core
```
Expected: all tests pass

- [ ] **Step 7: Commit**

```bash
git add crates/scale-bridge-core/src/
git commit -m "feat(core): add Protocol/Command traits and Scale<T,C,P>"
```

---

### Task 6: NCI types (commands + data structs)

**Files:**
- Create: `crates/scale-bridge-scp01/src/command.rs`
- Create: `crates/scale-bridge-scp01/src/types.rs`
- Create: `crates/scale-bridge-scp01/src/response.rs`

- [ ] **Step 1: Write types.rs**

```rust
// crates/scale-bridge-scp01/src/types.rs
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WeightReading {
    pub value: Decimal,
    pub unit: WeightUnit,
    pub format: WeightFormat,
    pub display: DisplayState,
    pub status: ScaleStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScaleStatus {
    pub motion: bool,
    pub at_zero: bool,
    pub under_capacity: bool,
    pub over_capacity: bool,
    pub ram_error: bool,
    pub rom_error: bool,
    pub eeprom_error: bool,
    pub faulty_calibration: bool,
    pub net_weight: bool,
    pub initial_zero_error: bool,
    pub range: WeightRange,
}

impl ScaleStatus {
    pub fn has_error(&self) -> bool {
        self.ram_error || self.rom_error || self.eeprom_error
            || self.faulty_calibration || self.initial_zero_error
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WeightUnit { Lb, Kg, Oz, G, LbOz }

impl WeightUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            WeightUnit::Lb => "lb",
            WeightUnit::Kg => "kg",
            WeightUnit::Oz => "oz",
            WeightUnit::G => "g",
            WeightUnit::LbOz => "lb oz",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WeightFormat { Decimal, PoundsOunces }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DisplayState { Normal, OverCapacity, UnderCapacity, ZeroError }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WeightRange { Low, High }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetrologyReading {
    pub raw_counts: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AboutInfo {
    pub model: String,
    pub version: String,
    pub capacity: String,
    pub load_cell_serial: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DiagnosticInfo {
    pub power_on_starts: u32,
    pub calibrations: u32,
    pub overcapacity_events: u32,
    pub normalized_counts: u32,
    pub span_counts: u32,
    pub zero_counts: u32,
    pub cal_gravity: Decimal,
    pub span_weight: String,
}
```

- [ ] **Step 2: Write command.rs**

```rust
// crates/scale-bridge-scp01/src/command.rs
use scale_bridge_core::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum NciCommand {
    Weight,
    Status,
    Zero,
    HighResolution,
    Units,
    Metrology,
    Tare,
    About,
    Diagnostic,
}

impl Command for NciCommand {
    fn command_byte(&self) -> u8 {
        match self {
            NciCommand::Weight        => b'W',
            NciCommand::Status        => b'S',
            NciCommand::Zero          => b'Z',
            NciCommand::HighResolution => b'H',
            NciCommand::Units         => b'U',
            NciCommand::Metrology     => b'M',
            NciCommand::Tare          => b'T',
            NciCommand::About         => b'A',
            NciCommand::Diagnostic    => b'D',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_commands_have_correct_bytes() {
        assert_eq!(NciCommand::Weight.command_byte(), b'W');
        assert_eq!(NciCommand::Status.command_byte(), b'S');
        assert_eq!(NciCommand::Zero.command_byte(), b'Z');
        assert_eq!(NciCommand::HighResolution.command_byte(), b'H');
        assert_eq!(NciCommand::Units.command_byte(), b'U');
        assert_eq!(NciCommand::Metrology.command_byte(), b'M');
        assert_eq!(NciCommand::Tare.command_byte(), b'T');
        assert_eq!(NciCommand::About.command_byte(), b'A');
        assert_eq!(NciCommand::Diagnostic.command_byte(), b'D');
    }
}
```

- [ ] **Step 3: Write response.rs**

```rust
// crates/scale-bridge-scp01/src/response.rs
use serde::Serialize;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum NciResponse {
    Weight(WeightReading),
    HighResolution(WeightReading),
    Status(ScaleStatus),
    Acknowledged,              // Zero, Tare, Units — scale echoes command byte + CR
    Metrology(MetrologyReading),
    About(AboutInfo),
    Diagnostic(DiagnosticInfo),
    UnrecognizedCommand,       // scale replied '?'
}
```

- [ ] **Step 4: Wire up lib.rs**

```rust
// crates/scale-bridge-scp01/src/lib.rs
pub mod command;
pub mod types;
pub mod response;
pub mod parser;
pub mod protocol;

pub use command::NciCommand;
pub use response::NciResponse;
pub use types::*;
pub use protocol::NciProtocol;
```

- [ ] **Step 5: Add parser/mod.rs stub**

```rust
// crates/scale-bridge-scp01/src/parser/mod.rs
pub mod status;
pub mod weight;
```

- [ ] **Step 6: Add protocol.rs stub**

```rust
// crates/scale-bridge-scp01/src/protocol.rs
use scale_bridge_core::{Protocol, ScaleError};
use crate::command::NciCommand;
use crate::response::NciResponse;

pub struct NciProtocol;

impl Protocol for NciProtocol {
    type Command = NciCommand;
    type Response = NciResponse;

    fn encode_command(&self, cmd: &NciCommand) -> Vec<u8> {
        vec![cmd.command_byte()]
    }

    fn decode_response(&self, cmd: &NciCommand, frame: &[u8]) -> Result<NciResponse, ScaleError> {
        // check for '?' error response
        if frame.starts_with(b"?") {
            return Ok(NciResponse::UnrecognizedCommand);
        }
        crate::parser::parse_frame(cmd, frame)
    }
}
```

- [ ] **Step 7: Add parse_frame stub to parser/mod.rs**

```rust
// crates/scale-bridge-scp01/src/parser/mod.rs
pub mod status;
pub mod weight;

use scale_bridge_core::ScaleError;
use crate::command::NciCommand;
use crate::response::NciResponse;

pub fn parse_frame(cmd: &NciCommand, frame: &[u8]) -> Result<NciResponse, ScaleError> {
    match cmd {
        NciCommand::Weight | NciCommand::HighResolution => {
            weight::parse_weight(cmd, frame)
        }
        NciCommand::Status => {
            let status = status::parse_status_only(frame)?;
            Ok(NciResponse::Status(status))
        }
        NciCommand::Zero | NciCommand::Tare | NciCommand::Units => {
            Ok(NciResponse::Acknowledged)
        }
        NciCommand::Metrology => weight::parse_metrology(frame),
        NciCommand::About => weight::parse_about(frame),
        NciCommand::Diagnostic => weight::parse_diagnostic(frame),
    }
}
```

- [ ] **Step 8: Run tests — expect pass**

```bash
cargo test -p scale-bridge-scp01
```

- [ ] **Step 9: Commit**

```bash
git add crates/scale-bridge-scp01/src/
git commit -m "feat(scp01): add NciCommand, NciResponse, and type definitions"
```

---

### Task 7: Status byte parser

**Files:**
- Create: `crates/scale-bridge-scp01/src/parser/status.rs`

Status byte layout (bits 0–7, LSB first):
- Byte 1: b0=motion, b1=at_zero, b2=RAM_err, b3=EEPROM_err, b4=1, b5=1, b6=0, b7=parity
- Byte 2: b0=under_cap, b1=over_cap, b2=ROM_err, b3=faulty_cal, b4=1, b5=1, b6=more_bytes, b7=parity
- Byte 3 (optional): b0=range_LSB, b1=net_weight, b2=init_zero_err, b3=reserved, b4=1, b5=1, b6=more_bytes, b7=parity

Parity is odd parity over bits 0–6 (bit 7 makes total count of 1s odd).

- [ ] **Step 1: Write failing tests**

```rust
// crates/scale-bridge-scp01/src/parser/status.rs (bottom)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{WeightRange};

    fn make_byte(bits_0_6: u8) -> u8 {
        // set parity bit (bit 7) so total 1-count in byte is odd
        let count = bits_0_6.count_ones();
        if count % 2 == 0 { bits_0_6 | 0x80 } else { bits_0_6 }
    }

    #[test]
    fn parses_stable_not_at_zero_two_bytes() {
        // bits 4,5 always 1 → 0x30 base
        let b1 = make_byte(0x30); // stable, no errors
        let b2 = make_byte(0x30); // no errors, last byte (bit6=0)
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(!status.motion);
        assert!(!status.at_zero);
        assert!(!status.under_capacity);
        assert!(!status.over_capacity);
        assert!(!status.ram_error);
        assert!(!status.has_error());
        assert_eq!(status.range, WeightRange::Low);
    }

    #[test]
    fn parses_motion_flag() {
        let b1 = make_byte(0x30 | 0x01); // bit 0 = motion
        let b2 = make_byte(0x30);
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(status.motion);
    }

    #[test]
    fn parses_at_zero_flag() {
        let b1 = make_byte(0x30 | 0x02); // bit 1 = at_zero
        let b2 = make_byte(0x30);
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(status.at_zero);
    }

    #[test]
    fn parses_over_capacity() {
        let b1 = make_byte(0x30);
        let b2 = make_byte(0x30 | 0x02); // bit 1 = over_capacity
        let status = parse_status_bytes(&[b1, b2]).unwrap();
        assert!(status.over_capacity);
    }

    #[test]
    fn parses_three_byte_status_with_net_weight() {
        let b1 = make_byte(0x30);
        let b2 = make_byte(0x30 | 0x40); // bit 6 = more bytes follow
        let b3 = make_byte(0x30 | 0x02); // bit 1 = net_weight
        let status = parse_status_bytes(&[b1, b2, b3]).unwrap();
        assert!(status.net_weight);
    }

    #[test]
    fn parses_high_range() {
        let b1 = make_byte(0x30);
        let b2 = make_byte(0x30 | 0x40); // more bytes
        let b3 = make_byte(0x30 | 0x08); // bits 3:2 = 10 → high range (bit3=1, bit2=0 → range=High... actually see spec)
        // Per spec: byte3 bits 1:0 = 00=Low range, 11=High range
        let b3_high = make_byte(0x30 | 0x40 | 0x03); // bits 0,1 set → High range; bit6=more? No, bit6=0 here (last byte)
        let b3_high_last = make_byte(0x30 | 0x03); // bits 0,1 set → High range, bit6=0 (last byte)
        let status = parse_status_bytes(&[b1, make_byte(0x30 | 0x40), b3_high_last]).unwrap();
        assert_eq!(status.range, WeightRange::High);
    }

    #[test]
    fn returns_error_for_empty_input() {
        assert!(parse_status_bytes(&[]).is_err());
    }

    #[test]
    fn returns_error_for_one_byte_only() {
        assert!(parse_status_bytes(&[0xB0]).is_err());
    }
}
```

- [ ] **Step 2: Run — expect compile failure**

```bash
cargo test -p scale-bridge-scp01 -- parser::status 2>&1 | head -10
```

- [ ] **Step 3: Implement status byte parser**

```rust
// crates/scale-bridge-scp01/src/parser/status.rs
use scale_bridge_core::ScaleError;
use crate::types::{ScaleStatus, WeightRange};

/// Parse status bytes from an NCI response.
/// Expects 2 bytes minimum; reads 3 if bit 6 of byte 2 is set.
pub fn parse_status_bytes(bytes: &[u8]) -> Result<ScaleStatus, ScaleError> {
    if bytes.len() < 2 {
        return Err(ScaleError::ParseError(
            format!("expected at least 2 status bytes, got {}", bytes.len())
        ));
    }

    let b1 = bytes[0];
    let b2 = bytes[1];

    let motion        = b1 & 0x01 != 0;
    let at_zero       = b1 & 0x02 != 0;
    let ram_error     = b1 & 0x04 != 0;
    let eeprom_error  = b1 & 0x08 != 0;

    let under_capacity   = b2 & 0x01 != 0;
    let over_capacity    = b2 & 0x02 != 0;
    let rom_error        = b2 & 0x04 != 0;
    let faulty_calibration = b2 & 0x08 != 0;
    let more_bytes       = b2 & 0x40 != 0;

    let mut net_weight       = false;
    let mut initial_zero_error = false;
    let mut range            = WeightRange::Low;

    if more_bytes {
        if bytes.len() < 3 {
            return Err(ScaleError::ParseError(
                "byte 2 signals byte 3 follows, but not enough bytes".into()
            ));
        }
        let b3 = bytes[2];
        // bits 1:0 of byte3: 00=Low, 11=High (per spec)
        range = if b3 & 0x03 == 0x03 { WeightRange::High } else { WeightRange::Low };
        net_weight         = b3 & 0x02 != 0;
        initial_zero_error = b3 & 0x04 != 0;
    }

    Ok(ScaleStatus {
        motion,
        at_zero,
        under_capacity,
        over_capacity,
        ram_error,
        rom_error,
        eeprom_error,
        faulty_calibration,
        net_weight,
        initial_zero_error,
        range,
    })
}

/// Extract status bytes from a full NCI response frame.
/// Frame format: <LF>[DATA]<CR><LF>[STATUS_BYTES]<CR><ETX>
pub fn extract_status_bytes(frame: &[u8]) -> Result<(Vec<u8>, Vec<u8>), ScaleError> {
    // Find the second LF (0x0A) — status bytes follow it
    let mut lf_count = 0;
    let mut status_start = None;
    for (i, &b) in frame.iter().enumerate() {
        if b == 0x0A {
            lf_count += 1;
            if lf_count == 2 {
                status_start = Some(i + 1);
                break;
            }
        }
    }
    let start = status_start.ok_or_else(|| {
        ScaleError::ParseError("could not find status bytes in frame".into())
    })?;

    // Status bytes end at CR (0x0D) before ETX
    let end = frame[start..].iter().position(|&b| b == 0x0D)
        .map(|p| start + p)
        .ok_or_else(|| ScaleError::ParseError("no CR after status bytes".into()))?;

    let data = frame[1..].iter()
        .take_while(|&&b| b != 0x0D)
        .cloned()
        .collect();

    Ok((data, frame[start..end].to_vec()))
}

/// Parse ScaleStatus from a full NCI frame (extracts status bytes internally).
pub fn parse_status_only(frame: &[u8]) -> Result<ScaleStatus, ScaleError> {
    let (_, status_bytes) = extract_status_bytes(frame)?;
    parse_status_bytes(&status_bytes)
}
```

- [ ] **Step 4: Run tests — expect pass**

```bash
cargo test -p scale-bridge-scp01 -- parser::status
```
Expected: 7 tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/scale-bridge-scp01/src/parser/status.rs
git commit -m "feat(scp01): add status byte parser"
```

---

### Task 8: Weight and other response parsers

**Files:**
- Create: `crates/scale-bridge-scp01/src/parser/weight.rs`

NCI frame bytes for responses:
- Stable 1234.56 lb: `\x0a  1234.56lb\x0d\x0a\xb0\xb0\x0d\x03`
  - 0xB0 = 0x30 | 0x80 (bits 4,5 set; parity bit makes total odd since 2 ones → even → add bit7)
- Over capacity: `\x0a^^^^^^^lb\x0d\x0a\xb0\xb2\x0d\x03`  (0xB2 = over_cap bit set)
- Scale error '?': `?\x0d\x0a\x03`

- [ ] **Step 1: Write failing tests**

```rust
// crates/scale-bridge-scp01/src/parser/weight.rs (bottom, cfg test)
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::types::*;
    use crate::command::NciCommand;

    // Helper: stable weight frame for "  1234.56lb"
    // Status: b1=0xB0 (stable,no err), b2=0xB0 (no err, last)
    fn stable_lb_frame() -> Vec<u8> {
        b"\x0a  1234.56lb\x0d\x0a\xb0\xb0\x0d\x03".to_vec()
    }

    #[test]
    fn parses_decimal_lb_weight() {
        let frame = stable_lb_frame();
        let reading = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let crate::response::NciResponse::Weight(w) = reading {
            assert_eq!(w.value, dec!(1234.56));
            assert_eq!(w.unit, WeightUnit::Lb);
            assert_eq!(w.format, WeightFormat::Decimal);
            assert_eq!(w.display, DisplayState::Normal);
            assert!(!w.status.motion);
        } else {
            panic!("expected Weight response");
        }
    }

    #[test]
    fn parses_kg_weight() {
        let frame = b"\x0a    0.567kg\x0d\x0a\xb0\xb0\x0d\x03".to_vec();
        let reading = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let crate::response::NciResponse::Weight(w) = reading {
            assert_eq!(w.value, dec!(0.567));
            assert_eq!(w.unit, WeightUnit::Kg);
        } else {
            panic!();
        }
    }

    #[test]
    fn parses_lb_oz_format() {
        // "  10lb  2.3oz"
        let frame = b"\x0a  10lb  2.3oz\x0d\x0a\xb0\xb0\x0d\x03".to_vec();
        let reading = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let crate::response::NciResponse::Weight(w) = reading {
            assert_eq!(w.unit, WeightUnit::LbOz);
            assert_eq!(w.format, WeightFormat::PoundsOunces);
        } else {
            panic!();
        }
    }

    #[test]
    fn parses_over_capacity_display_state() {
        // b2=0xB2: bit1=over_cap set; parity: 0x32 has 3 ones (bits 1,4,5) → odd → no bit7 → 0x32
        let frame = b"\x0a^^^^^^^lb\x0d\x0a\xb0\x32\x0d\x03".to_vec();
        let reading = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let crate::response::NciResponse::Weight(w) = reading {
            assert_eq!(w.display, DisplayState::OverCapacity);
            assert!(w.status.over_capacity);
        } else {
            panic!();
        }
    }

    #[test]
    fn parses_under_capacity_display_state() {
        // b2 bit0=under_cap: 0x31 → 3 ones → odd parity → no bit7 → 0x31
        let frame = b"\x0a_______lb\x0d\x0a\xb0\x31\x0d\x03".to_vec();
        let reading = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let crate::response::NciResponse::Weight(w) = reading {
            assert_eq!(w.display, DisplayState::UnderCapacity);
        } else {
            panic!();
        }
    }

    #[test]
    fn parses_zero_error_display_state() {
        let frame = b"\x0a-------lb\x0d\x0a\xb0\xb0\x0d\x03".to_vec();
        let reading = parse_weight(&NciCommand::Weight, &frame).unwrap();
        if let crate::response::NciResponse::Weight(w) = reading {
            assert_eq!(w.display, DisplayState::ZeroError);
        } else {
            panic!();
        }
    }

    #[test]
    fn parses_about_response() {
        // Format: <LF>MMMM,VV-RR,CCCC,xxxxxx<CR><ETX>
        let frame = b"\x0a7600,01-02,150lb,ABC123\x0d\x03".to_vec();
        let about = parse_about(&frame).unwrap();
        if let crate::response::NciResponse::About(a) = about {
            assert_eq!(a.model, "7600");
            assert_eq!(a.version, "01-02");
            assert_eq!(a.capacity, "150lb");
            assert_eq!(a.load_cell_serial, Some("ABC123".into()));
        } else {
            panic!();
        }
    }

    #[test]
    fn parses_metrology_response() {
        let frame = b"\x0a   65000\x0d\x0a\xb0\xb0\x0d\x03".to_vec();
        let result = parse_metrology(&frame).unwrap();
        if let crate::response::NciResponse::Metrology(m) = result {
            assert_eq!(m.raw_counts, 65000);
        } else {
            panic!();
        }
    }
}
```

- [ ] **Step 2: Run — expect compile failure**

```bash
cargo test -p scale-bridge-scp01 -- parser::weight 2>&1 | head -10
```

- [ ] **Step 3: Implement weight parser**

```rust
// crates/scale-bridge-scp01/src/parser/weight.rs
use rust_decimal::Decimal;
use std::str::FromStr;
use scale_bridge_core::ScaleError;
use crate::command::NciCommand;
use crate::response::NciResponse;
use crate::types::*;
use super::status::{extract_status_bytes, parse_status_bytes};

pub fn parse_weight(cmd: &NciCommand, frame: &[u8]) -> Result<NciResponse, ScaleError> {
    let (data_bytes, status_bytes) = extract_status_bytes(frame)?;
    let status = parse_status_bytes(&status_bytes)?;

    let data = std::str::from_utf8(&data_bytes)
        .map_err(|e| ScaleError::ParseError(format!("non-UTF8 data: {e}")))?
        .trim();

    // Detect display states by first char
    let display = if data.starts_with('^') {
        DisplayState::OverCapacity
    } else if data.starts_with('_') {
        DisplayState::UnderCapacity
    } else if data.starts_with('-') {
        DisplayState::ZeroError
    } else {
        DisplayState::Normal
    };

    // Parse lb-oz format: "10lb  2.3oz"
    if data.contains("lb") && data.contains("oz") {
        // value = lbs for storage (simplified); format tracks PoundsOunces
        let lb_part: Decimal = data
            .split("lb")
            .next()
            .and_then(|s| Decimal::from_str(s.trim()).ok())
            .unwrap_or(Decimal::ZERO);

        let reading = WeightReading {
            value: lb_part,
            unit: WeightUnit::LbOz,
            format: WeightFormat::PoundsOunces,
            display,
            status,
        };
        return Ok(match cmd {
            NciCommand::HighResolution => NciResponse::HighResolution(reading),
            _ => NciResponse::Weight(reading),
        });
    }

    // Detect unit suffix (last 2–3 chars)
    let unit_suffixes = [
        ("lb", WeightUnit::Lb),
        ("kg", WeightUnit::Kg),
        ("oz", WeightUnit::Oz),
        ("g",  WeightUnit::G),
    ];

    let mut value_str = data;
    let mut unit = WeightUnit::Lb; // default
    for (suffix, u) in &unit_suffixes {
        if data.ends_with(suffix) {
            value_str = data[..data.len() - suffix.len()].trim();
            unit = u.clone();
            break;
        }
    }

    let value = if display == DisplayState::Normal {
        Decimal::from_str(value_str)
            .map_err(|e| ScaleError::ParseError(format!("cannot parse weight '{value_str}': {e}")))?
    } else {
        Decimal::ZERO
    };

    let reading = WeightReading { value, unit, format: WeightFormat::Decimal, display, status };
    Ok(match cmd {
        NciCommand::HighResolution => NciResponse::HighResolution(reading),
        _ => NciResponse::Weight(reading),
    })
}

pub fn parse_metrology(frame: &[u8]) -> Result<NciResponse, ScaleError> {
    let (data_bytes, _) = extract_status_bytes(frame)?;
    let s = std::str::from_utf8(&data_bytes)
        .map_err(|e| ScaleError::ParseError(e.to_string()))?
        .trim();
    let raw_counts: u32 = s.parse()
        .map_err(|e| ScaleError::ParseError(format!("bad metrology counts '{s}': {e}")))?;
    Ok(NciResponse::Metrology(MetrologyReading { raw_counts }))
}

pub fn parse_about(frame: &[u8]) -> Result<NciResponse, ScaleError> {
    // Frame: <LF>MMMM,VV-RR,CCCC,xxxxxx<CR><ETX>
    // strip leading LF and trailing CR+ETX
    let inner = frame
        .iter()
        .skip(1) // skip LF
        .take_while(|&&b| b != 0x0D && b != 0x03)
        .cloned()
        .collect::<Vec<u8>>();
    let s = std::str::from_utf8(&inner)
        .map_err(|e| ScaleError::ParseError(e.to_string()))?;
    let parts: Vec<&str> = s.splitn(4, ',').collect();
    if parts.len() < 3 {
        return Err(ScaleError::ParseError(format!("malformed About response: '{s}'")));
    }
    Ok(NciResponse::About(crate::types::AboutInfo {
        model: parts[0].to_string(),
        version: parts[1].to_string(),
        capacity: parts[2].to_string(),
        load_cell_serial: parts.get(3).map(|s| s.to_string()),
    }))
}

pub fn parse_diagnostic(frame: &[u8]) -> Result<NciResponse, ScaleError> {
    // Format: <LF>SSS,CCC,OOO,nnnnnn,ssssss,zzzzzz,x.xxxx,SWT<CR><ETX>
    use rust_decimal::Decimal;
    let inner = frame.iter().skip(1).take_while(|&&b| b != 0x0D && b != 0x03)
        .cloned().collect::<Vec<u8>>();
    let s = std::str::from_utf8(&inner)
        .map_err(|e| ScaleError::ParseError(e.to_string()))?;
    let p: Vec<&str> = s.splitn(8, ',').collect();
    if p.len() < 8 {
        return Err(ScaleError::ParseError(format!("malformed Diagnostic: '{s}'")));
    }
    let parse_u32 = |v: &str| v.trim().parse::<u32>()
        .map_err(|e| ScaleError::ParseError(format!("bad u32 '{v}': {e}")));
    let parse_dec = |v: &str| Decimal::from_str(v.trim())
        .map_err(|e| ScaleError::ParseError(format!("bad decimal '{v}': {e}")));
    Ok(NciResponse::Diagnostic(crate::types::DiagnosticInfo {
        power_on_starts:     parse_u32(p[0])?,
        calibrations:        parse_u32(p[1])?,
        overcapacity_events: parse_u32(p[2])?,
        normalized_counts:   parse_u32(p[3])?,
        span_counts:         parse_u32(p[4])?,
        zero_counts:         parse_u32(p[5])?,
        cal_gravity:         parse_dec(p[6])?,
        span_weight:         p[7].trim().to_string(),
    }))
}
```

- [ ] **Step 4: Run tests — expect pass**

```bash
cargo test -p scale-bridge-scp01
```
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/scale-bridge-scp01/src/
git commit -m "feat(scp01): implement weight, metrology, about, diagnostic parsers"
```

---

### Task 9: Proptest fuzz tests for NCI parser

**Files:**
- Create: `crates/scale-bridge-scp01/tests/fuzz.rs`

- [ ] **Step 1: Write proptest fuzz tests**

```rust
// crates/scale-bridge-scp01/tests/fuzz.rs
use proptest::prelude::*;
use scale_bridge_scp01::{NciCommand, NciProtocol};
use scale_bridge_core::Protocol;

proptest! {
    #[test]
    fn weight_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        let p = NciProtocol;
        let _ = p.decode_response(&NciCommand::Weight, &bytes);
    }

    #[test]
    fn status_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..32)) {
        let _ = scale_bridge_scp01::parser::status::parse_status_bytes(&bytes);
    }

    #[test]
    fn about_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..128)) {
        let p = NciProtocol;
        let _ = p.decode_response(&NciCommand::About, &bytes);
    }

    #[test]
    fn diagnostic_parser_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        let p = NciProtocol;
        let _ = p.decode_response(&NciCommand::Diagnostic, &bytes);
    }
}
```

- [ ] **Step 2: Make parser modules pub for test access**

In `crates/scale-bridge-scp01/src/lib.rs`, change:
```rust
pub mod parser;
```
And in `crates/scale-bridge-scp01/src/parser/mod.rs`, change:
```rust
pub mod status;
pub mod weight;
```

- [ ] **Step 3: Run fuzz tests**

```bash
cargo test -p scale-bridge-scp01 --test fuzz
```
Expected: all proptest cases pass (no panics)

- [ ] **Step 4: Commit**

```bash
git add crates/scale-bridge-scp01/tests/
git commit -m "test(scp01): add proptest fuzz tests for parser"
```

---

### Task 10: SerialTransport and TcpTransport

**Files:**
- Create: `crates/scale-bridge-core/src/transport/serial.rs`
- Create: `crates/scale-bridge-core/src/transport/tcp.rs`

- [ ] **Step 1: Implement SerialTransport**

```rust
// crates/scale-bridge-core/src/transport/serial.rs
#[cfg(feature = "serial")]
mod inner {
    use std::io::{self, Read, Write};
    use std::time::Duration;
    use crate::{ScaleError, transport::Transport};

    pub struct SerialTransport {
        port: Box<dyn serialport::SerialPort>,
    }

    impl SerialTransport {
        pub fn open(port_name: &str, baud_rate: u32) -> Result<Self, ScaleError> {
            let port = serialport::new(port_name, baud_rate)
                .data_bits(serialport::DataBits::Seven)
                .parity(serialport::Parity::None)
                .stop_bits(serialport::StopBits::One)
                .timeout(Duration::from_secs(2))
                .open()
                .map_err(|e| ScaleError::SerialPort(e.to_string()))?;
            Ok(Self { port })
        }
    }

    impl Read for SerialTransport {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.port.read(buf)
        }
    }

    impl Write for SerialTransport {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.port.write(buf) }
        fn flush(&mut self) -> io::Result<()> { self.port.flush() }
    }

    impl Transport for SerialTransport {
        fn set_timeout(&mut self, timeout: Duration) -> Result<(), ScaleError> {
            self.port.set_timeout(timeout)
                .map_err(|e| ScaleError::SerialPort(e.to_string()))
        }
        fn flush_output(&mut self) -> Result<(), ScaleError> {
            self.port.flush().map_err(Into::into)
        }
    }
}

#[cfg(feature = "serial")]
pub use inner::SerialTransport;
```

- [ ] **Step 2: Implement TcpTransport**

```rust
// crates/scale-bridge-core/src/transport/tcp.rs
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use crate::{ScaleError, transport::Transport};

pub struct TcpTransport {
    stream: TcpStream,
}

impl TcpTransport {
    pub fn connect(host: &str, port: u16) -> Result<Self, ScaleError> {
        let stream = TcpStream::connect((host, port))
            .map_err(|e| ScaleError::Transport(e))?;
        Ok(Self { stream })
    }
}

impl Read for TcpTransport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.stream.read(buf) }
}

impl Write for TcpTransport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.stream.write(buf) }
    fn flush(&mut self) -> io::Result<()> { self.stream.flush() }
}

impl Transport for TcpTransport {
    fn set_timeout(&mut self, timeout: Duration) -> Result<(), ScaleError> {
        self.stream.set_read_timeout(Some(timeout)).map_err(Into::into)?;
        self.stream.set_write_timeout(Some(timeout)).map_err(Into::into)
    }
    fn flush_output(&mut self) -> Result<(), ScaleError> {
        self.stream.flush().map_err(Into::into)
    }
}
```

- [ ] **Step 3: Export from transport/mod.rs**

```rust
// crates/scale-bridge-core/src/transport/mod.rs
use std::io::{Read, Write};
use std::time::Duration;
use crate::ScaleError;

pub trait Transport: Read + Write {
    fn set_timeout(&mut self, timeout: Duration) -> Result<(), ScaleError>;
    fn flush_output(&mut self) -> Result<(), ScaleError>;
}

pub mod mock;
pub mod tcp;
pub use mock::MockTransport;
pub use tcp::TcpTransport;

#[cfg(feature = "serial")]
pub mod serial;
#[cfg(feature = "serial")]
pub use serial::SerialTransport;
```

- [ ] **Step 4: Export from lib.rs**

```rust
// crates/scale-bridge-core/src/lib.rs  (add to exports)
#[cfg(feature = "serial")]
pub use transport::SerialTransport;
pub use transport::TcpTransport;
```

- [ ] **Step 5: Run all core tests**

```bash
cargo test -p scale-bridge-core
```
Expected: all pass

- [ ] **Step 6: Commit**

```bash
git add crates/scale-bridge-core/src/transport/
git commit -m "feat(core): add SerialTransport and TcpTransport"
```

---

### Task 11: CLI — args.rs and transport_builder.rs

**Files:**
- Create: `crates/scale-bridge-cli/src/args.rs`
- Create: `crates/scale-bridge-cli/src/transport_builder.rs`
- Modify: `crates/scale-bridge-cli/src/main.rs`

- [ ] **Step 1: Write args.rs**

```rust
// crates/scale-bridge-cli/src/args.rs
use clap::{Parser, Subcommand, ValueEnum};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "scale-bridge", about = "Avery WeighTronix scale CLI")]
pub struct Cli {
    /// Serial port (e.g. /dev/ttyUSB0 or COM3)
    #[arg(long, conflicts_with = "host")]
    pub serial_port: Option<String>,

    /// Baud rate for serial connection
    #[arg(long, default_value = "9600")]
    pub baud: u32,

    /// TCP hostname (for scales with Ethernet)
    #[arg(long, conflicts_with = "port")]
    pub host: Option<String>,

    /// TCP port number
    #[arg(long = "tcp-port", default_value = "3001")]
    pub tcp_port: u16,

    /// Suppress timestamps and color (for systemd/journald)
    #[arg(long)]
    pub systemd: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Read current weight
    Weight {
        #[arg(long, short)]
        watch: bool,
        #[arg(long, default_value = "1s", value_parser = parse_duration)]
        interval: Duration,
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Read scale status
    Status {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Zero the scale
    Zero,
    /// Tare the scale
    Tare,
    /// Switch units
    Units,
    /// Read high-resolution weight
    HighResolution {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Read raw metrology counts
    Metrology {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Read model/version info (7600 series)
    About {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Read diagnostic data (7600 series)
    Diagnostic {
        #[arg(long, short, default_value = "text")]
        output: OutputFormat,
    },
    /// Start HTTPS REST server
    Serve {
        #[arg(long, default_value = "8443")]
        port: u16,
        #[arg(long)]
        scale_port: Option<String>,
        #[arg(long)]
        cert: Option<String>,
        #[arg(long)]
        key: Option<String>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    if let Some(ms) = s.strip_suffix("ms") {
        ms.parse::<u64>().map(Duration::from_millis)
            .map_err(|e| e.to_string())
    } else if let Some(secs) = s.strip_suffix('s') {
        secs.parse::<u64>().map(Duration::from_secs)
            .map_err(|e| e.to_string())
    } else {
        s.parse::<u64>().map(Duration::from_secs)
            .map_err(|e| e.to_string())
    }
}
```

- [ ] **Step 2: Write transport_builder.rs**

```rust
// crates/scale-bridge-cli/src/transport_builder.rs
use scale_bridge_core::ScaleError;
use crate::args::Cli;

pub enum AnyTransport {
    #[cfg(feature = "serial")]
    Serial(scale_bridge_core::SerialTransport),
    Tcp(scale_bridge_core::TcpTransport),
    #[cfg(feature = "mock")]
    Mock(scale_bridge_core::MockTransport),
}

use std::io::{Read, Write};
use std::time::Duration;
use scale_bridge_core::Transport;

impl Read for AnyTransport {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.read(buf),
            AnyTransport::Tcp(t) => t.read(buf),
            #[cfg(feature = "mock")]
            AnyTransport::Mock(t) => t.read(buf),
        }
    }
}

impl Write for AnyTransport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.write(buf),
            AnyTransport::Tcp(t) => t.write(buf),
            #[cfg(feature = "mock")]
            AnyTransport::Mock(t) => t.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.flush(),
            AnyTransport::Tcp(t) => t.flush(),
            #[cfg(feature = "mock")]
            AnyTransport::Mock(t) => t.flush(),
        }
    }
}

impl Transport for AnyTransport {
    fn set_timeout(&mut self, d: Duration) -> Result<(), ScaleError> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.set_timeout(d),
            AnyTransport::Tcp(t) => t.set_timeout(d),
            #[cfg(feature = "mock")]
            AnyTransport::Mock(t) => t.set_timeout(d),
        }
    }
    fn flush_output(&mut self) -> Result<(), ScaleError> {
        match self {
            #[cfg(feature = "serial")]
            AnyTransport::Serial(t) => t.flush_output(),
            AnyTransport::Tcp(t) => t.flush_output(),
            #[cfg(feature = "mock")]
            AnyTransport::Mock(t) => t.flush_output(),
        }
    }
}

pub fn build_transport(cli: &Cli) -> Result<AnyTransport, ScaleError> {
    #[cfg(feature = "mock")]
    if std::env::var("SCALE_BRIDGE_MOCK").is_ok() {
        // Default mock: returns stable 1234.56 lb response
        let resp = b"\x0a  1234.56lb\x0d\x0a\xb0\xb0\x0d\x03".to_vec();
        return Ok(AnyTransport::Mock(scale_bridge_core::MockTransport::with_response(resp)));
    }

    if let Some(host) = &cli.host {
        return Ok(AnyTransport::Tcp(
            scale_bridge_core::TcpTransport::connect(host, cli.tcp_port)?
        ));
    }

    #[cfg(feature = "serial")]
    if let Some(port) = &cli.serial_port {
        return Ok(AnyTransport::Serial(
            scale_bridge_core::SerialTransport::open(port, cli.baud)?
        ));
    }

    Err(ScaleError::Transport(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "specify --port or --host",
    )))
}
```

- [ ] **Step 3: Verify compile**

```bash
cargo build -p scale-bridge-cli
```
Expected: compiles cleanly

- [ ] **Step 4: Commit**

```bash
git add crates/scale-bridge-cli/src/
git commit -m "feat(cli): add CLI argument structures and transport builder"
```

---

### Task 12: CLI output formatters

**Files:**
- Create: `crates/scale-bridge-cli/src/output/mod.rs`
- Create: `crates/scale-bridge-cli/src/output/text.rs`
- Create: `crates/scale-bridge-cli/src/output/json.rs`
- Create: `crates/scale-bridge-cli/src/output/csv.rs`

- [ ] **Step 1: Write output/mod.rs**

```rust
// crates/scale-bridge-cli/src/output/mod.rs
pub mod text;
pub mod json;
pub mod csv;

use scale_bridge_scp01::NciResponse;
use crate::args::OutputFormat;
use scale_bridge_core::ScaleError;

pub fn print_response(response: &NciResponse, format: &OutputFormat) -> Result<(), ScaleError> {
    match format {
        OutputFormat::Text => text::print(response),
        OutputFormat::Json => json::print(response),
        OutputFormat::Csv  => csv::print(response),
    }
}
```

- [ ] **Step 2: Write text.rs**

```rust
// crates/scale-bridge-cli/src/output/text.rs
use scale_bridge_scp01::{NciResponse, types::DisplayState};
use scale_bridge_core::ScaleError;

pub fn print(response: &NciResponse) -> Result<(), ScaleError> {
    match response {
        NciResponse::Weight(w) | NciResponse::HighResolution(w) => {
            match w.display {
                DisplayState::Normal => {
                    println!("{} {}", w.value, w.unit.as_str());
                }
                DisplayState::OverCapacity  => println!("OVER CAPACITY"),
                DisplayState::UnderCapacity => println!("UNDER CAPACITY"),
                DisplayState::ZeroError     => println!("ZERO ERROR"),
            }
            if w.status.motion { eprintln!("[motion]"); }
        }
        NciResponse::Status(s) => {
            println!("motion={} at_zero={} over_cap={} under_cap={}",
                s.motion, s.at_zero, s.over_capacity, s.under_capacity);
            if s.has_error() {
                eprintln!("SCALE ERROR: ram={} rom={} eeprom={} cal={}",
                    s.ram_error, s.rom_error, s.eeprom_error, s.faulty_calibration);
            }
        }
        NciResponse::Acknowledged => println!("OK"),
        NciResponse::Metrology(m) => println!("raw_counts={}", m.raw_counts),
        NciResponse::About(a) => {
            println!("model={} version={} capacity={}", a.model, a.version, a.capacity);
            if let Some(sn) = &a.load_cell_serial { println!("serial={sn}"); }
        }
        NciResponse::Diagnostic(d) => {
            println!("power_on_starts={} calibrations={} overcapacity={}",
                d.power_on_starts, d.calibrations, d.overcapacity_events);
        }
        NciResponse::UnrecognizedCommand => {
            eprintln!("scale did not recognize command");
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Write json.rs**

```rust
// crates/scale-bridge-cli/src/output/json.rs
use scale_bridge_scp01::NciResponse;
use scale_bridge_core::ScaleError;

pub fn print(response: &NciResponse) -> Result<(), ScaleError> {
    let json = serde_json::to_string(response)
        .map_err(|e| ScaleError::ParseError(e.to_string()))?;
    println!("{json}");
    Ok(())
}
```

- [ ] **Step 4: Write csv.rs**

```rust
// crates/scale-bridge-cli/src/output/csv.rs
use scale_bridge_scp01::{NciResponse, types::DisplayState};
use scale_bridge_core::ScaleError;

pub fn print(response: &NciResponse) -> Result<(), ScaleError> {
    let ts = chrono_timestamp();
    match response {
        NciResponse::Weight(w) | NciResponse::HighResolution(w) => {
            let state = match w.display {
                DisplayState::Normal        => "normal",
                DisplayState::OverCapacity  => "over_capacity",
                DisplayState::UnderCapacity => "under_capacity",
                DisplayState::ZeroError     => "zero_error",
            };
            let motion = if w.status.motion { "motion" } else { "stable" };
            println!("{},{},{},{},{}", ts, w.value, w.unit.as_str(), state, motion);
        }
        _ => {
            let json = serde_json::to_string(response)
                .map_err(|e| ScaleError::ParseError(e.to_string()))?;
            println!("{ts},{json}");
        }
    }
    Ok(())
}

fn chrono_timestamp() -> String {
    // RFC 3339 without external deps using std
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple ISO-8601 UTC (seconds precision); for production use `chrono` or `time` crate
    format!("{secs}")
}
```

> Note: Add `chrono = "0.4"` to scale-bridge-cli Cargo.toml for proper timestamps, or keep the unix-seconds stub and upgrade later.

- [ ] **Step 5: Compile check**

```bash
cargo build -p scale-bridge-cli
```

- [ ] **Step 6: Commit**

```bash
git add crates/scale-bridge-cli/src/output/
git commit -m "feat(cli): add text, JSON, and CSV output formatters"
```

---

### Task 13: CLI runner + main.rs

**Files:**
- Create: `crates/scale-bridge-cli/src/runner.rs`
- Modify: `crates/scale-bridge-cli/src/main.rs`

- [ ] **Step 1: Write runner.rs**

```rust
// crates/scale-bridge-cli/src/runner.rs
use std::time::Duration;
use scale_bridge_core::{EtxCodec, Scale, ScaleError};
use scale_bridge_scp01::{NciCommand, NciProtocol};
use crate::args::{Commands, OutputFormat};
use crate::output::print_response;
use crate::transport_builder::AnyTransport;

pub fn run(
    transport: AnyTransport,
    command: &Commands,
) -> Result<(), ScaleError> {
    let mut scale = Scale::new(transport, EtxCodec::new(), NciProtocol);

    match command {
        Commands::Weight { watch, interval, output } => {
            run_command_maybe_watch(&mut scale, NciCommand::Weight, output, *watch, *interval)
        }
        Commands::Status { output } => {
            let resp = scale.send(NciCommand::Status)?;
            print_response(&resp, output)
        }
        Commands::Zero  => { scale.send(NciCommand::Zero)?;  println!("OK"); Ok(()) }
        Commands::Tare  => { scale.send(NciCommand::Tare)?;  println!("OK"); Ok(()) }
        Commands::Units => { scale.send(NciCommand::Units)?; println!("OK"); Ok(()) }
        Commands::HighResolution { output } => {
            let resp = scale.send(NciCommand::HighResolution)?;
            print_response(&resp, output)
        }
        Commands::Metrology { output } => {
            let resp = scale.send(NciCommand::Metrology)?;
            print_response(&resp, output)
        }
        Commands::About { output } => {
            let resp = scale.send(NciCommand::About)?;
            print_response(&resp, output)
        }
        Commands::Diagnostic { output } => {
            let resp = scale.send(NciCommand::Diagnostic)?;
            print_response(&resp, output)
        }
        Commands::Serve { port, .. } => {
            eprintln!("Server mode not yet implemented (port {port})");
            Ok(())
        }
    }
}

fn run_command_maybe_watch<T, C, P>(
    scale: &mut Scale<T, C, P>,
    cmd: scale_bridge_scp01::NciCommand,
    output: &OutputFormat,
    watch: bool,
    interval: Duration,
) -> Result<(), ScaleError>
where
    T: scale_bridge_core::Transport,
    C: scale_bridge_core::Codec,
    P: scale_bridge_core::Protocol<Command = NciCommand, Response = scale_bridge_scp01::NciResponse>,
{
    loop {
        let resp = scale.send(cmd.clone())?;
        print_response(&resp, output)?;
        if !watch { break; }
        std::thread::sleep(interval);
    }
    Ok(())
}
```

- [ ] **Step 2: Write main.rs**

```rust
// crates/scale-bridge-cli/src/main.rs
mod args;
mod output;
mod runner;
mod transport_builder;

use clap::Parser;
use args::Cli;
use transport_builder::build_transport;

fn main() {
    let cli = Cli::parse();

    // Set up tracing
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(!cli.systemd)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    let transport = match build_transport(&cli) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("connection error: {e}");
            std::process::exit(2);
        }
    };

    match runner::run(transport, &cli.command) {
        Ok(()) => {}
        Err(scale_bridge_core::ScaleError::UnrecognizedCommand) => {
            eprintln!("scale did not recognize command");
            std::process::exit(1);
        }
        Err(scale_bridge_core::ScaleError::Timeout)
        | Err(scale_bridge_core::ScaleError::Transport(_))
        | Err(scale_bridge_core::ScaleError::SerialPort(_)) => {
            eprintln!("transport error");
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("parse error: {e}");
            std::process::exit(3);
        }
    }
}
```

- [ ] **Step 3: Build the binary**

```bash
cargo build -p scale-bridge-cli
```
Expected: binary at `target/debug/scale-bridge`

- [ ] **Step 4: Smoke test help output**

```bash
./target/debug/scale-bridge --help
./target/debug/scale-bridge weight --help
```
Expected: help text printed, exits 0

- [ ] **Step 5: Commit**

```bash
git add crates/scale-bridge-cli/src/
git commit -m "feat(cli): add runner and main entry point"
```

---

### Task 14: CLI integration tests

**Files:**
- Create: `crates/scale-bridge-cli/tests/cli.rs`

- [ ] **Step 1: Write integration tests**

```rust
// crates/scale-bridge-cli/tests/cli.rs
use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    let mut c = Command::cargo_bin("scale-bridge").unwrap();
    c.env("SCALE_BRIDGE_MOCK", "1");
    c
}

#[test]
fn weight_text_exits_zero_and_prints_value() {
    cmd()
        .args(["weight"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1234.56"));
}

#[test]
fn weight_json_output_is_valid_json_with_value_field() {
    let output = cmd()
        .args(["weight", "--output", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(output).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(v.get("Weight").is_some() || v.to_string().contains("1234.56"));
}

#[test]
fn weight_csv_output_contains_unit() {
    cmd()
        .args(["weight", "--output", "csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lb"));
}

#[test]
fn zero_command_exits_zero_and_prints_ok() {
    cmd()
        .args(["zero"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

#[test]
fn missing_port_and_host_exits_nonzero() {
    // Without mock env var, no port or host → exit 2
    Command::cargo_bin("scale-bridge")
        .unwrap()
        .args(["weight"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn help_exits_zero() {
    Command::cargo_bin("scale-bridge")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p scale-bridge-cli --test cli --features mock
```
Expected: all 6 tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/scale-bridge-cli/tests/
git commit -m "test(cli): add integration tests using mock transport"
```

---

### Task 15: Server stub + systemd service file

**Files:**
- Modify: `crates/scale-bridge-server/src/lib.rs`
- Create: `scale-bridge.service`

- [ ] **Step 1: Write server stub**

```rust
// crates/scale-bridge-server/src/lib.rs
//! HTTPS REST server for scale-bridge.
//!
//! # Planned REST API
//!
//! GET  /api/weight        → WeightReading as JSON
//! GET  /api/status        → ScaleStatus as JSON
//! POST /api/zero          → 204 No Content
//! POST /api/tare          → 204 No Content
//!
//! # Planned server mode
//!
//! ```bash
//! scale-bridge serve --port 8443 --scale-port /dev/ttyUSB0 \
//!     --cert cert.pem --key key.pem
//! ```
//!
//! # TODO
//! - Choose HTTP library (axum recommended)
//! - TLS via rustls
//! - sd_notify NOTIFY_SOCKET support for systemd Type=notify
//! - SIGTERM graceful shutdown

pub struct ServerConfig {
    pub https_port: u16,
    pub scale_port: Option<String>,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

/// Start the HTTPS server. Not yet implemented.
pub fn serve(_config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    todo!("HTTPS server not yet implemented — see module docs")
}
```

- [ ] **Step 2: Write systemd service file**

```ini
# scale-bridge.service
[Unit]
Description=scale-bridge scale communication service
After=network.target
Documentation=https://github.com/yourusername/scale-bridge

[Service]
Type=simple
ExecStart=/usr/local/bin/scale-bridge --systemd --serial-port /dev/ttyUSB0 weight --watch
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal
SyslogIdentifier=scale-bridge

# Hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict

[Install]
WantedBy=multi-user.target
```

- [ ] **Step 3: Compile check**

```bash
cargo build --workspace
```

- [ ] **Step 4: Commit**

```bash
git add crates/scale-bridge-server/ scale-bridge.service
git commit -m "feat(server): add server stub and systemd service file"
```

---

### Task 16: Justfile + GitHub Actions CI

**Files:**
- Create: `Justfile`
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Write Justfile**

```makefile
# Justfile — run `just` to see available commands

# Run all tests
test:
    cargo nextest run --workspace

# Run tests for a specific crate
test-crate crate:
    cargo nextest run -p {{crate}}

# Lint with clippy (deny warnings)
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check formatting
fmt:
    cargo fmt --all --check

# Fix formatting
fmt-fix:
    cargo fmt --all

# Full CI check (mirrors GitHub Actions)
ci: fmt lint test

# Build all crates
build:
    cargo build --workspace

# Generate and open docs
docs:
    cargo doc --workspace --no-deps --open

# Build release binary
release:
    cargo build --workspace --release

# Show binary size
size:
    ls -lh target/release/scale-bridge 2>/dev/null || echo "run 'just release' first"
```

- [ ] **Step 2: Write GitHub Actions CI**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-nextest
        uses: taiki-e/install-action@nextest

      - name: Install Linux serial deps
        if: matrix.os == 'ubuntu-latest'
        run: sudo apt-get install -y libudev-dev

      - name: Check formatting
        run: cargo fmt --all --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo nextest run --workspace

  docs:
    name: Docs
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: sudo apt-get install -y libudev-dev
      - run: cargo doc --workspace --no-deps
```

- [ ] **Step 3: Commit**

```bash
git add Justfile .github/
git commit -m "ci: add Justfile and GitHub Actions matrix CI"
```

---

### Task 17: Final polish — clippy + fmt pass

- [ ] **Step 1: Fix all clippy warnings**

```bash
cargo clippy --workspace --all-targets --all-features 2>&1
# Fix each warning inline
```

- [ ] **Step 2: Format all code**

```bash
cargo fmt --all
```

- [ ] **Step 3: Run full test suite**

```bash
cargo nextest run --workspace
```
Expected: all tests pass, 0 failures

- [ ] **Step 4: Final commit**

```bash
git add -u
git commit -m "chore: final clippy and fmt pass"
```

---

## Verification Checklist

```bash
# 1. Workspace builds
cargo build --workspace

# 2. Full test suite (all three parsers, codec, integration, fuzz, CLI)
cargo nextest run --workspace

# 3. No clippy warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 4. Formatted
cargo fmt --all --check

# 5. CLI help works
./target/debug/scale-bridge --help
./target/debug/scale-bridge weight --help

# 6. CLI mock mode works
SCALE_BRIDGE_MOCK=1 ./target/debug/scale-bridge weight
SCALE_BRIDGE_MOCK=1 ./target/debug/scale-bridge weight --output json
SCALE_BRIDGE_MOCK=1 ./target/debug/scale-bridge weight --output csv
SCALE_BRIDGE_MOCK=1 ./target/debug/scale-bridge zero

# 7. All three CI platforms pass
# (push to GitHub, check Actions tab)
```
