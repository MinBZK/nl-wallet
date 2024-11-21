use std::concat;
use std::env;

use lib_flutter_rust_bridge_codegen::config_parse;
use lib_flutter_rust_bridge_codegen::frb_codegen;
use lib_flutter_rust_bridge_codegen::get_symbols_if_no_duplicates;
use lib_flutter_rust_bridge_codegen::RawOpts;

/// Path of input Rust code
const RUST_INPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../flutter_api/src/api.rs");
/// Path of output generated Dart code
const DART_OUTPUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../wallet_app/packages/wallet_core/lib/src/bridge_generated.dart"
);
/// Path where Rust crate can be found
const RUST_CRATE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../flutter_api");
/// Path of output generated C code
const C_OUTPUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../wallet_app/ios/Runner/bridge_generated.h"
);
/// Path of output generated Rust code
const RUST_OUTPUT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../flutter_api/src/bridge_generated/bridge.rs"
);

fn main() {
    // Options for frb_codegen
    let raw_opts = RawOpts {
        rust_input: vec![RUST_INPUT.to_string()],
        dart_output: vec![DART_OUTPUT.to_string()],
        dart_format_line_length: 120,
        class_name: Some(vec!["WalletCore".to_string()]),
        rust_crate_dir: Some(vec![RUST_CRATE_DIR.to_string()]),
        c_output: Some(vec![C_OUTPUT.to_string()]),
        rust_output: Some(vec![RUST_OUTPUT.to_string()]),
        skip_add_mod_to_lib: true,

        // for other options use defaults
        ..Default::default()
    };

    // get opts from raw opts
    let all_configs = config_parse(raw_opts);

    // generation of rust api for ffi (single block)
    let all_symbols = get_symbols_if_no_duplicates(&all_configs).unwrap();
    assert_eq!(all_configs.len(), 1);
    frb_codegen(&all_configs[0], &all_symbols).unwrap();
}
