fn main() {
    // generate Rust bindings from UDL file during build process
    uniffi::generate_scaffolding("./udl/platform_support.udl").unwrap();
}
