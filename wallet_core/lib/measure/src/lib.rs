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
/// This attribute macro automatically records three metrics for the annotated function:
/// - A **counter** named `{name}_total` that increments on each function call
/// - A **counter** named `{name}_failures_total` that increments only when the function returns an `Err`
/// - A **histogram** named `{name}_duration_seconds` that records the execution time in seconds
///
/// Both the total counter and the histogram include a `function` label with the function name,
/// plus any additional labels specified in the macro invocation.
/// The histogram also includes a `failure` label ("true" or "false") indicating whether the function failed.
/// The failures counter only includes the `function` label and additional labels (no `failure` label).
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
/// * `name` - **Required**. The base name for the metrics. The macro will append `_total` for the counter,
///   `_failures_total` for the failure counter, and `_duration_seconds` for the histogram.
///
/// * Additional labels - **Optional**. Any number of key-value pairs in the format `"key" => "value"` that will be
///   added as metric labels. These follow the same syntax as the `metrics` crate's `counter!` and `histogram!` macros.
///
/// # Labels
///
/// All metrics automatically include:
/// - `function`: The name of the annotated function
///
/// The histogram additionally includes:
/// - `failure`: "true" if the function returned an `Err`, "false" otherwise
///
/// Additional labels provided in the macro invocation are added to all metrics.
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
/// - Counter: `database_query_total` with label `function = "fetch_user"`
/// - Counter: `database_query_failures_total` with label `function = "fetch_user"` (only on errors)
/// - Histogram: `database_query_duration_seconds` with labels `function = "fetch_user"` and `failure = "true"/"false"`
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
/// - Counter: `hsm_operation_failures_total` with the same labels (only on errors)
/// - Histogram: `hsm_operation_duration_seconds` with the same labels plus `failure = "true"/"false"`
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
/// - The total counter increments on every call
/// - The failures counter increments only when the function returns an `Err` variant
/// - The histogram records the elapsed time with the appropriate `failure` label
///
/// This behavior ensures you can track error rates and latencies for both successful and failing operations.
///
/// # Performance Considerations
///
/// - The macro adds minimal overhead: two or three metric recordings and one timestamp operation
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
/// - Failure detection only works for functions returning `Result<T, E>` types
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
    let additional_labels_with_comma = if additional_labels.is_empty() {
        quote! {}
    } else {
        quote! { , #additional_labels }
    };

    // Check if return type is Result to determine if we should track failures
    let return_type = &input.sig.output;
    let is_result = match return_type {
        syn::ReturnType::Type(_, ty) => {
            // Check if the type path contains "Result"
            matches!(ty.as_ref(), syn::Type::Path(type_path)
                if type_path.path.segments.last().map(|s| s.ident == "Result").unwrap_or(false))
        }
        _ => false,
    };

    let failure_tracking = if is_result {
        quote! {
            let is_error = match &result {
                Ok(_) => false,
                Err(_) => true,
            };

            if is_error {
                ::metrics::counter!(
                    concat!(#metric_name, "_failures_total"),
                    "function" => #fn_name_str #additional_labels_with_comma
                ).increment(1);
            }

            ::metrics::histogram!(
                concat!(#metric_name, "_duration_seconds"),
                "function" => #fn_name_str #additional_labels_with_comma,
                "failure" => if is_error { "true" } else { "false" }
            ).record(duration);
        }
    } else {
        quote! {
            ::metrics::histogram!(
                concat!(#metric_name, "_duration_seconds"),
                "function" => #fn_name_str #additional_labels_with_comma
            ).record(duration);
        }
    };

    let output = quote! {
        #(#attrs)*
        #vis #sig {
            ::metrics::counter!(
                concat!(#metric_name, "_total"),
                "function" => #fn_name_str #additional_labels_with_comma
            ).increment(1);

            let start = ::std::time::Instant::now();
            let result = async move #block.await;
            let duration = start.elapsed().as_secs_f64();

            #failure_tracking

            result
        }
    };

    output.into()
}
