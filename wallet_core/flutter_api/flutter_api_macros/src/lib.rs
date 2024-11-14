use proc_macro::TokenStream;
use quote::quote;
use quote::quote_spanned;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::ItemFn;

/// Converts the body of a function to an asynchronous task and executes it on the flutter_api's tokio runtime.
/// This macro can only be applied in the `flutter_api` crate, because it generates code using `crate::async_runtime`.
/// This macro also binds the Sentry [`Hub`], to mitigate the concerns raised in
/// https://docs.rs/sentry-core/0.34.0/sentry_core/index.html#parallelism-concurrency-and-async
#[proc_macro_attribute]
pub fn async_runtime(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = parse_macro_input!(item as ItemFn);
    let stmts = &block.stmts;

    if sig.asyncness.is_none() {
        return quote_spanned! { sig.span() =>
            compile_error!("The `async_runtime` macro can only be applied to async functions.");
        }
        .into();
    }

    sig.asyncness = None;

    quote! {
        #(#attrs)* #vis #sig {
            crate::async_runtime::get_async_runtime().block_on(
                ::sentry::SentryFutureExt::bind_hub(
                    async {
                        #(#stmts)*
                    },
                    ::sentry::Hub::new_from_top(::sentry::Hub::current())
                )
            )
        }
    }
    .into()
}

/// This macro is to be used for all API functions that are exposed to Flutter
/// via `flutter-rust-bridge`. Unfortunately, the bridging code generated can only
/// propagate error values as Dart exceptions if the [`anyhow::Result`] type is used.
/// In order to transmit error details to Flutter, the [`anyhow::Error`] type will
/// contain a JSON encoded string for errors we know the [`wallet::wallet::Wallet`] to
/// produce.
///
/// For this macro to make sense in the context in which it is used, its input should
/// also be a [`anyhow::Result`]. This way, the function to which it is applied will
/// actually seen to return the correct result type.
///
/// To convert one [`anyhow::Result`] to another [`anyhow::Result`] this macro takes
/// the following steps:
///
/// 1. Wrap the function to which is is applied in a closure that is called immediately. The effect of this is that any
///    use of the `?` operator to convert and return error values is contained and performed within the closure.
/// 2. Any [`anyhow::Error`] resulting from the closure is converted to a [`crate::errors::FlutterApiError`], using its
///    [`TryFrom`] trait implementation. If this conversion fails, the [`anyhow::Error`] is simply propagated.
/// 3. The [`crate::errors::FlutterApiError`] is logged using [`tracing`], by using the [`Display`] and [`Debug`] traits
///    on the error.
/// 4. A new [`anyhow::Error`] is created and propagated, containing the [`crate::errors::FlutterApiError`] encoded as
///    JSON as its message. On the Flutter side, this message can be extracted from the resulting `FfiException`.
#[proc_macro_attribute]
pub fn flutter_api_error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(item as ItemFn);

    quote! {
        #(#attrs)* #vis #sig {
            (|| #block)()
                .map_err(|error: ::anyhow::Error|
                    match crate::errors::FlutterApiError::try_from(error) {
                        Ok(flutter_error) => {
                            ::tracing::warn!("Error: {}", flutter_error);
                            ::tracing::info!("Error details: {:?}", flutter_error);

                            ::anyhow::anyhow!(flutter_error.to_json())
                        },
                        Err(error) => error,
                    }
                )
        }
    }
    .into()
}
