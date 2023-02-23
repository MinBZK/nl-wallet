fn main() {
    #[cfg(feature = "hardware")]
    uniffi::generate_scaffolding("./udl/hw_keystore.udl").unwrap();
}
