use itertools::Itertools;
use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::FnArg;
use syn::ItemFn;
use syn::Pat;
use syn::parse_macro_input;

/// Wraps an async function body with an `AuditLog::audit()` call.
///
/// Exactly one parameter must be annotated with `#[auditor]` to identify the
/// `AuditLog` implementor. Zero or more parameters may be annotated with
/// `#[audit]` to include them in the JSON parameters passed to the audit log.
///
/// The macro generates a struct named `AuditParameters` with a field for each
/// `#[audit]` parameter, derives `serde::Serialize`, and uses
/// `serde_json::to_value()` to produce the JSON.
///
/// # Example
///
/// ```rust
/// # use audit_log::model::AuditLog;
/// # use audit_log_macros::audited;
/// # struct DbErr;
/// # struct RevocationError;
/// # type WalletId = String;
/// # impl audit_log::model::FromAuditLogError for RevocationError {
/// #     fn from_audit_log_error(_e: Box<dyn std::error::Error + Send + Sync>) -> Self { Self }
/// # }
/// #[audited]
/// pub async fn revoke_wallet(
///     #[audit] wallet_id: &WalletId,
///     #[auditor] audit_log: &impl AuditLog,
/// ) -> Result<(), RevocationError> {
///     Ok(())
/// }
/// ```
///
/// This expands to the equivalent of:
///
/// ```rust
/// # use audit_log::model::AuditLog;
/// # struct DbErr;
/// # struct RevocationError;
/// # type WalletId = String;
/// # impl audit_log::model::FromAuditLogError for RevocationError {
/// #     fn from_audit_log_error(_e: Box<dyn std::error::Error + Send + Sync>) -> Self { Self }
/// # }
/// pub async fn revoke_wallet(wallet_id: &WalletId, audit_log: &impl AuditLog) -> Result<(), RevocationError> {
///     #[derive(::serde::Serialize)]
///     struct AuditParameters<'__audit> {
///         wallet_id: &'__audit str,
///     }
///     let __audit_params_json = {
///         let __audit_params = AuditParameters { wallet_id };
///         match ::serde_json::to_value(__audit_params) {
///             Ok(params) => params,
///             Err(error) => {
///                 return Err(audit_log::model::FromAuditLogError::from_audit_log_error(Box::new(
///                     error,
///                 )));
///             }
///         }
///     };
///     audit_log::model::AuditLog::audit(audit_log, "revoke_wallet", __audit_params_json, async move || Ok(())).await
/// }
/// ```
#[proc_macro_attribute]
pub fn audited(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    match audited_inner(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

const AUDITOR: &str = "auditor";
const AUDIT: &str = "audit";

#[derive(Debug, PartialEq)]
struct AuditParam {
    ident: syn::Ident,
    ty: Box<syn::Type>,
}

#[derive(Debug, PartialEq)]
enum ParamRole {
    Auditor(syn::Ident),
    AuditParam(AuditParam),
    Plain,
}

/// Parses and transforms the annotated function into an audit-wrapped version.
fn audited_inner(input: &ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // Macro can only be applied to async functions
    if input.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            input.sig.fn_token,
            "#[audited] can only be applied to async functions",
        ));
    }

    // Iterate through all parameters, and try to classify them
    let (successes, errors): (Vec<_>, Vec<_>) = input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(classify_param(pat_type).map(|role| (pat_type, role)))
            } else {
                None
            }
        })
        .partition_result();

    // If there are any errors, combine and return them.
    if let Some(error) = errors.into_iter().reduce(|mut acc, error| {
        acc.combine(error);
        acc
    }) {
        return Err(error);
    }

    // Iterate through all classified parameters, and collect #[auditor] and parameters to #[audit].
    let (auditor_ident, audit_params) = successes.into_iter().try_fold(
        (None, Vec::new()),
        |(mut auditor_ident, mut audit_params), (pat_type, role)| {
            match role {
                ParamRole::Auditor(ident) => {
                    if auditor_ident.is_some() {
                        return Err(syn::Error::new_spanned(
                            pat_type,
                            "only one parameter may be annotated with #[auditor], found multiple",
                        ));
                    }
                    auditor_ident = Some(ident);
                }
                ParamRole::AuditParam(param) => audit_params.push(param),
                ParamRole::Plain => {}
            }
            Ok((auditor_ident, audit_params))
        },
    )?;

    // Require that the #[auditor] parameter exists
    let audit_log_ident = auditor_ident.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.sig,
            "exactly one parameter must be annotated with #[auditor], found none",
        )
    })?;

    // Generate code
    let fn_name_str = input.sig.ident.to_string();
    let struct_name = format_ident!("AuditParameters");
    let lifetimes = lifetimes_from_iter(audit_params.iter().map(|p| p.ty.as_ref()))?;
    let (struct_def, struct_init) = generate_parameter_struct(&struct_name, &audit_params, &lifetimes);

    let vis = &input.vis;
    let sig = remove_parameter_attributes(input.sig.clone());
    let attrs = &input.attrs;
    let stmts = &input.block.stmts;

    let output = quote! {
        #(#attrs)*
        #vis #sig {
            #struct_def
            let __audit_params_json = {
                #struct_init
                match ::serde_json::to_value(__audit_params) {
                    Ok(params) => params,
                    Err(error) => {
                        return Err(audit_log::model::FromAuditLogError::from_audit_log_error(Box::new(error)));
                    }
                }
            };
            ::audit_log::model::AuditLog::audit(
                #audit_log_ident,
                #fn_name_str,
                __audit_params_json,
                async move || { #(#stmts)* },
            ).await
        }
    };

    Ok(output)
}

/// Strips `#[auditor]` and `#[audit]` attributes from function parameters.
fn remove_parameter_attributes(mut sig: syn::Signature) -> syn::Signature {
    sig.inputs
        .iter_mut()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_type) => Some(pat_type),
            _ => None,
        })
        .for_each(|pat_type| {
            pat_type
                .attrs
                .retain(|attr| !attr.path().is_ident(AUDITOR) && !attr.path().is_ident(AUDIT))
        });

    sig
}

/// Generates a `#[derive(Serialize)]` struct definition and initializer for the `#[audit]` parameters.
fn generate_parameter_struct(
    struct_name: &syn::Ident,
    audit_params: &[AuditParam],
    lifetimes: &[&syn::Lifetime],
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    // For each audit parameter, determine the struct field type and initializer.
    // If the parameter is already a reference (`&T`), use `&'__audit T` as the
    // field type and pass the parameter directly. For owned types, wrap with
    // `&'__audit T` and initialize with `&param`. This avoids generating
    // `&'__audit &T` which would contain an elided lifetime that is not allowed
    // in struct definitions.
    let field_defs: Vec<_> = audit_params.iter().map(generate_field_definition).collect();
    let field_inits: Vec<_> = audit_params.iter().map(generate_field_initialization).collect();

    let lifetimes = if !audit_params.is_empty() || !lifetimes.is_empty() {
        let lifetime = (!audit_params.is_empty()).then(|| quote! { '__audit });
        // collect all lifetimes
        let lifetimes = lifetime
            .into_iter()
            .chain(lifetimes.iter().unique().map(|lifetime| quote! { #lifetime }));
        // Wrap lifetimes with `<` and `>`
        Some(quote! { < #(#lifetimes,)* > })
    } else {
        None
    };

    let def = quote! {
        #[derive(::serde::Serialize)]
        struct #struct_name #lifetimes {
            #(#field_defs,)*
        }
    };
    let init = quote! {
        let __audit_params = #struct_name {
            #(#field_inits,)*
        };
    };
    (def, init)
}

/// Generates the struct field initializer expression for an `#[audit]` parameter.
fn generate_field_initialization(p: &AuditParam) -> proc_macro2::TokenStream {
    let ident = &p.ident;
    if matches!(p.ty.as_ref(), syn::Type::Reference(_)) {
        quote! { #ident }
    } else {
        quote! { #ident: &#ident }
    }
}

/// Generates the struct field type definition for an `#[audit]` parameter.
fn generate_field_definition(p: &AuditParam) -> proc_macro2::TokenStream {
    let ident = &p.ident;
    if let syn::Type::Reference(type_ref) = p.ty.as_ref() {
        let inner = &type_ref.elem;
        quote! { #ident: &'__audit #inner }
    } else {
        let ty = &p.ty;
        quote! { #ident: &'__audit #ty }
    }
}

/// Extracts the identifier from a typed parameter, returning an error if it is not a simple ident.
fn require_ident(pat_type: &syn::PatType, attr_name: &str) -> syn::Result<syn::Ident> {
    let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
        return Err(syn::Error::new_spanned(
            &pat_type.pat,
            format!("#[{attr_name}] parameter must be a simple identifier"),
        ));
    };
    Ok(pat_ident.ident.clone())
}

/// Classify the parameter as a [`ParamRole`].
fn classify_param(pat_type: &syn::PatType) -> syn::Result<ParamRole> {
    let (roles, errors): (Vec<_>, Vec<_>) = pat_type
        .attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident(AUDITOR) {
                require_ident(pat_type, AUDITOR).map(ParamRole::Auditor).into()
            } else if attr.path().is_ident(AUDIT) {
                require_ident(pat_type, AUDIT)
                    .map(|ident| {
                        ParamRole::AuditParam(AuditParam {
                            ident,
                            ty: pat_type.ty.clone(),
                        })
                    })
                    .into()
            } else {
                None
            }
        })
        .partition_result();

    // If there are any errors, combine and return them.
    if let Some(error) = errors.into_iter().reduce(|mut acc, error| {
        acc.combine(error);
        acc
    }) {
        return Err(error);
    }

    // Only a single attribute is allowed on a parameter.
    if roles.len() > 1 {
        return Err(syn::Error::new_spanned(
            pat_type,
            format!(
                "found multiple #[{AUDITOR}] and/or #[{AUDIT}] attributes on a single parameter, only a single is \
                 allowed"
            ),
        ));
    }

    Ok(roles.into_iter().next().unwrap_or(ParamRole::Plain))
}

/// Recursively collects all lifetimes from a [`syn::Type`].
fn lifetimes(syn_type: &syn::Type) -> syn::Result<Vec<&syn::Lifetime>> {
    match syn_type {
        syn::Type::Array(type_array) => lifetimes(&type_array.elem),
        syn::Type::Group(type_group) => lifetimes(&type_group.elem),
        syn::Type::ImplTrait(type_impl_trait) => Ok(type_impl_trait
            .bounds
            .iter()
            .flat_map(lifetime_from_bound)
            .collect_vec()),
        syn::Type::Paren(type_paren) => lifetimes(&type_paren.elem),
        syn::Type::Path(type_path) => lifetimes_from_path(type_path),
        syn::Type::Ptr(type_ptr) => lifetimes(&type_ptr.elem),
        syn::Type::Reference(type_reference) => lifetimes(&type_reference.elem),
        syn::Type::Slice(type_slice) => lifetimes(&type_slice.elem),
        syn::Type::TraitObject(type_trait_object) => Ok(type_trait_object
            .bounds
            .iter()
            .flat_map(lifetime_from_bound)
            .collect_vec()),
        syn::Type::Tuple(type_tuple) => lifetimes_from_iter(type_tuple.elems.iter()),
        _ => Err(syn::Error::new_spanned(syn_type, "type cannot be audited")),
    }
}

/// Collects all lifetimes from a [`syn::TypePath`], including generic arguments and qualified self types.
fn lifetimes_from_path(type_path: &syn::TypePath) -> Result<Vec<&syn::Lifetime>, syn::Error> {
    let direct_lifetimes = type_path.path.segments.iter().flat_map(|seg| match &seg.arguments {
        syn::PathArguments::AngleBracketed(args) => args
            .args
            .iter()
            .filter_map(|arg| match arg {
                syn::GenericArgument::Lifetime(lt) => Some(lt),
                _ => None,
            })
            .collect_vec(),
        _ => vec![],
    });

    let nested_types = type_path
        .qself
        .iter()
        .map(|q| q.ty.as_ref())
        .chain(type_path.path.segments.iter().flat_map(|seg| {
            match &seg.arguments {
                syn::PathArguments::AngleBracketed(args) => args
                    .args
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::GenericArgument::Type(ty) => Some(ty),
                        _ => None,
                    })
                    .collect_vec(),
                syn::PathArguments::Parenthesized(args) => args
                    .inputs
                    .iter()
                    .chain(match &args.output {
                        syn::ReturnType::Type(_, ty) => Some(ty.as_ref()),
                        syn::ReturnType::Default => None,
                    })
                    .collect_vec(),
                syn::PathArguments::None => vec![],
            }
        }));

    let nested_lifetimes = lifetimes_from_iter(nested_types)?;

    Ok(direct_lifetimes.chain(nested_lifetimes).collect())
}

/// Collects all lifetimes from an iterator of [`syn::Type`]s, combining any errors.
fn lifetimes_from_iter<'a>(iter: impl Iterator<Item = &'a syn::Type>) -> syn::Result<Vec<&'a syn::Lifetime>> {
    let (lifetimes, errors): (Vec<_>, Vec<syn::Error>) = iter.map(lifetimes).partition_result();

    // If there are any errors, combine and return them.
    if let Some(error) = errors.into_iter().reduce(|mut acc, error| {
        acc.combine(error);
        acc
    }) {
        return Err(error);
    }

    Ok(lifetimes.into_iter().flatten().collect())
}

/// Extracts a lifetime from a [`syn::TypeParamBound`], if it is a lifetime bound.
fn lifetime_from_bound(bound: &syn::TypeParamBound) -> Option<&syn::Lifetime> {
    match bound {
        syn::TypeParamBound::Lifetime(lt) => Some(lt),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn non_async_function() -> ItemFn {
        syn::parse_quote! {
            fn sync_fn(#[auditor] log: &Log) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn missing_auditor() -> ItemFn {
        syn::parse_quote! {
            async fn no_auditor(param: &str) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn duplicate_auditor() -> ItemFn {
        syn::parse_quote! {
            async fn two_auditors(
                #[auditor] log1: &Log,
                #[auditor] log2: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn non_ident_auditor() -> ItemFn {
        syn::parse_quote! {
            async fn destructured(
                #[auditor] (a, b): (Log, Log),
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }
    fn non_ident_audit() -> ItemFn {
        syn::parse_quote! {
            async fn destructured_audit(
                #[auditor] log: &Log,
                #[audit] (a, b): (String, String),
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn both_audit_and_auditor_on_same_param() -> ItemFn {
        syn::parse_quote! {
            async fn both_attrs(
                #[audit] #[auditor] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn double_audit_on_same_param() -> ItemFn {
        syn::parse_quote! {
            async fn both_attrs(
                #[audit] #[audit] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn double_auditor_on_same_param() -> ItemFn {
        syn::parse_quote! {
            async fn both_attrs(
                #[auditor] #[auditor] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn type_inference() -> ItemFn {
        syn::parse_quote! {
            async fn both_attrs(
                #[audit] param: _,
                #[auditor] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    #[rstest]
    #[case::non_async_function(non_async_function(), "#[audited] can only be applied to async functions")]
    #[case::missing_auditor(
        missing_auditor(),
        "exactly one parameter must be annotated with #[auditor], found none"
    )]
    #[case::duplicate_auditor(
        duplicate_auditor(),
        "only one parameter may be annotated with #[auditor], found multiple"
    )]
    #[case::non_ident_auditor(non_ident_auditor(), "#[auditor] parameter must be a simple identifier")]
    #[case::non_ident_audit(non_ident_audit(), "#[audit] parameter must be a simple identifier")]
    #[case::both_audit_and_auditor_on_same_param(
        both_audit_and_auditor_on_same_param(),
        "found multiple #[auditor] and/or #[audit] attributes on a single parameter, only a single is allowed"
    )]
    #[case::both_audit_and_auditor_on_same_param(
        double_audit_on_same_param(),
        "found multiple #[auditor] and/or #[audit] attributes on a single parameter, only a single is allowed"
    )]
    #[case::both_audit_and_auditor_on_same_param(
        double_auditor_on_same_param(),
        "found multiple #[auditor] and/or #[audit] attributes on a single parameter, only a single is allowed"
    )]
    #[case::type_inference(type_inference(), "type cannot be audited")]
    fn test_audited_inner_rejects(#[case] input: ItemFn, #[case] expected_error: &str) {
        let err = audited_inner(&input).unwrap_err();
        assert_eq!(err.to_string(), expected_error);
    }

    fn no_audit_params() -> ItemFn {
        syn::parse_quote! {
            async fn no_params(#[auditor] log: &Log, unused: String) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn with_audit_params() -> ItemFn {
        syn::parse_quote! {
            async fn with_params(
                #[audit] name: &str,
                #[auditor] log: &Log,
                unused: String,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn with_lifetime_in_audit_param() -> ItemFn {
        syn::parse_quote! {
            async fn with_lifetimes<'a>(
                #[audit] name: Cow<'a, str>,
                #[auditor] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn with_multiple_lifetimes_in_audit_params() -> ItemFn {
        syn::parse_quote! {
            async fn with_multi_lifetimes<'a, 'b>(
                #[audit] first: Cow<'a, str>,
                #[audit] second: Cow<'b, str>,
                #[auditor] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn with_nested_lifetimes_in_audit_param() -> ItemFn {
        syn::parse_quote! {
            async fn with_nested_lifetimes<'a, 'b>(
                #[audit] data: HashMap<Cow<'a, str>, Vec<Cow<'b, str>>>,
                #[auditor] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn with_reference_to_type_with_lifetimes() -> ItemFn {
        syn::parse_quote! {
            async fn with_ref_lifetimes<'a, 'b>(
                #[audit] data: &MyType<'a, 'b>,
                #[auditor] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    #[rstest]
    #[case::no_audit_params(no_audit_params())]
    #[case::with_audit_params(with_audit_params())]
    #[case::with_lifetime_in_audit_param(with_lifetime_in_audit_param())]
    #[case::with_multiple_lifetimes(with_multiple_lifetimes_in_audit_params())]
    #[case::with_nested_lifetimes(with_nested_lifetimes_in_audit_param())]
    #[case::with_reference_to_type_with_lifetimes(with_reference_to_type_with_lifetimes())]
    fn audited_inner_succeeds(#[case] input: ItemFn) {
        assert!(audited_inner(&input).is_ok());
    }

    fn assert_lifetime_strs(ty: &syn::Type, expected: &[&str]) {
        let result = lifetimes(ty).unwrap();
        let strs: Vec<_> = result.iter().map(|lt| lt.to_string()).collect();
        assert_eq!(strs, expected);
    }

    #[rstest]
    #[case::simple_path("String", &[])]
    #[case::path_with_one_lifetime("Cow<'a, str>", &["'a"])]
    #[case::path_with_multiple_lifetimes("MyType<'a, 'b, 'c>", &["'a", "'b", "'c"])]
    #[case::reference_to_simple_type("&str", &[])]
    #[case::reference_to_type_with_lifetimes("&MyType<'a, 'b>", &["'a", "'b"])]
    #[case::nested_generics("Vec<Cow<'a, str>>", &["'a"])]
    #[case::tuple("(Cow<'a, str>, Cow<'b, str>)", &["'a", "'b"])]
    #[case::array("[Cow<'a, str>; 3]", &["'a"])]
    #[case::slice("[Cow<'a, str>]", &["'a"])]
    #[case::deeply_nested("HashMap<Cow<'a, str>, Vec<Cow<'b, str>>>", &["'a", "'b"])]
    #[case::trait_object("dyn Trait + 'a", &["'a"])]
    #[case::impl_trait("impl Trait + 'a", &["'a"])]
    fn test_lifetimes_extraction(#[case] type_str: &str, #[case] expected: &[&str]) {
        let ty: syn::Type = syn::parse_str(type_str).unwrap();
        assert_lifetime_strs(&ty, expected);
    }

    #[rstest]
    #[case::inferred_type("_")]
    fn test_lifetimes_extraction_rejects(#[case] type_str: &str) {
        let ty: syn::Type = syn::parse_str(type_str).unwrap();
        assert!(lifetimes(&ty).is_err());
    }
}
