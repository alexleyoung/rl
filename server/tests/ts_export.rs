// ts-rs generates `export_bindings_*` tests per #[derive(TS)] type inside the
// crate. Running `cargo test -p rl-server` (with TS_RS_EXPORT_DIR set to the
// workspace root) writes the .ts files into client/src/lib/types/. This file
// exists so `just types` has a stable test target name to filter on.

#[test]
fn ts_export_sentinel() {}
