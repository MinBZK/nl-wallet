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

/// Derive `wallet_common::error_category::ErrorCategory` for Error enums.
///
/// Each variant can be categorized using the `category` attribute, which can have the following values:
///
/// - `expected`: This is an expected error and does not need to be reported.
/// - `critical`: This is a critical error that must be reported.
/// - `pd`: This is a critical error that must be reported, but the contents may contain privacy sensitive data.
/// - `defer`: Analysis of categorization is deferred to one of the fields of this variant.
///
/// A flat error hierarchy may look like this:
///
/// ```
/// # use std::io::{self, ErrorKind};
/// # use wallet_common::error_category::{Category, ErrorCategory};
/// # struct Attribute;
/// #[derive(wallet_common_macros::ErrorCategory)]
/// enum AttributeError {
///   #[category(pd)]
///   UnexpectedAttributes(Vec<Attribute>),
///   #[category(critical)]
///   IoError(io::Error),
///   #[category(expected)]
///   NotFound,
/// }
///
/// assert_eq!(AttributeError::UnexpectedAttributes(vec![]).category(), Category::PersonalData);
/// assert_eq!(AttributeError::IoError(io::Error::new(ErrorKind::PermissionDenied, "")).category(), Category::Critical);
/// assert_eq!(AttributeError::NotFound.category(), Category::Expected);
/// ```
///
/// For nested Error hierarchies, the `defer` category can be used to defer the decision lower in the hierarchy, for example:
///
/// ```
/// # use std::io::{self, ErrorKind};
/// # use wallet_common::error_category::{Category, ErrorCategory};
/// # struct Attribute;
/// # #[derive(wallet_common_macros::ErrorCategory)]
/// # enum AttributeError {
/// #   #[category(pd)]
/// #   UnexpectedAttributes(Vec<Attribute>),
/// #   #[category(critical)]
/// #   IoError(io::Error),
/// #   #[category(expected)]
/// #   NotFound(String)
/// # }
/// #[derive(wallet_common_macros::ErrorCategory)]
/// enum Error {
///   #[category(defer)]
///   Attribute(AttributeError),
/// }
/// ```
///
/// When the enum variant contains multiple fields, the `defer` attribute must be used to mark the field containing the nested error.
///
/// ```
/// # use std::io::{self, ErrorKind};
/// # use wallet_common::error_category::{Category, ErrorCategory};
/// # struct Attribute;
/// # #[derive(wallet_common_macros::ErrorCategory)]
/// # enum AttributeError {
/// #   #[category(pd)]
/// #   UnexpectedAttributes(Vec<Attribute>),
/// #   #[category(critical)]
/// #   IoError(io::Error),
/// #   #[category(expected)]
/// #   NotFound(String),
/// # }
/// #[derive(wallet_common_macros::ErrorCategory)]
/// enum Error {
///   #[category(defer)]
///   Attribute {
///     msg: String,
///     #[defer]
///     cause: AttributeError,
///    },
/// }
/// ```
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
            let (variants, errors): (Vec<_>, Vec<_>) =
                data.variants.iter().map(variant_category).partition(Result::is_ok);
            if errors.is_empty() {
                let variants = variants.into_iter().map(Result::unwrap);
                Ok(quote! { #(#variants)* })
            } else {
                // Combine multiple syn::Errors into a single syn::Error.
                // unwrap is safe here because of is_empty check above
                let error = errors
                    .into_iter()
                    .map(Result::unwrap_err)
                    .reduce(|mut acc, item| {
                        acc.combine(item);
                        acc
                    })
                    .unwrap();
                Err(error)
            }
        }
        Data::Struct(ref data) => Err(Error::new(
            data.struct_token.span(),
            "ErrorCategory can only be derived for enums.",
        )),
        Data::Union(ref data) => Err(Error::new(
            data.union_token.span(),
            "ErrorCategory can only be derived for enums.",
        )),
    }
}

/// Generate code for this `variant`.
fn variant_category(variant: &Variant) -> Result<TokenStream> {
    let category_attribute = find_list_attribute(&variant.attrs, CATEGORY);

    match category_attribute {
        Some(category) => {
            let variant_pattern = category_variant_pattern(variant, category)?;
            let variant_code = category_variant_code(category)?;
            Ok(quote! { #variant_pattern => #variant_code, })
        }
        None => Err(Error::new(
            variant.ident.span(),
            format!("Enum variant is missing `{}` attribute", CATEGORY),
        )),
    }
}

/// Find the [`MetaList`] attribute in `attrs` with the given `name`.
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

/// Generate the match pattern for `variant` based on the `category`.
fn category_variant_pattern(variant: &Variant, category: &MetaList) -> Result<TokenStream> {
    let cat = category.tokens.to_string();
    let result = match cat.as_str() {
        CRITICAL | EXPECTED | PD => variant_pattern(variant),
        DEFER => variant_pattern_defer(variant)?,
        _ => Err(Error::new(category.tokens.span(), invalid_category_error(&cat)))?,
    };

    Ok(result)
}

/// Generate code for the match arm based on the `category`.
fn category_variant_code(category: &MetaList) -> Result<TokenStream> {
    let cat = category.tokens.to_string();
    let result = match cat.as_str() {
        CRITICAL => quote! { wallet_common::error_category::Category::Critical },
        EXPECTED => quote! { wallet_common::error_category::Category::Expected },
        PD => quote! { wallet_common::error_category::Category::PersonalData },
        DEFER => quote! { wallet_common::error_category::ErrorCategory::category(defer) },
        _ => Err(Error::new(category.tokens.span(), invalid_category_error(&cat)))?,
    };

    Ok(result)
}

/// Construct error message for invalid category `cat`.
fn invalid_category_error(cat: &String) -> String {
    format!(
        "Expected any of {:?}, got {:?}.",
        vec![EXPECTED, CRITICAL, PD, DEFER],
        cat
    )
}

/// Generate a [`TokenStream`] that represents a match case.
/// This function supports unit, named and tuple structs with 0, 1, or multiple fields.
fn variant_pattern(variant: &Variant) -> TokenStream {
    let name = &variant.ident;
    match &variant.fields {
        Fields::Named(fields) => {
            if fields.named.is_empty() {
                quote! { Self::#name {} }
            } else {
                quote! { Self::#name { .. } }
            }
        }
        Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().map(|f| Ident::new("_", f.span()));
            quote! { Self::#name( #(#fields),* ) }
        }
        Fields::Unit => {
            quote! { Self::#name }
        }
    }
}

/// Generate a [`TokenStream`] that represents a match case for the defer case.
/// This function supports named and tuple structs with one or more fields.
fn variant_pattern_defer(variant: &Variant) -> Result<TokenStream> {
    let name = &variant.ident;
    let result = match &variant.fields {
        Fields::Named(fields) => {
            let (_index, defer_field) = find_defer_field(variant, &fields.named)?;
            let defer_field = defer_field.ident.clone();
            if fields.named.len() == 1 {
                quote! { Self::#name { #defer_field: defer } }
            } else {
                quote! { Self::#name { #defer_field: defer, .. } }
            }
        }
        Fields::Unnamed(fields) => {
            let (index, _defer_field) = find_defer_field(variant, &fields.unnamed)?;
            let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let pattern = if i == index { DEFER } else { "_" };
                Ident::new(pattern, f.span())
            });
            quote! { Self::#name( #(#fields),* ) }
        }
        Fields::Unit => Err(Error::new(
            variant.ident.span(),
            "#[category(defer)] is not supported on unit variants.",
        ))?,
    };
    Ok(result)
}

/// Find the [`Field`] together with its index in `fields` to defer into.
/// When there is only a single field, that field is selected.
/// When there are multiple fields, select the single field which is marked by the `#[defer]` attribute.
/// Returns an Error when no single field is found.
fn find_defer_field<'a>(variant: &'a Variant, fields: &'a Punctuated<Field, Comma>) -> Result<(usize, &'a Field)> {
    match fields.len() {
        0 => Err(Error::new(
            variant.ident.span(),
            "Expected a field to defer into, found none.",
        )),
        1 => Ok((0, &fields[0])),
        _ => {
            let deferred_fields: Vec<(usize, &Field)> = fields
                .iter()
                .enumerate()
                .filter(|(_index, field)| find_path_attribute(&field.attrs, DEFER).is_some())
                .collect();

            match deferred_fields.len() {
                0 => Err(Error::new(
                    variant.ident.span(),
                    "Expected #[defer] attribute to identify the field to defer into, found none.",
                )),
                1 => Ok(deferred_fields[0]),
                _ => Err(Error::new(
                    variant.ident.span(),
                    format!(
                        "Expected a single #[defer] attribute to identify the field to defer into, found {}.",
                        deferred_fields.len()
                    ),
                )),
            }
        }
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
