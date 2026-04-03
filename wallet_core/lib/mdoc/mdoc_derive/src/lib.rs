use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Expr;
use syn::ExprLit;
use syn::Field;
use syn::Fields;
use syn::Lit;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;

/// Derive macro for [`CborIndexedFields`].
///
/// # Example
///
/// ```ignore
/// #[derive(CborIndexedFields)]
/// pub struct BleOptions {
///     pub peripheral_server_mode: bool,         // index 0
///     pub central_client_mode: bool,            // index 1
///     #[cbor_index = 10]
///     pub peripheral_server_uuid: Option<...>,  // index 10
///     pub central_client_uuid: Option<...>,     // index 11
///     #[cbor_index = 20]
///     pub peripheral_server_address: Option<...>, // index 20
/// }
/// ```
#[proc_macro_derive(CborIndexedFields, attributes(cbor_index))]
pub fn derive_cbor_indexed_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_inner(input).unwrap_or_else(|e| e.into_compile_error().into())
}

fn named_fields(input: &DeriveInput) -> syn::Result<&Punctuated<Field, Comma>> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(&fields.named),
            _ => Err(syn::Error::new(
                Span::call_site(),
                "`CborIndexedFields` only supports structs with named fields",
            )),
        },
        _ => Err(syn::Error::new(
            Span::call_site(),
            "`CborIndexedFields` only supports structs",
        )),
    }
}

fn field_name_index_pairs(fields: &Punctuated<Field, Comma>) -> syn::Result<Vec<(String, u64)>> {
    let mut current_index: u64 = 0;

    fields
        .iter()
        .map(|field| -> syn::Result<(String, u64)> {
            let field_name = field.ident.as_ref().unwrap().to_string();

            let cbor_index: Option<u64> = field
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("cbor_index"))
                .map(|attr| -> syn::Result<u64> {
                    let nv = attr.meta.require_name_value()?;
                    let Expr::Lit(ExprLit {
                        lit: Lit::Int(lit_int), ..
                    }) = &nv.value
                    else {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "`cbor_index` value must be a non-negative integer",
                        ));
                    };

                    lit_int
                        .base10_parse::<u64>()
                        .map_err(|e| syn::Error::new(lit_int.span(), e))
                })
                .transpose()?;

            current_index = match cbor_index {
                Some(idx) if idx >= current_index => idx,
                Some(_) => {
                    return Err(syn::Error::new(
                        field.span(),
                        "`cbor_index` value must be larger than the current field counter",
                    ));
                }
                None => current_index,
            };

            let to_return = (field_name, current_index);
            current_index += 1;

            Ok(to_return)
        })
        .collect()
}

fn derive_inner(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = named_fields(&input)?;
    let (field_names, field_indices): (Vec<_>, Vec<_>) = field_name_index_pairs(fields)?.into_iter().unzip();

    let name = input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics CborIndexedFields for #name #type_generics #where_clause {
            fn field_indices() -> &'static [(&'static str, u64)] {
                &[ #( (#field_names, #field_indices), )* ]
            }
        }
    }
    .into())
}
