use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Arm, Attribute, Data, DataStruct, DeriveInput, ExprAssign, ExprMatch, Field, Ident, LitStr, Token, Type, parse2, punctuated::Punctuated, spanned::Spanned
};

use crate::proc::{Error, Result, utils::extract_attribute_list};

struct FieldConfig {
    ident: Ident,
    typ: Type,
    flag: bool,
    default: Option<Option<TokenStream>>,
    names: Vec<LitStr>,
}

struct FromArgsConfig {
    start_match: Vec<TokenStream>,
    end_match: Vec<TokenStream>,
}

/// Implementation of the derive proc macro for [`crate::FromArgs`]
pub fn derive_from_args(item: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse2(item)?;

    let ident = input.ident;

    if !input.generics.params.is_empty() {
        return Error::msg_span(
            input.generics.span(),
            "FromArgs doens't support generics.",
        )
        .err();
    }

    let cfg = FromArgsConfig::parse(input.attrs)?;

    match input.data {
        Data::Struct(sd) => derive_from_args_struct(ident, cfg, sd),
        _ => Error::msg_span(
            ident.span(),
            "FromArgs derive macro supports only structs.",
        )
        .err(),
    }
}

fn derive_from_args_struct(
    ident: Ident,
    cfg: FromArgsConfig,
    data: DataStruct,
) -> Result<TokenStream> {
    let fields = data
        .fields
        .into_iter()
        .map(FieldConfig::parse)
        .collect::<Result<Vec<_>>>()?;

    let mut res = TokenStream::new();

    for field in &fields {
        let id = &field.ident;
        let ty = &field.typ;
        res.extend(quote! {
            let mut #id: Option<#ty> = None;
        });
    }

    let mut branches = TokenStream::new();
    branches.extend(cfg.start_match);
    
    for field in &fields {
        if field.names.is_empty() {
            continue;
        }

        let id = &field.ident;
        let pat: Punctuated<_, Token![|]> = field.names.iter().collect();

        if field.flag {
            branches.extend(quote! {
                #pat => #id = Some(true),
            });
        } else {
            branches.extend(quote! {
                #pat => #id = Some(args.next_arg()?),
            });
        }
    }
    
    branches.extend(cfg.end_match);
    branches.extend(quote! {
        _ => return args.err_unknown_argument().err(),
    });

    res.extend(quote! {
        while let Some(arg) = args.next() {
            match arg {
                #branches
            }
        }
    });

    for field in &fields {
        let id = field.ident.clone();
        let expr = match &field.default {
            Some(Some(v)) => quote! {
                let #id = #id.unwrap_or_else(|| #v);
            },
            Some(None) => quote! {
                let #id = #id.unwrap_or_default();
            },
            None if let Some(n) = field.names.first() => {
                let msg = format!("Missing required argument `{}`", n.value());
                let msg = LitStr::new(&msg, Span::mixed_site());
                quote! {
                    let Some(#id) = #id else {
                        return args.err_no_more_arguments().hint(#msg).err();
                    };
                }
            }
            None => quote! {
                let #id = #id.unwrap_or_default();
            },
        };

        res.extend(expr);
    }

    let names: Punctuated<_, Token![,]> =
        fields.iter().map(|a| &a.ident).collect();
    res.extend(quote! { Ok(Self { #names }) });

    Ok(quote! {
        impl<'a, S: AsRef<str>> pareg::FromArgs<'a, S> for #ident {
            fn parse_args(args: &mut pareg::ParegRef<'a, S>) -> pareg::Result<Self> {
                #res
            }
        }
    })
}

impl FieldConfig {
    pub fn parse(field: Field) -> Result<Self> {
        let span = field.span();
        let ident = field.ident.ok_or_else(|| {
            Error::msg_span(
                span,
                "Unnamed fields are not supported by FromArgs derive macro.",
            )
        })?;
        let typ = field.ty;
        let mut names = vec![];
        let mut default = None;
        let mut flag = false;

        for attr in field.attrs {
            if !attr.path().is_ident("from_args") {
                continue;
            }
            let vars = extract_attribute_list(attr)?;
            for v in vars {
                if let Ok(n) = parse2::<LitStr>(v.clone()) {
                    names.push(n);
                } else if let Ok(n) = parse2::<Ident>(v.clone()) {
                    match n.to_string().as_str() {
                        "default" => default = Some(None),
                        "flag" => flag = true,
                        _ => {
                            return Error::msg_span(
                                n.span(),
                                "Unknown option for from_args.",
                            )
                            .err();
                        }
                    }
                } else if let Ok(a) = parse2::<ExprAssign>(v.clone()) {
                    let id: Ident = parse2(a.left.into_token_stream())?;
                    match id.to_string().as_str() {
                        "default" => {
                            default = Some(Some(a.right.into_token_stream()))
                        }
                        _ => {
                            return Error::msg_span(
                                id.span(),
                                "Unknown option for from_args.",
                            )
                            .err();
                        }
                    }
                } else {
                    return Error::msg_span(
                        v.span(),
                        "Unknown option for FromArg.",
                    )
                    .err();
                }
            }
        }

        Ok(Self {
            ident,
            typ,
            flag,
            names,
            default,
        })
    }
}

impl FromArgsConfig {
    pub fn parse(attrs: Vec<Attribute>) -> Result<Self> {
        let mut start_match = vec![];
        let mut end_match = vec![];

        for attr in attrs {
            if !attr.path().is_ident("from_args") {
                continue;
            }
            let vars = extract_attribute_list(attr)?;
            for v in vars {
                if let Ok(a) = parse2::<ExprMatch>(v.clone()) {
                    let id = parse2::<Ident>(a.expr.into_token_stream())?;
                    match id.to_string().as_str() {
                        "start" => {
                            start_match.extend(a.arms.into_iter().map(arm_to_token_stream));
                        }
                        "end" => {
                            end_match.extend(a.arms.into_iter().map(arm_to_token_stream));
                        }
                        _ => {
                            return Error::msg_span(
                                id.span(),
                                "Unknown option for from_args.",
                            )
                            .err();
                        }
                    }
                } else {
                    return Error::msg_span(
                        v.span(),
                        "Unknown option for FromArg.",
                    )
                    .err();
                }
            }
        }

        Ok(Self {
            start_match,
            end_match,
        })
    }
}

fn arm_to_token_stream(arm: Arm) -> TokenStream {
    let pat = arm.pat;
    let body = arm.body;
    quote! { #pat => #body, }
}
