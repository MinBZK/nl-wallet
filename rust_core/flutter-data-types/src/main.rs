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

// Create Dart class definitions.
fn generate_dart(registry: &Registry) {
    let config = serde_generate::CodeGeneratorConfig::new("core_domain".to_string())
        .with_encodings(vec![serde_generate::Encoding::Bincode])
        .with_c_style_enums(true);
    let generator = serde_generate::dart::CodeGenerator::new(&config);
    let _result = generator.output(PathBuf::from("../../pub/core_domain"), registry);
}
