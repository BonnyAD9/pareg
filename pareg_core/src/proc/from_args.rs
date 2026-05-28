use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Arm, Attribute, Data, DataStruct, DeriveInput, ExprAssign, ExprMatch,
    Field, Ident, LitStr, Token, Type, parse2, punctuated::Punctuated,
    spanned::Spanned,
};

use crate::proc::{Error, Result, utils::extract_attribute_list};

type UnnamedMap<'a> = HashMap<String, Vec<(&'a FieldConfig, u32)>>;

struct FieldConfig {
    ident: Ident,
    typ: Type,
    flag: bool,
    unnamed: bool,
    option: bool,
    collect: Option<Option<TokenStream>>,
    default: Option<Option<TokenStream>>,
    names: Vec<LitStr>,
    no_rewrite: Option<bool>,
}

struct FromArgsConfig {
    start_match: Vec<TokenStream>,
    end_match: Vec<TokenStream>,
    unnamed_guard: bool,
    no_rewrite: bool,
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

    let common_unnamed = declare_fields(&mut res, &fields);

    match_args(&mut res, &cfg, &fields, common_unnamed)?;

    extract_fields(&mut res, &fields);

    let names: Punctuated<_, Token![,]> =
        fields.iter().map(|a| &a.ident).collect();
    res.extend(quote! { Ok(Self { #names }) });

    Ok(quote! {
        impl<'a, S: AsRef<str>> pareg::FromArgs<'a, S> for #ident {
            fn parse_args(args: &mut pareg::ParegRef<'a, S>)
                -> pareg::Result<Self>
            {
                #res
            }
        }
    })
}

fn declare_fields<'a>(
    res: &mut TokenStream,
    fields: impl IntoIterator<Item = &'a FieldConfig>,
) -> UnnamedMap<'a> {
    let mut common_unnamed: HashMap<_, Vec<_>> = HashMap::new();
    let mut unnamed_cnt: u32 = 0;

    for field in fields {
        let id = &field.ident;
        let ty = &field.typ;

        if field.collect.is_some() {
            if let Some(Some(d)) = &field.default {
                res.extend(quote! {
                    let mut #id: #ty = #d;
                });
            } else {
                res.extend(quote! {
                    let mut #id: #ty = Default::default();
                });
            }
        } else if field.option {
            res.extend(quote! {
                let mut #id: #ty = None;
            });
        } else {
            res.extend(quote! {
                let mut #id: Option<#ty> = None;
            });
        }

        if field.unnamed {
            for n in &field.names {
                common_unnamed
                    .entry(n.value())
                    .or_default()
                    .push((field, unnamed_cnt));
            }
            unnamed_cnt += 1;
        }
    }

    common_unnamed.retain(|_, v| v.len() > 1);
    common_unnamed
}

fn match_args<'a>(
    res: &mut TokenStream,
    cfg: &FromArgsConfig,
    fields: impl IntoIterator<Item = &'a FieldConfig>,
    common_unnamed: UnnamedMap,
) -> Result<()> {
    let mut branches = TokenStream::new();

    branches.extend(cfg.start_match.iter().cloned());

    let unnamed = unique_arms(&mut branches, cfg, fields, &common_unnamed)?;

    common_arms(&mut branches, cfg, common_unnamed);

    branches.extend(cfg.end_match.iter().cloned());

    if !unnamed.is_empty() {
        res.extend(quote! {
            let mut __unnamed_bits: u64 = 0;
        });
    }

    catch_arms(&mut branches, cfg, unnamed);

    res.extend(quote! {
        while let Some(arg) = args.next() {
            match arg {
                #branches
            }
        }
    });

    Ok(())
}

fn unique_arms<'a>(
    res: &mut TokenStream,
    cfg: &FromArgsConfig,
    fields: impl IntoIterator<Item = &'a FieldConfig>,
    common_unnamed: &UnnamedMap,
) -> Result<TokenStream> {
    let mut unnamed = TokenStream::new();
    let mut unnamed_cnt: u32 = 0;

    for field in fields {
        let id = &field.ident;
        let bit: u64 = 1 << unnamed_cnt;

        if field.unnamed {
            if unnamed_cnt >= 63 {
                return Error::msg_span(id.span(), "Too many unnamed fields.")
                    .err();
            }
            let expr = field.set_field(cfg, bit, true);
            unnamed.extend(quote! {
                #unnamed_cnt => #expr,
            });
            unnamed_cnt += 1;
        }

        let pat: Punctuated<_, Token![|]> = if field.unnamed {
            field
                .names
                .iter()
                .filter(|v| !common_unnamed.contains_key(&v.value()))
                .collect()
        } else {
            field.names.iter().collect()
        };

        if pat.is_empty() {
            continue;
        }

        let expr = field.set_field(cfg, bit, false);

        res.extend(quote! {
            #pat => { #expr },
        })
    }

    Ok(unnamed)
}

fn common_arms(
    res: &mut TokenStream,
    cfg: &FromArgsConfig,
    common_unnamed: UnnamedMap,
) {
    for (n, fields) in common_unnamed {
        let mut arms = TokenStream::new();
        let mut mask = 0;
        for (field, bid) in fields {
            let bit: u64 = 1 << bid;
            mask |= bit;
            let expr = field.set_field(cfg, bit, false);
            arms.extend(quote! {
                #bid => #expr,
            });
        }
        mask = !mask;
        let msg = format!("All arguments for `{n}` already have value.");
        res.extend(quote! {
            #n => {
                let val = __unnamed_bits | #mask;
                match val.trailing_ones() {
                    #arms
                    _ => return args.err_unknown_argument().hint(#msg).err(),
                }
            }
        });
    }
}

fn catch_arms(
    res: &mut TokenStream,
    cfg: &FromArgsConfig,
    unnamed: TokenStream,
) {
    if cfg.unnamed_guard {
        res.extend(quote! {
            v if v.starts_with('-') => return args
                .err_unknown_argument()
                .hint(format!(
                    "Use explicit option to specify {v} as unnamed argument."
                ))
                .err(),
        });
    }

    if unnamed.is_empty() {
        res.extend(quote! {
            _ => return args.err_unknown_argument().err(),
        });
    } else {
        res.extend(quote! {
            _ => match __unnamed_bits.trailing_ones() {
                #unnamed
                _ => return args
                    .err_unknown_argument()
                    .hint("No more unnamed arguments were expected.")
                    .err(),
            }
        });
    }
}

fn extract_fields<'a>(
    res: &mut TokenStream,
    fields: impl IntoIterator<Item = &'a FieldConfig>,
) {
    for field in fields {
        let id = &field.ident;

        if let Some(c) = &field.collect {
            if let Some(r) = c {
                let name = field.name(false);
                let msg = format!(
                    "Expected number of arguments for `{name}` to satisfy `{r}`."
                );
                res.extend(quote! {
                    if !(#r).contains(&#id.len()) {
                        return args.map_err(
                            pareg::ArgError::invalid_number_of_arguments(#msg)
                        ).err();
                    }
                });
            }
            continue;
        }

        if field.option {
            continue;
        }

        let expr = match &field.default {
            Some(Some(v)) => quote! {
                let #id = #id.unwrap_or_else(|| #v);
            },
            Some(None) => quote! {
                let #id = #id.unwrap_or_default();
            },
            None if field.unnamed => {
                let msg = format!("Missing unnamed argument for `{id}`.");
                quote! {
                    let Some(#id) = #id else {
                        return args.err_no_more_arguments().hint(#msg).err();
                    };
                }
            }
            None if let Some(n) = field.names.first() => {
                let msg = format!("Missing required argument `{}`", n.value());
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
}

impl FieldConfig {
    pub fn set_field(
        &self,
        cfg: &FromArgsConfig,
        ubit: u64,
        cur: bool,
    ) -> TokenStream {
        let id = &self.ident;

        let mut res = TokenStream::new();
        let value = if cur {
            quote! { args.cur_arg()? }
        } else {
            quote! { args.next_arg()? }
        };

        if self.collect.is_some() {
            if self.flag {
                if cur {
                    res.extend(quote! {
                        #id.extend([#value]);
                    });
                } else {
                    res.extend(quote! {
                        #id.extend([true]);
                    });
                }
            } else {
                res.extend(quote! {
                    #id.extend([#value]);
                })
            }
        } else {
            if self.no_rewrite.unwrap_or(cfg.no_rewrite) {
                let name = self.name(cur);
                let msg =
                    format!("The argument `{name}` may be set only once.");
                res.extend(quote! {
                    if #id.is_some() {
                        return args
                            .err_cur_too_many_arguments()
                            .hint(#msg)
                            .err();
                    }
                })
            }

            if self.flag {
                if cur {
                    res.extend(quote! {
                        #id = Some(#value);
                    });
                } else {
                    res.extend(quote! {
                        #id = Some(true.into());
                    });
                }
            } else {
                res.extend(quote! {
                    #id = Some(#value);
                });
            }

            if self.unnamed {
                res.extend(quote! {
                    __unnamed_bits |= #ubit;
                });
            }
        }

        quote! { { #res } }
    }

    pub fn name(&self, cur: bool) -> String {
        if !cur && let Some(v) = self.names.first() {
            v.value()
        } else {
            self.ident.to_string()
        }
    }

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
        let mut unnamed = false;
        let mut collect = None;
        let mut no_rewrite = None;
        let mut option = false;

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
                        "unnamed" => unnamed = true,
                        "option" => option = true,
                        "collect" => collect = Some(None),
                        "no_rewrite" => no_rewrite = Some(true),
                        "rewrite" => no_rewrite = Some(false),
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
                            default = Some(Some(a.right.into_token_stream()));
                        }
                        "collect" => {
                            collect = Some(Some(a.right.into_token_stream()));
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

        if option && collect.is_some() {
            return Error::msg_span(
                span,
                "`option` and `collect` are incompatible.",
            )
            .err();
        }

        Ok(Self {
            ident,
            typ,
            flag,
            unnamed,
            collect,
            names,
            default,
            no_rewrite,
            option,
        })
    }
}

impl FromArgsConfig {
    pub fn parse(attrs: Vec<Attribute>) -> Result<Self> {
        let mut start_match = vec![];
        let mut end_match = vec![];
        let mut unnamed_guard = false;
        let mut no_rewrite = false;

        for attr in attrs {
            if !attr.path().is_ident("from_args") {
                continue;
            }
            let vars = extract_attribute_list(attr)?;
            for v in vars {
                if let Ok(a) = parse2::<Ident>(v.clone()) {
                    match a.to_string().as_str() {
                        "unnamed_guard" => unnamed_guard = true,
                        "no_rewrite" => no_rewrite = true,
                        _ => {
                            return Error::msg_span(
                                a.span(),
                                "Unknown option for from_args.",
                            )
                            .err();
                        }
                    }
                } else if let Ok(a) = parse2::<ExprMatch>(v.clone()) {
                    let id = parse2::<Ident>(a.expr.into_token_stream())?;
                    match id.to_string().as_str() {
                        "start" => {
                            start_match.extend(
                                a.arms.into_iter().map(arm_to_token_stream),
                            );
                        }
                        "end" => {
                            end_match.extend(
                                a.arms.into_iter().map(arm_to_token_stream),
                            );
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
            unnamed_guard,
            no_rewrite,
        })
    }
}

fn arm_to_token_stream(arm: Arm) -> TokenStream {
    let pat = arm.pat;
    let body = arm.body;
    quote! { #pat => #body, }
}
