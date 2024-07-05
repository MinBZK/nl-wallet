use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, token::Comma, AttrStyle, Attribute, Block, Data,
    DataEnum, DataStruct, DeriveInput, Error, Field, Fields, Ident, ImplItem, ImplItemFn, Item, ItemFn, ItemImpl, Meta,
    MetaList, Path, Result, Signature, Variant, Visibility,
};

const CATEGORY: &str = "category";

const CRITICAL: &str = "critical";
const EXPECTED: &str = "expected";
const PD: &str = "pd";
const DEFER: &str = "defer";

/// Attribute macro to classify errors and report to Sentry.
/// This macro can be set on `fn` and `impl` blocks.
/// Setting this macro on an `impl` block is the same as setting this on all `fn`s in the impl block.
#[proc_macro_attribute]
pub fn sentry_capture_error(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse::<Item>(item);
    match item {
        Ok(Item::Fn(item_fn)) => handle_error_category_fn(item_fn).into(),
        Ok(Item::Impl(item_impl)) => handle_error_category_impl_block(item_impl).into(),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
        Ok(item) => {
            let mut token_stream = Error::new(
                item.span(),
                "attribute macro `handle_error_category` not supported here",
            )
            .into_compile_error();
            // Copy the original item, to prevent new/other compilation errors
            item.to_tokens(&mut token_stream);
            token_stream.into()
        }
    }
}

/// Generate code for a `fn`.
fn handle_error_category_fn(ItemFn { attrs, vis, sig, block }: ItemFn) -> TokenStream {
    let defaultness = None;
    handle_error_category_function(&attrs, &vis, &defaultness, &sig, &block)
}

/// Generate code for functions, can be used both for regular functions and associated functions.
fn handle_error_category_function(
    attrs: &[Attribute],
    vis: &Visibility,
    defaultness: &Option<syn::token::Default>,
    sig: &Signature,
    block: &Block,
) -> TokenStream {
    let stmts = &block.stmts;

    quote! {
        #(#attrs)* #vis #defaultness #sig {
            Result::map_err(
                {
                    #(#stmts)*
                }
                , ::wallet_common::sentry::classify_and_report_to_sentry)
        }
    }
}

/// Generate code for an `impl` block.
fn handle_error_category_impl_block(
    ItemImpl {
        attrs,
        defaultness,
        unsafety,
        impl_token,
        generics,
        trait_,
        self_ty,
        brace_token: _brace_token,
        items,
    }: ItemImpl,
) -> TokenStream {
    // Handle all functions
    let items = items
        .into_iter()
        .map(|item| match item {
            ImplItem::Fn(item_fn) => handle_error_category_impl_fn(item_fn),
            item => item.into_token_stream(),
        })
        .collect::<Vec<_>>();
    match trait_ {
        Some((not_, path_, for_)) => quote! {
            #(#attrs)*
            #defaultness #unsafety #impl_token #generics #not_ #path_ #for_ #self_ty {
                #(#items)*
            }
        },
        None => quote! {
            #(#attrs)*
            #defaultness #unsafety #impl_token #generics #self_ty {
                #(#items)*
            }
        },
    }
}

/// Generate code for an associated `fn`.
fn handle_error_category_impl_fn(
    ImplItemFn {
        attrs,
        vis,
        defaultness,
        sig,
        block,
    }: ImplItemFn,
) -> TokenStream {
    handle_error_category_function(&attrs, &vis, &defaultness, &sig, &block)
}

/// Derive `wallet_common::error_category::ErrorCategory` for Error types.
///
/// Errors can be classified using the `category` attribute, which can have the following values:
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
/// #[derive(ErrorCategory)]
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
/// # use std::io;
/// # use wallet_common::error_category::{Category, ErrorCategory};
/// # struct Attribute;
/// # #[derive(ErrorCategory)]
/// # enum AttributeError {
/// #   #[category(pd)]
/// #   UnexpectedAttributes(Vec<Attribute>),
/// #   #[category(critical)]
/// #   IoError(io::Error),
/// #   #[category(expected)]
/// #   NotFound(String)
/// # }
/// #[derive(ErrorCategory)]
/// enum Error {
///   #[category(defer)]
///   Attribute(AttributeError),
/// }
/// ```
///
/// When an enum variant that uses `#[category(defer)]` contains multiple fields, the `defer` attribute must be used
/// to mark the field containing the nested error.
///
/// ```
/// # use std::io;
/// # use wallet_common::error_category::{Category, ErrorCategory};
/// # struct Attribute;
/// # #[derive(ErrorCategory)]
/// # enum AttributeError {
/// #   #[category(pd)]
/// #   UnexpectedAttributes(Vec<Attribute>),
/// #   #[category(critical)]
/// #   IoError(io::Error),
/// #   #[category(expected)]
/// #   NotFound(String),
/// # }
/// #[derive(ErrorCategory)]
/// enum Error {
///   #[category(defer)]
///   Attribute {
///     msg: String,
///     #[defer]
///     cause: AttributeError,
///    },
/// }
/// ```
///
/// `ErrorCategory` can also be derived for structs, as shown in the following example:
///
/// ```
/// # use std::io;
/// # use wallet_common::error_category::{Category, ErrorCategory};
/// # struct Attribute;
/// # #[derive(ErrorCategory)]
/// # enum AttributeError {
/// #   #[category(pd)]
/// #   UnexpectedAttributes(Vec<Attribute>),
/// #   #[category(critical)]
/// #   IoError(io::Error),
/// #   #[category(expected)]
/// #   NotFound(String),
/// # }
/// #[derive(ErrorCategory)]
/// #[category(defer)]
/// struct Error {
///   msg: String,
///   #[defer]
///   cause: AttributeError,
/// }
/// ```
#[proc_macro_derive(ErrorCategory, attributes(category, defer))]
pub fn error_category(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand(input).unwrap_or_else(Error::into_compile_error).into()
}

fn expand(input: DeriveInput) -> Result<TokenStream> {
    let body = match input.data {
        Data::Enum(ref data) => expand_enum(data),
        Data::Struct(ref data) => expand_struct(&input, data),
        Data::Union(ref data) => Err(Error::new(
            data.union_token.span(),
            "`ErrorCategory` can not be derived for unions",
        )),
    }?;

    let name = input.ident;

    let expanded = quote! {
        #[automatically_derived]
        impl ::wallet_common::error_category::ErrorCategory for #name {
            fn category(&self) -> ::wallet_common::error_category::Category {
                #body
            }
        }
    };

    Ok(expanded)
}

/// Generate code for the implementation of `ErrorCategory` for the given `enum_data`.
fn expand_enum(enum_data: &DataEnum) -> Result<TokenStream> {
    let (variants, errors): (Vec<_>, Vec<_>) = enum_data
        .variants
        .iter()
        .map(enum_variant_category)
        .partition(Result::is_ok);
    if errors.is_empty() {
        let variants = variants.into_iter().map(Result::unwrap);
        Ok(quote! {
            match self {
                #(#variants),*
            }
        })
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

/// Generate code for the implementation of `ErrorCategory` for the given `struct_data`.
fn expand_struct(input: &DeriveInput, struct_data: &DataStruct) -> Result<TokenStream> {
    let name = &input.ident;
    let category = find_list_attribute(&input.attrs, CATEGORY).ok_or(Error::new(
        input.span(),
        format!("expected `{}` attribute on struct `{}`", CATEGORY, name),
    ))?;

    let category_code = category_code(category)?;
    let cat = category.tokens.to_string();
    match cat.as_str() {
        CRITICAL | EXPECTED | PD => Ok(quote! { #category_code }),
        DEFER => {
            let category_defer_pattern = category_defer_pattern(name.span(), &struct_data.fields)?;
            Ok(quote! {
                let #name #category_defer_pattern = self;
                #category_code
            })
        }
        _ => Err(Error::new(category.tokens.span(), invalid_category_error(&cat)))?,
    }
}

/// Generate code for this enum  `variant`.
fn enum_variant_category(variant: &Variant) -> Result<TokenStream> {
    let category = find_list_attribute(&variant.attrs, CATEGORY).ok_or(Error::new(
        variant.ident.span(),
        format!("enum variant is missing `{}` attribute", CATEGORY),
    ))?;

    let variant_pattern = enum_variant_category_pattern(variant, category)?;
    let variant_code = category_code(category)?;
    Ok(quote! { #variant_pattern => #variant_code })
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

/// Generate a [`TokenStream`] match pattern for the enum `variant` based on the `category`.
fn enum_variant_category_pattern(variant: &Variant, category: &MetaList) -> Result<TokenStream> {
    let cat = category.tokens.to_string();
    let pattern = match cat.as_str() {
        CRITICAL | EXPECTED | PD => variant_pattern(&variant.fields),
        DEFER => category_defer_pattern(variant.ident.span(), &variant.fields)?,
        _ => Err(Error::new(category.tokens.span(), invalid_category_error(&cat)))?,
    };

    let name = &variant.ident;
    Ok(quote! { Self::#name #pattern })
}

/// Generate a [`TokenStream`] that represents a pattern match for a struct with the given `fields`, that ignores the fields.
/// This function supports unit, named and tuple structs with 0, 1, or multiple fields.
/// It returns the pattern without the struct or enum variant name, e.g.: `(_, _)`, `{ .. }`, `()`, `{}`,
fn variant_pattern(fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            if fields.named.is_empty() {
                quote! { {} }
            } else {
                quote! { { .. } }
            }
        }
        Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().map(|f| Ident::new("_", f.span()));
            quote! { ( #(#fields),* ) }
        }
        Fields::Unit => {
            quote! {}
        }
    }
}

/// Generate a [`TokenStream`] that represents a pattern match for a struct with the given `fields`, extracting the defer field.
/// This function supports named and tuple structs with one or more fields.
/// It returns the pattern without the struct or enum variant name, e.g. `(defer, _)`, `{ field_1: defer, .. }`
fn category_defer_pattern(span: Span, fields: &Fields) -> Result<TokenStream> {
    let result = match fields {
        Fields::Named(fields) => {
            let (_index, defer_field) = find_defer_field(span, &fields.named)?;
            let defer_field = defer_field.ident.clone();
            if fields.named.len() == 1 {
                quote! { { #defer_field: defer } }
            } else {
                quote! { { #defer_field: defer, .. } }
            }
        }
        Fields::Unnamed(fields) => {
            let (index, _defer_field) = find_defer_field(span, &fields.unnamed)?;
            let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let pattern = if i == index { DEFER } else { "_" };
                Ident::new(pattern, f.span())
            });
            quote! { ( #(#fields),* ) }
        }
        Fields::Unit => Err(Error::new(
            span,
            "`#[category(defer)]` is not supported on unit variants",
        ))?,
    };
    Ok(result)
}

/// Generate an expression for the given `category`.
fn category_code(category: &MetaList) -> Result<TokenStream> {
    let cat = category.tokens.to_string();
    let result = match cat.as_str() {
        CRITICAL => quote! { ::wallet_common::error_category::Category::Critical },
        EXPECTED => quote! { ::wallet_common::error_category::Category::Expected },
        PD => quote! { ::wallet_common::error_category::Category::PersonalData },
        DEFER => quote! { ::wallet_common::error_category::ErrorCategory::category(defer) },
        _ => Err(Error::new(category.tokens.span(), invalid_category_error(&cat)))?,
    };

    Ok(result)
}

/// Construct error message for invalid category `cat`.
fn invalid_category_error(cat: &String) -> String {
    format!(
        "expected any of {:?}, got {:?}",
        vec![EXPECTED, CRITICAL, PD, DEFER],
        cat
    )
}

/// Find the [`Field`] together with its index in `fields` to defer into.
/// When there is only a single field, that field is selected.
/// When there are multiple fields, select the single field which is marked by the `#[defer]` attribute.
/// Returns an Error when no single field is found.
fn find_defer_field(span: Span, fields: &Punctuated<Field, Comma>) -> Result<(usize, &Field)> {
    match fields.len() {
        0 => Err(Error::new(span, "expected a field to defer into, found none")),
        1 => Ok((0, &fields[0])),
        _ => {
            let deferred_fields: Vec<(usize, &Field)> = fields
                .iter()
                .enumerate()
                .filter(|(_index, field)| find_path_attribute(&field.attrs, DEFER).is_some())
                .collect();

            match deferred_fields.len() {
                0 => Err(Error::new(
                    span,
                    "expected `#[defer]` attribute to identify the field to defer into, found none",
                )),
                1 => Ok(deferred_fields[0]),
                _ => Err(Error::new(
                    span,
                    format!(
                        "expected a single `#[defer]` attribute to identify the field to defer into, found {}",
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
