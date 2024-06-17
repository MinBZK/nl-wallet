use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma, AttrStyle, Attribute, Data, DeriveInput,
    Error, Field, Fields, Ident, Meta, MetaList, Path, Result, Variant,
};

const CATEGORY: &str = "category";

const CRITICAL: &str = "critical";
const EXPECTED: &str = "expected";
const PD: &str = "pd";
const DEFER: &str = "defer";

#[proc_macro_derive(ErrorCategory, attributes(category, defer))]
pub fn error_category(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand(input).unwrap_or_else(Error::into_compile_error).into()
}

fn expand(input: DeriveInput) -> Result<TokenStream> {
    let name = input.ident;

    let variant_categories = variant_categories(&input.data)?;

    let expanded = quote! {
        #[automatically_derived]
        impl wallet_common::error_category::ErrorCategory for #name {
            fn category(&self) -> wallet_common::error_category::Category {
                match self {
                    #variant_categories
                }
            }
        }
    };

    Ok(expanded)
}

fn variant_categories(data: &Data) -> Result<TokenStream> {
    match *data {
        Data::Enum(ref data) => {
            let variants = data.variants.iter().map(variant_category).collect::<Result<Vec<_>>>()?;
            Ok(quote! {
                #(#variants)*
            })
        }
        Data::Struct(ref data) => Err(Error::new(
            data.struct_token.span(),
            "ErrorCategory can only be derived for Enums.",
        )),
        Data::Union(ref data) => Err(Error::new(
            data.union_token.span(),
            "ErrorCategory can only be derived for Enums.",
        )),
    }
}

fn variant_category(variant: &Variant) -> Result<TokenStream> {
    let category_attribute = find_list_attribute(&variant.attrs, CATEGORY);

    let result = if let Some(category) = category_attribute {
        let variant_pattern = category_variant_pattern(variant, category)?;
        let variant_code_block = category_code_block(category)?;
        quote! {
            #variant_pattern => #variant_code_block,
        }
    } else {
        return Err(Error::new(
            variant.ident.span(),
            format!("Enum variant is missing `{}` attribute", CATEGORY),
        ));
    };

    Ok(result)
}

fn find_list_attribute<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a MetaList> {
    attrs
        .iter()
        .filter(|a| matches!(a.style, AttrStyle::Outer))
        .flat_map(|a| {
            if let Meta::List(list) = &a.meta {
                list.into()
            } else {
                None
            }
        })
        .find(|a| path_equals(&a.path, name))
}

fn category_variant_pattern(variant: &Variant, category: &MetaList) -> Result<TokenStream> {
    let result = match category.tokens.to_string().as_str() {
        CRITICAL | EXPECTED | PD => variant_pattern(variant),
        DEFER => variant_pattern_defer(variant)?,
        _ => {
            return Err(Error::new(
                category.tokens.span(),
                format!("Expected any of {:?}.", vec![EXPECTED, CRITICAL, PD, DEFER]),
            ))
        }
    };

    Ok(result)
}

fn category_code_block(category: &MetaList) -> Result<TokenStream> {
    let result = match category.tokens.to_string().as_str() {
        CRITICAL => quote! { wallet_common::error_category::Category::Critical },
        EXPECTED => quote! { wallet_common::error_category::Category::Expected },
        PD => quote! { wallet_common::error_category::Category::PersonalData },
        DEFER => quote! { wallet_common::error_category::ErrorCategory::category(defer) },
        _ => {
            return Err(Error::new(
                category.tokens.span(),
                format!("Expected any of {:?}.", vec![EXPECTED, CRITICAL, PD, DEFER]),
            ))
        }
    };

    Ok(result)
}

/// Generate a [`TokenStream`] that represents a match case.
///
/// ```ignore
/// Variant {}
/// ```
///
/// ```ignore
/// Variant { .. }
/// ```
///
/// ```ignore
/// Variant()
/// ```
///
/// ```ignore
/// Variant(_, _, ...)
/// ```
///
/// ```ignore
/// Variant
/// ```
fn variant_pattern(variant: &Variant) -> TokenStream {
    let name = &variant.ident;
    match &variant.fields {
        Fields::Named(fields) => {
            if fields.named.is_empty() {
                quote! {
                    Self::#name {}
                }
            } else {
                quote! {
                    Self::#name { .. }
                }
            }
        }
        Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().map(|f| Ident::new("_", f.span()));
            quote! {
                Self::#name( #(#fields),* )
            }
        }
        Fields::Unit => {
            quote! {
                Self::#name
            }
        }
    }
}

/// Generate a [`TokenStream`] that represents a match case for the defer case.
///
/// ```ignore
/// Variant { field: defer }
/// ```
///
/// ```ignore
/// Variant { field: defer, .. }
/// ```
///
/// ```ignore
/// Variant(defer)
/// ```
///
/// ```ignore
/// Variant(_, defer, _)
/// ```
fn variant_pattern_defer(variant: &Variant) -> Result<TokenStream> {
    let name = &variant.ident;
    let result = match &variant.fields {
        Fields::Named(fields) => {
            // Verify there is a single field with the defer attribute
            let defer_field = find_defer_field(variant, &fields.named)?.ident.clone();
            if fields.named.len() == 1 {
                quote! {
                    Self::#name { #defer_field: defer }
                }
            } else {
                quote! {
                    Self::#name { #defer_field: defer, .. }
                }
            }
        }
        Fields::Unnamed(fields) => {
            // Verify there is a single field with the defer attribute
            let _defer_field = find_defer_field(variant, &fields.unnamed)?;
            let fields = fields.unnamed.iter().map(|f| {
                let pattern = if find_path_attribute(&f.attrs, DEFER).is_some() {
                    DEFER
                } else {
                    "_"
                };
                Ident::new(pattern, f.span())
            });
            quote! {
                Self::#name( #(#fields),* )
            }
        }
        Fields::Unit => Err(Error::new(
            variant.ident.span(),
            "#[category(defer)] is not supported on unit variants.",
        ))?,
    };
    Ok(result)
}

/// Find the [`Field`] in `fields` that is annotated with the `defer` attribute.
/// Returns an Error when none or multiple fields are found.
fn find_defer_field<'a>(variant: &'a Variant, fields: &'a Punctuated<Field, Comma>) -> Result<&'a Field> {
    let deferred_fields: Vec<&Field> = fields
        .iter()
        .filter(|field| find_path_attribute(&field.attrs, DEFER).is_some())
        .collect();

    if fields.is_empty() {
        Err(Error::new(
            variant.ident.span(),
            "Expected a field to defer into, found none.",
        ))
    } else if deferred_fields.is_empty() {
        Err(Error::new(
            variant.ident.span(),
            "Expected #[defer] attribute to identify the field to defer into, found none.",
        ))
    } else if deferred_fields.len() > 1 {
        Err(Error::new(
            variant.ident.span(),
            format!(
                "Expected a single #[defer] attribute to identify the field to defer into, found {}.",
                deferred_fields.len()
            ),
        ))
    } else {
        let defer_field = deferred_fields.first().unwrap();
        Ok(defer_field)
    }
}

/// Find the [`Path`] attribute with the given `name`, if any.
fn find_path_attribute<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a Path> {
    attrs
        .iter()
        .flat_map(|a| {
            if let Meta::Path(path) = &a.meta {
                path.into()
            } else {
                None
            }
        })
        .find(|path| path_equals(path, name))
}

/// Check whether `path`'s identifier is equal to the `expected` string.
fn path_equals(path: &Path, expected: &str) -> bool {
    path.get_ident()
        .map(|ident| ident.to_string().eq(expected))
        .unwrap_or(false)
}
