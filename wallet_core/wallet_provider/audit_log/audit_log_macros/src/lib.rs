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
/// The macro generates a struct named `{FunctionName}AuditParameters` (in
/// CamelCase) with a field for each `#[audit]` parameter, derives
/// `serde::Serialize`, and uses `serde_json::to_value()` to produce the JSON.
///
/// # Example
///
/// ```rust
/// # use audit_log::model::AuditLog;
/// # use audit_log_macros::audited;
/// # struct DbErr;
/// # struct RevocationError;
/// # impl audit_log::model::FromAuditLogError for RevocationError { fn from_audit_log_error(_e: Box<dyn std::error::Error + Send + Sync>) -> Self { Self } }
/// #[audited]
/// pub async fn revoke_wallet(
///     #[audit] wallet_id: &str,
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
/// # impl audit_log::model::FromAuditLogError for RevocationError { fn from_audit_log_error(_e: Box<dyn std::error::Error + Send + Sync>) -> Self { Self } }
/// pub async fn revoke_wallet(
///     wallet_id: &str,
///     audit_log: &impl AuditLog,
/// ) -> Result<(), RevocationError> {
///     #[derive(::serde::Serialize)]
///     struct RevokeWalletAuditParameters<'__audit> {
///         wallet_id: &'__audit str,
///     }
///     let __audit_params_json = {
///         let __audit_params = RevokeWalletAuditParameters { wallet_id: wallet_id };
///         ::serde_json::to_value(__audit_params)
///             .expect("audit parameters should serialize to JSON")
///     };
///     audit_log::model::AuditLog::audit(
///         audit_log,
///         "revoke_wallet",
///         __audit_params_json,
///         async move || { Ok(()) },
///     ).await
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

/// Converts a `snake_case` string to `CamelCase`.
fn snake_to_camel_case(s: &str) -> String {
    s.split('_')
        .filter_map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        })
        .collect()
}

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
    let struct_name = format_ident!("{}AuditParameters", snake_to_camel_case(&fn_name_str));
    let (struct_def, struct_init) = generate_parameter_struct(&struct_name, &audit_params);

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
                ::serde_json::to_value(__audit_params)
                    .expect("audit parameters should serialize to JSON")
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

// Strip #[auditor] and #[audit] attributes from function parameters
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
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    // For each audit parameter, determine the struct field type and initializer.
    // If the parameter is already a reference (`&T`), use `&'__audit T` as the
    // field type and pass the parameter directly. For owned types, wrap with
    // `&'__audit T` and initialize with `&param`. This avoids generating
    // `&'__audit &T` which would contain an elided lifetime that is not allowed
    // in struct definitions.
    let field_defs: Vec<_> = audit_params.iter().map(generate_field_definition).collect();
    let field_inits: Vec<_> = audit_params.iter().map(generate_field_initialization).collect();

    // Only include the lifetime parameter when there are fields that use it.
    let lifetime = (!audit_params.is_empty()).then(|| quote! { <'__audit> });

    let def = quote! {
        #[derive(::serde::Serialize)]
        struct #struct_name #lifetime {
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

/// Drains `#[auditor]` and `#[audit]` helper attributes from the parameter and
/// classifies it into a [`ParamRole`].
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::converts_simple_name("hello_world", "HelloWorld")]
    #[case::converts_single_word("hello", "Hello")]
    #[case::handles_empty_string("", "")]
    #[case::handles_leading_underscores("__leading", "Leading")]
    #[case::handles_consecutive_underscores("a__b", "AB")]
    fn snake_to_camel_case_handles(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(snake_to_camel_case(input), expected);
    }

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

    #[rstest]
    #[case::no_audit_params(no_audit_params())]
    #[case::with_audit_params(with_audit_params())]
    fn audited_inner_succeeds(#[case] input: ItemFn) {
        assert!(audited_inner(&input).is_ok());
    }
}
