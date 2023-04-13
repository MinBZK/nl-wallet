fn main() {
    // generate Rust bindings from UDL file during build process
    #[cfg(feature = "hardware")]
    uniffi::generate_scaffolding("./udl/hw_keystore.udl").unwrap();
}
