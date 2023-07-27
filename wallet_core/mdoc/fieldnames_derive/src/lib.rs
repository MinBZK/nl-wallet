use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(FieldNames)]
pub fn derive_field_names(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let fieldnames = field_names(&input.data);

    let expanded = quote! {
        impl fieldnames::FieldNames for #name {
            fn field_names() -> Vec<String> {
                #fieldnames
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

/// Given a struct, return code creating a Vec that contains the struct's fieldnames.
fn field_names(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let names = fields.named.iter().map(|f| {
                    let name = f.ident.to_token_stream().to_string();
                    quote! {
                        #name.to_string()
                    }
                });
                quote! {
                    vec![#(#names),*]
                }
            }
            Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
