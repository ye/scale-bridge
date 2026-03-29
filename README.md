# scale-bridge

Rust CLI and library for Avery Weigh-Tronix scales speaking the SCP-01/NCI protocol over serial or Ethernet.

## Current Hardware Notes

The implementation has been validated against this device:

- Model: `NCI 6720-15`
- Capacity: `15 kg / 30 lb`

Observed device-specific behavior on that unit:

- Serial parity must be `even`. The CLI now defaults to `--parity even`.
- `weight` replies use uppercase units such as `LB`.
- `status` replies may be standalone ASCII frames such as `S00`.
- `weight` may also reply with a standalone ASCII status frame such as `S01` instead of a weight payload.
- Unsupported commands reply with framed `?` responses like `LF ? CR ETX`.

## Supported Commands On Tested Hardware

Verified working:

- `weight`
- `status`
- `zero`

Verified unsupported on the tested `NCI 6720-15`:

- `tare`
- `metrology`
- `about`
- `diagnostic`

Unsupported commands were observed returning the framed `?` response on the wire.

## Operational Notes For NCI 6720-15

- If the metal weigh tray is lifted or removed, `weight` may return a status-only response instead of a numeric weight.
- On the tested unit, a standalone `S01` reply was observed in that state and maps to `at_zero=true`.
- `zero` is recognized by the scale, but it may not actually zero the displayed weight if the current load is outside the scale's allowed zero window.
- In testing, issuing `zero` with about `1.32 lb` present returned a status reply and the follow-up weight remained non-zero.
- Treat this as device behavior or configuration/calibration state, not a transport or parser failure.

## Serial Usage

Typical invocation for the tested unit:

```bash
./target/debug/scale-bridge --port /dev/ttyUSB0 --baud 9600 weight
```

The default serial settings used by the CLI are:

- `--baud 9600`
- `--parity even`
- `7 data bits`
- `1 stop bit`

## Debugging Wire Responses

Enable wire logging with `--verbose 1` or higher:

```bash
./target/debug/scale-bridge --port /dev/ttyUSB0 --verbose 1 weight
```

Example observed traffic on the tested unit:

```text
DEBUG tx: 57 0D
DEBUG rx: 0A 30 30 31 2E 33 34 4C 42 0D 0A 53 30 30 0D 03
```

That response decodes to `1.34 lb` with ASCII status `S00`.
