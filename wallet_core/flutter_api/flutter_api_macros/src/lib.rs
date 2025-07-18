use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;
use syn::parse_macro_input;

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
/// 1. Wrap the function to which is is applied in a closure that is called immediately. This function may or may not be
///    `async`. The effect of this is that any use of the `?` operator to convert and return error values is contained
///    and performed within the closure.
/// 2. Any [`anyhow::Error`] resulting from the closure is converted to a [`crate::errors::FlutterApiError`], using its
///    [`TryFrom`] trait implementation. If this conversion fails, the [`anyhow::Error`] is simply propagated.
/// 3. The [`crate::errors::FlutterApiError`] is logged using [`tracing`], by using the [`Display`] and [`Debug`] traits
///    on the error.
/// 4. A new [`anyhow::Error`] is created and propagated, containing the [`crate::errors::FlutterApiError`] encoded as
///    JSON as its message. On the Flutter side, this message can be extracted from the resulting `FfiException`.
#[proc_macro_attribute]
pub fn flutter_api_error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(item as ItemFn);

    // Wrap the block of the original function in a closure that is called.
    let wrapped_block = if sig.asyncness.is_some() {
        quote! {
            (|| async #block)().await
        }
    } else {
        quote! {
            (|| #block)()
        }
    };

    // Attempt to perform error conversion, which always results in `anyhow::Error`.
    let stream = quote! {
        #(#attrs)* #vis #sig {
            #wrapped_block.map_err(|error: ::anyhow::Error|

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
    };

    stream.into()
}
