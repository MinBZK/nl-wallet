use flutter_data_types::*;
use serde_reflection::{Registry, Tracer, TracerConfig};
use std::path::PathBuf;

fn main() {
    let mut tracer = Tracer::new(TracerConfig::default());
    tracer.trace_simple_type::<PinResult>().unwrap();
    tracer.trace_simple_type::<PinError>().unwrap();
    let registry = tracer.registry().unwrap();
    // generate_java(&registry);
    generate_dart(&registry);
    // generate_swift(&registry);
}

// Create Java class definitions.
fn generate_java(registry: &Registry) {
    let config =
        serde_generate::CodeGeneratorConfig::new("nl.rijksoverheid.edi.wallet".to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode])
            .with_c_style_enums(true);
    let generator = serde_generate::java::CodeGenerator::new(&config);
    let _result = generator.write_source_files(PathBuf::from("./generated/java/"), &registry);
}

// Create Dart class definitions.
fn generate_dart(registry: &Registry) {
    let config = serde_generate::CodeGeneratorConfig::new("core_domain".to_string())
        .with_encodings(vec![serde_generate::Encoding::Bincode])
        .with_c_style_enums(true);
    let generator = serde_generate::dart::CodeGenerator::new(&config);
    let _result = generator.output(PathBuf::from("../../pub/core_domain"), &registry);
}

// Create Swift class definitions.
fn generate_swift(registry: &Registry) {
    let mut source = Vec::new();
    let config =
        serde_generate::CodeGeneratorConfig::new("nl.rijksoverheid.edi.wallet".to_string())
            .with_c_style_enums(true)
            .with_encodings(vec![serde_generate::Encoding::Bincode]);
    let generator = serde_generate::swift::CodeGenerator::new(&config);
    // let _result = generator.output(PathBuf::from("./generated/swift/"), &registry);
    let _result = generator.output(&mut source, &registry);
    println!("{}", String::from_utf8_lossy(&source))
}
