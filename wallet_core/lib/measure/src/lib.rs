use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Ident;
use syn::LitStr;
use syn::Result;
use syn::Token;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;

struct MeasureArgs {
    name: LitStr,
    labels: TokenStream2,
}

impl Parse for MeasureArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse: name = "metric_name"
        let name_ident: Ident = input.parse()?;
        if name_ident != "name" {
            return Err(syn::Error::new_spanned(name_ident, "expected `name` parameter"));
        }
        let _: Token![=] = input.parse()?;
        let name: LitStr = input.parse()?;

        // Parse optional comma and remaining labels
        let labels = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            input.parse()?
        } else {
            TokenStream2::new()
        };

        Ok(MeasureArgs { name, labels })
    }
}

/// Instruments an async function with execution metrics.
///
/// This attribute macro automatically records two metrics for the annotated function:
/// - A **counter** named `{name}_total` that increments on each function call
/// - A **histogram** named `{name}_duration_seconds` that records the execution time in seconds
///
/// Both metrics include a `name` label with the function name, plus any additional
/// labels specified in the macro invocation.
///
/// # Syntax
///
/// ```ignore
/// #[measure(name = "metric_name")]
/// #[measure(name = "metric_name", "label_key" => "label_value", ...)]
/// ```
///
/// # Arguments
///
/// * `name` - **Required**. The base name for the metrics. The macro will append `_total` for the counter and
///   `_duration_seconds` for the histogram.
///
/// * Additional labels - **Optional**. Any number of key-value pairs in the format `"key" => "value"` that will be
///   added as metric labels. These follow the same syntax as the `metrics` crate's `counter!` and `histogram!` macros.
///
/// # Labels
///
/// All metrics automatically include:
/// - `function`: The name of the annotated function
///
/// Additional labels provided in the macro invocation are added to both metrics.
///
/// # Examples
///
/// ## Basic usage with just a metric name
///
/// ```ignore
/// use measure::measure;
///
/// #[measure(name = "database_query")]
/// async fn fetch_user(id: u64) -> Result<User, Error> {
///     // Function implementation
/// }
/// ```
///
/// This generates:
/// - Counter: `database_query_total` with label `method = "fetch_user"`
/// - Histogram: `database_query_elapsed` with label `method = "fetch_user"`
///
/// ## With additional labels
///
/// ```ignore
/// use measure::measure;
///
/// #[measure(name = "hsm_operation", "service" => "pkcs11", "operation" => "sign")]
/// async fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>, HsmError> {
///     // Function implementation
/// }
/// ```
///
/// This generates:
/// - Counter: `hsm_operation_total` with labels:
///   - `function = "sign_data"`
///   - `service = "pkcs11"`
///   - `operation = "sign"`
/// - Histogram: `hsm_operation_duration_seconds` with the same labels
///
/// ## On struct methods
///
/// ```ignore
/// use measure::measure;
///
/// impl MyService {
///     #[measure(name = "service_call", "component" => "backend")]
///     pub async fn process(&self, request: Request) -> Result<Response, Error> {
///         // Implementation
///     }
/// }
/// ```
///
/// # Requirements
///
/// - The annotated function must be `async`
/// - The function must return a value (can be `Result`, `Option`, or any other type)
/// - The `metrics` crate must be available in scope (with `counter!` and `histogram!` macros)
///
/// # Error Handling
///
/// The macro preserves the function's return value, including errors. Metrics are recorded
/// **regardless of whether the function succeeds or fails**. This means:
/// - The counter increments even if the function returns an error
/// - The histogram records the elapsed time up to the point of error
///
/// This behavior ensures you can track error rates and latencies for failing operations.
///
/// # Performance Considerations
///
/// - The macro adds minimal overhead: two metric recordings and one timestamp operation
/// - Metric recording is typically very fast (sub-microsecond) with most metric backends
/// - The histogram records time in seconds as `f64`
/// - No heap allocations are added by the macro itself
///
/// # Thread Safety
///
/// The macro is safe to use with multi-threaded async runtimes like Tokio. The `function` label
/// and user-provided labels are string literals embedded at compile time, so there are no
/// runtime thread-safety concerns.
///
/// # Limitations
///
/// - Cannot be applied to non-async functions (will result in a compile error)
/// - Cannot be applied to trait definitions (only to implementations)
/// - The metric name must be a string literal (cannot be a variable or computed at runtime)
/// - Label values must be string literals or expressions valid in the `metrics!` macro syntax
#[proc_macro_attribute]
pub fn measure(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as MeasureArgs);
    let input = parse_macro_input!(item as syn::ItemFn);

    let metric_name = args.name.value();
    let additional_labels = args.labels;

    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let attrs = &input.attrs;

    // Build the output with function name added to labels
    let output = if additional_labels.is_empty() {
        // Only function label
        quote! {
            #(#attrs)*
            #vis #sig {
                ::metrics::counter!(
                    concat!(#metric_name, "_total"),
                    "function" => #fn_name_str
                ).increment(1);

                let start = ::std::time::Instant::now();
                let result = async move #block.await;
                let duration = start.elapsed().as_secs_f64();

                ::metrics::histogram!(
                    concat!(#metric_name, "_duration_seconds"),
                    "function" => #fn_name_str
                ).record(duration);

                result
            }
        }
    } else {
        // Function label + additional labels
        quote! {
            #(#attrs)*
            #vis #sig {
                ::metrics::counter!(
                    concat!(#metric_name, "_total"),
                    "function" => #fn_name_str, #additional_labels
                ).increment(1);

                let start = ::std::time::Instant::now();
                let result = async move #block.await;
                let duration = start.elapsed().as_secs_f64();

                ::metrics::histogram!(
                    concat!(#metric_name, "_duration_seconds"),
                    "function" => #fn_name_str, #additional_labels
                ).record(duration);

                result
            }
        }
    };

    output.into()
}
