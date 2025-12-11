#![expect(clippy::type_complexity)]

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

use metrics::CounterFn;

use serial_test::serial;

// Custom counter implementation
struct MockCounter {
    name: String,
    labels: Vec<(String, String)>,
    counters: Arc<Mutex<Vec<(String, Vec<(String, String)>)>>>,
}

impl CounterFn for MockCounter {
    fn increment(&self, _value: u64) {
        self.counters
            .lock()
            .unwrap()
            .push((self.name.clone(), self.labels.clone()));
    }

    fn absolute(&self, _value: u64) {
        self.counters
            .lock()
            .unwrap()
            .push((self.name.clone(), self.labels.clone()));
    }
}

// Custom histogram implementation
struct MockHistogram {
    name: String,
    labels: Vec<(String, String)>,
    histograms: Arc<Mutex<Vec<(String, Vec<(String, String)>, f64)>>>,
}

impl metrics::HistogramFn for MockHistogram {
    fn record(&self, value: f64) {
        self.histograms
            .lock()
            .unwrap()
            .push((self.name.clone(), self.labels.clone(), value));
    }
}

// Mock recorder that captures metric calls
#[derive(Clone, Default)]
struct MockRecorder {
    counters: Arc<Mutex<Vec<(String, Vec<(String, String)>)>>>,
    histograms: Arc<Mutex<Vec<(String, Vec<(String, String)>, f64)>>>,
}

impl MockRecorder {
    fn new() -> Self {
        Self::default()
    }

    fn get_counters(&self) -> Vec<(String, Vec<(String, String)>)> {
        self.counters.lock().unwrap().clone()
    }

    fn get_histograms(&self) -> Vec<(String, Vec<(String, String)>, f64)> {
        self.histograms.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.counters.lock().unwrap().clear();
        self.histograms.lock().unwrap().clear();
    }
}

impl metrics::Recorder for MockRecorder {
    fn describe_counter(
        &self,
        _key: metrics::KeyName,
        _unit: Option<metrics::Unit>,
        _description: metrics::SharedString,
    ) {
    }
    fn describe_gauge(
        &self,
        _key: metrics::KeyName,
        _unit: Option<metrics::Unit>,
        _description: metrics::SharedString,
    ) {
    }
    fn describe_histogram(
        &self,
        _key: metrics::KeyName,
        _unit: Option<metrics::Unit>,
        _description: metrics::SharedString,
    ) {
    }

    fn register_counter(&self, key: &metrics::Key, _metadata: &metrics::Metadata<'_>) -> metrics::Counter {
        let name = key.name().to_string();
        let labels: Vec<_> = key
            .labels()
            .map(|label| (label.key().to_string(), label.value().to_string()))
            .collect();

        let counter = MockCounter {
            name,
            labels,
            counters: self.counters.clone(),
        };

        metrics::Counter::from_arc(Arc::new(counter))
    }

    fn register_gauge(&self, _key: &metrics::Key, _metadata: &metrics::Metadata<'_>) -> metrics::Gauge {
        metrics::Gauge::noop()
    }

    fn register_histogram(&self, key: &metrics::Key, _metadata: &metrics::Metadata<'_>) -> metrics::Histogram {
        let name = key.name().to_string();
        let labels: Vec<_> = key
            .labels()
            .map(|label| (label.key().to_string(), label.value().to_string()))
            .collect();

        let histogram = MockHistogram {
            name,
            labels,
            histograms: self.histograms.clone(),
        };

        metrics::Histogram::from_arc(Arc::new(histogram))
    }
}

// Helper to set up recorder once using OnceLock
static RECORDER: OnceLock<MockRecorder> = OnceLock::new();

fn setup_recorder() -> &'static MockRecorder {
    RECORDER.get_or_init(|| {
        let recorder = MockRecorder::new();

        // Register the `MockRecorder` as the global recorder.
        // Box::leak converts Box<MockRecorder> -> &'static MockRecorder since that is what `set_global_recorder`
        // expects. We ignore the Result since we only call this once and know it will succeed
        let _ = metrics::set_global_recorder(Box::leak(Box::new(recorder.clone())));

        recorder
    })
}

// Test module with the macro applied
mod test_functions {
    use std::time::Duration;

    use measure::measure;

    #[measure(name = "custom_metric", "service" => "test")]
    pub async fn simple_function() -> Result<u32, ()> {
        Ok(42)
    }

    #[measure(name = "custom_metric", "service" => "test", "operation" => "complex")]
    pub async fn function_with_multiple_labels() -> Result<String, ()> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok("done".to_string())
    }

    #[measure(name = "custom_metric")]
    pub async fn function_without_labels() -> Result<bool, ()> {
        Ok(true)
    }

    #[measure(name = "custom_metric", "service" => "test")]
    pub async fn function_that_errors() -> Result<(), &'static str> {
        Err("test error")
    }

    pub struct TestStruct;

    impl TestStruct {
        #[measure(name = "custom_metric", "service" => "test", "context" => "method")]
        pub async fn method_function(&self) -> Result<i32, ()> {
            Ok(100)
        }
    }
}

#[tokio::test]
#[serial(recorder)]
async fn test_simple_function_metrics() {
    let recorder = setup_recorder();
    recorder.clear();

    let result = test_functions::simple_function().await;

    assert_eq!(result, Ok(42));

    let counters = recorder.get_counters();
    assert!(
        counters.iter().any(|(name, labels)| name == "custom_metric_total"
            && labels
                == &vec![
                    ("function".to_string(), "simple_function".to_string()),
                    ("service".to_string(), "test".to_string())
                ]),
        "Counter not found. Got: {:?}",
        counters
    );

    let histograms = recorder.get_histograms();
    assert!(
        histograms
            .iter()
            .any(|(name, labels, duration)| name == "custom_metric_duration_seconds"
                && labels
                    == &vec![
                        ("function".to_string(), "simple_function".to_string()),
                        ("service".to_string(), "test".to_string())
                    ]
                && *duration >= 0.0),
        "Histogram not found. Got: {:?}",
        histograms
    );
}

#[tokio::test]
#[serial(recorder)]
async fn test_function_without_additional_labels() {
    let recorder = setup_recorder();
    recorder.clear();

    let result = test_functions::function_without_labels().await;

    assert_eq!(result, Ok(true));

    let counters = recorder.get_counters();
    assert!(
        counters.iter().any(|(name, labels)| name == "custom_metric_total"
            && labels == &vec![("function".to_string(), "function_without_labels".to_string())]),
        "Counter not found. Got: {:?}",
        counters
    );
}

#[tokio::test]
#[serial(recorder)]
async fn test_function_preserves_errors() {
    let recorder = setup_recorder();
    recorder.clear();

    let result = test_functions::function_that_errors().await;

    assert_eq!(result, Err("test error"));

    let counters = recorder.get_counters();
    assert!(
        counters.iter().any(|(name, labels)| name == "custom_metric_total"
            && labels
                == &vec![
                    ("function".to_string(), "function_that_errors".to_string()),
                    ("service".to_string(), "test".to_string())
                ]),
        "Counter not found. Got: {:?}",
        counters
    );

    let histograms = recorder.get_histograms();
    assert!(
        histograms
            .iter()
            .any(|(name, labels, _)| name == "custom_metric_duration_seconds"
                && labels
                    == &vec![
                        ("function".to_string(), "function_that_errors".to_string()),
                        ("service".to_string(), "test".to_string())
                    ]),
        "Histogram not found. Got: {:?}",
        histograms
    );
}

#[tokio::test]
#[serial(recorder)]
async fn test_method_function() {
    let recorder = setup_recorder();
    recorder.clear();

    let test_struct = test_functions::TestStruct;
    let result = test_struct.method_function().await;

    assert_eq!(result, Ok(100));

    let counters = recorder.get_counters();
    assert!(
        counters.iter().any(|(name, labels)| name == "custom_metric_total"
            && labels.contains(&("function".to_string(), "method_function".to_string()))
            && labels.contains(&("service".to_string(), "test".to_string()))
            && labels.contains(&("context".to_string(), "method".to_string()))),
        "Counter not found. Got: {:?}",
        counters
    );

    let histograms = recorder.get_histograms();
    assert!(
        histograms
            .iter()
            .any(|(name, labels, _)| name == "custom_metric_duration_seconds"
                && labels.contains(&("function".to_string(), "method_function".to_string()))
                && labels.contains(&("service".to_string(), "test".to_string()))
                && labels.contains(&("context".to_string(), "method".to_string()))),
        "Histogram not found. Got: {:?}",
        histograms
    );
}
