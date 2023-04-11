use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Converts the body of a function to an asynchronous task and executes it on the flutter_api's tokio runtime.
/// This macro can only be applied in the `flutter_api` crate, because it generates code using `crate::async_runtime`.
#[proc_macro_attribute]
pub fn async_runtime(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = parse_macro_input!(item as ItemFn);
    let stmts = &block.stmts;

    sig.asyncness = match sig.asyncness {
        Some(_) => None,
        None => panic!("The `async_runtime` macro can only be applied to async fn's."),
    };

    quote! {
        #(#attrs)* #vis #sig {
            crate::async_runtime::get_async_runtime().block_on(async {
                #(#stmts)*
            })
        }
    }
    .into()
}
