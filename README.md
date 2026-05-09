# rust-wsman-messages

Rust crate bindings for Intel AMT WS-Management, mirroring the Go library
at https://github.com/device-management-toolkit/go-wsman-messages.

## Status

POC — only `AMT_GeneralSettings` is wired up. The long-term plan is to
generate the rest from the Go library via a codegen tool. See the design
spec in the sibling Go repo at
`docs/superpowers/specs/2026-04-17-rust-wsman-poc-design.md`.

## Crates

- `wsman-core` — protocol-agnostic WS-MAN runtime: HTTP + digest auth,
  SOAP envelope builder, generic `WsmanService<T>`.
- `wsman-amt` — AMT-specific class bindings (POC ships only `general`).

## Toolchain

The workspace pins `channel = "stable"` in `rust-toolchain.toml`. `rustup`
will auto-install the current stable on first `cargo` invocation. The MSRV
we claim to support (via `workspace.package.rust-version`) is `1.88`.

## Example

Run against a real AMT endpoint:

    AMT_ENDPOINT=https://10.0.0.5:16993/wsman \
    AMT_USER=admin AMT_PASSWORD='P@ssw0rd' \
    cargo run -p wsman-amt --example get_general_settings

See `crates/wsman-amt/examples/get_general_settings.rs`.

## Test

    cargo test --workspace

## License

Apache-2.0.
