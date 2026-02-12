use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::FnArg;
use syn::ItemFn;
use syn::Pat;
use syn::parse_macro_input;

/// Wraps an async function body with an `AuditLog::audit()` call.
///
/// Exactly one parameter must be annotated with `#[auditer]` to identify the
/// `AuditLog` implementor. Zero or more parameters may be annotated with
/// `#[audit]` to include them in the JSON parameters passed to the audit log.
///
/// The macro generates a struct named `{FunctionName}AuditParameters` (in
/// CamelCase) with a field for each `#[audit]` parameter, derives
/// `serde::Serialize`, and uses `serde_json::to_value()` to produce the JSON.
///
/// # Example
///
/// ```ignore
/// #[audited]
/// pub async fn revoke_wallet(
///     #[audit] wallet_id: &str,
///     #[auditer] audit_log: &impl AuditLog<Error = PostgresAuditLogError>,
/// ) -> Result<(), RevocationError> {
///     Ok(())
/// }
/// ```
///
/// This expands to the equivalent of:
///
/// ```ignore
/// pub async fn revoke_wallet(
///     wallet_id: &str,
///     audit_log: &impl AuditLog<Error = PostgresAuditLogError>,
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

    match audited_inner(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

const AUDITER: &str = "auditer";
const AUDIT: &str = "audit";

/// Converts a `snake_case` string to `CamelCase`.
fn snake_to_camel_case(s: &str) -> String {
    s.split('_')
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

struct AuditParam {
    ident: syn::Ident,
    ty: Box<syn::Type>,
}

enum ParamRole {
    Auditer(syn::Ident),
    AuditParam(AuditParam),
    Plain,
}

/// Parses and transforms the annotated function into an audit-wrapped version.
fn audited_inner(mut input: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    // Macro can only be applied to async functions
    if input.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            input.sig.fn_token,
            "#[audited] can only be applied to async functions",
        ));
    }

    // Iterate through all parameters, and collect #[auditer] and parameters to #[audit].
    let mut auditer_ident = None;
    let mut audit_params = Vec::new();
    for arg in &mut input.sig.inputs {
        let FnArg::Typed(pat_type) = arg else {
            continue;
        };

        match classify_param(pat_type)? {
            ParamRole::Auditer(ident) => {
                if auditer_ident.is_some() {
                    return Err(syn::Error::new_spanned(
                        pat_type,
                        "only one parameter may be annotated with #[auditer]",
                    ));
                }
                auditer_ident = Some(ident);
            }
            ParamRole::AuditParam(param) => audit_params.push(param),
            ParamRole::Plain => {}
        }
    }

    // Require that the #[auditer] parameter exists
    let audit_log_ident = auditer_ident.ok_or_else(|| {
        syn::Error::new_spanned(&input.sig, "exactly one parameter must be annotated with #[auditer]")
    })?;

    // Generate code
    let fn_name_str = input.sig.ident.to_string();
    let struct_name = format_ident!("{}AuditParameters", snake_to_camel_case(&fn_name_str));
    let (struct_def, struct_init) = generate_parameter_struct(&struct_name, &audit_params);

    let vis = &input.vis;
    let sig = &input.sig;
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

/// Drains `#[auditer]` and `#[audit]` helper attributes from the parameter and
/// classifies it into a [`ParamRole`].
fn classify_param(pat_type: &mut syn::PatType) -> syn::Result<ParamRole> {
    let mut is_auditer = false;
    let mut is_audit = false;

    pat_type.attrs.retain(|attr| {
        if attr.path().is_ident(AUDITER) {
            is_auditer = true;
            false
        } else if attr.path().is_ident(AUDIT) {
            is_audit = true;
            false
        } else {
            true
        }
    });

    if is_auditer && is_audit {
        Err(syn::Error::new_spanned(
            &pat_type,
            format!("either #[{AUDITER}] or #[{AUDIT}] attribute allowed on a single parameter"),
        ))
    } else if is_auditer {
        let ident = require_ident(pat_type, AUDITER)?;
        Ok(ParamRole::Auditer(ident))
    } else if is_audit {
        let ident = require_ident(pat_type, AUDIT)?;
        Ok(ParamRole::AuditParam(AuditParam {
            ident,
            ty: pat_type.ty.clone(),
        }))
    } else {
        Ok(ParamRole::Plain)
    }
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
            fn sync_fn(#[auditer] log: &Log) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn missing_auditer() -> ItemFn {
        syn::parse_quote! {
            async fn no_auditer(param: &str) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn duplicate_auditer() -> ItemFn {
        syn::parse_quote! {
            async fn two_auditers(
                #[auditer] log1: &Log,
                #[auditer] log2: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn non_ident_auditer() -> ItemFn {
        syn::parse_quote! {
            async fn destructured(
                #[auditer] (a, b): (Log, Log),
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }
    fn non_ident_audit() -> ItemFn {
        syn::parse_quote! {
            async fn destructured_audit(
                #[auditer] log: &Log,
                #[audit] (a, b): (String, String),
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn both_audit_and_auditer_on_same_param() -> ItemFn {
        syn::parse_quote! {
            async fn both_attrs(
                #[audit] #[auditer] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    #[rstest]
    #[case::non_async_function(non_async_function(), "#[audited] can only be applied to async functions")]
    #[case::missing_auditer(missing_auditer(), "exactly one parameter must be annotated with #[auditer]")]
    #[case::duplicate_auditer(duplicate_auditer(), "only one parameter may be annotated with #[auditer]")]
    #[case::non_ident_auditer(non_ident_auditer(), "#[auditer] parameter must be a simple identifier")]
    #[case::non_ident_audit(non_ident_audit(), "#[audit] parameter must be a simple identifier")]
    #[case::both_audit_and_auditer_on_same_param(
        both_audit_and_auditer_on_same_param(),
        "either #[auditer] or #[audit] attribute allowed on a single parameter"
    )]
    fn test_audited_inner_rejects(#[case] input: ItemFn, #[case] expected_error: &str) {
        let err = audited_inner(input).unwrap_err();
        assert_eq!(err.to_string(), expected_error);
    }

    fn no_audit_params() -> ItemFn {
        syn::parse_quote! {
            async fn no_params(#[auditer] log: &Log) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    fn with_audit_params() -> ItemFn {
        syn::parse_quote! {
            async fn with_params(
                #[audit] name: &str,
                #[auditer] log: &Log,
            ) -> Result<(), Error> {
                Ok(())
            }
        }
    }

    #[rstest]
    #[case::no_audit_params(no_audit_params())]
    #[case::with_audit_params(with_audit_params())]
    fn audited_inner_succeeds(#[case] input: ItemFn) {
        assert!(audited_inner(input).is_ok());
    }
}
