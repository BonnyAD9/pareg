use std::{collections::HashMap, fmt::Write};

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Arm, Attribute, Data, DataStruct, DeriveInput, ExprArray, ExprAssign,
    ExprMatch, Field, Ident, LitStr, Token, Type, parse2,
    punctuated::Punctuated, spanned::Spanned,
};

use crate::proc::{Error, Result, utils::extract_attribute_list};

type PositionalMap<'a> = HashMap<String, Vec<(&'a FieldConfig, u32)>>;
type IdMap<'a> = HashMap<String, &'a FieldConfig>;

struct FieldConfig {
    ident: Ident,
    typ: Type,
    flag: bool,
    positional: bool,
    option: bool,
    collect: Option<Option<TokenStream>>,
    default: Option<Option<TokenStream>>,
    names: Vec<LitStr>,
    check: Option<TokenStream>,
    no_rewrite: Option<bool>,
    conflict: Vec<String>,
    require: Vec<String>,
}

struct FromArgsConfig {
    start_match: Vec<TokenStream>,
    end_match: Vec<TokenStream>,
    positional_guard: bool,
    no_rewrite: bool,
    check: Vec<TokenStream>,
    conflict: Vec<Vec<String>>,
    require: Vec<Vec<String>>,
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

    let (common_unnamed, id_map) = declare_fields(&mut res, &fields);

    match_args(&mut res, &cfg, &fields, common_unnamed)?;

    validate_conflicts(&mut res, &fields, &id_map);
    validate_mutual_conflicts(&mut res, &cfg, &id_map);
    validate_require(&mut res, &fields, &id_map);
    validate_mutual_require(&mut res, &cfg, &id_map);
    validate_field_check(&mut res, &fields);
    validate_check(&mut res, &cfg);

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
) -> (PositionalMap<'a>, IdMap<'a>) {
    let mut common_unnamed: HashMap<_, Vec<_>> = HashMap::new();
    let mut id_map = HashMap::new();
    let mut unnamed_cnt: u32 = 0;

    for field in fields {
        let id = &field.ident;
        let ty = &field.typ;

        id_map.insert(id.to_string(), field);

        if field.collect.is_some() {
            let value = if let Some(Some(d)) = &field.default {
                d.clone()
            } else {
                quote! { Default::default() }
            };

            if field.option {
                res.extend(quote! {
                    let #id: #ty = Some(#value);
                    let mut #id = #id.unwrap();
                })
            } else {
                res.extend(quote! {
                    let mut #id: #ty = #value;
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

        if field.positional {
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
    (common_unnamed, id_map)
}

fn match_args<'a>(
    res: &mut TokenStream,
    cfg: &FromArgsConfig,
    fields: impl IntoIterator<Item = &'a FieldConfig>,
    common_unnamed: PositionalMap,
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
    common_positional: &PositionalMap,
) -> Result<TokenStream> {
    let mut unnamed = TokenStream::new();
    let mut positional_cnt: u32 = 0;

    for field in fields {
        let id = &field.ident;
        let bit: u64 = 1 << positional_cnt;

        if field.positional {
            if positional_cnt >= 63 {
                return Error::msg_span(id.span(), "Too many unnamed fields.")
                    .err();
            }
            let expr = field.set_field(cfg, bit, true);
            unnamed.extend(quote! {
                #positional_cnt => #expr,
            });
            positional_cnt += 1;
        }

        let pat: Punctuated<_, Token![|]> = if field.positional {
            field
                .names
                .iter()
                .filter(|v| !common_positional.contains_key(&v.value()))
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
    common_positional: PositionalMap,
) {
    for (n, fields) in common_positional {
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
    if cfg.positional_guard {
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

fn validate_conflicts<'a>(
    res: &mut TokenStream,
    fields: impl IntoIterator<Item = &'a FieldConfig>,
    id_map: &IdMap,
) {
    for field in fields {
        let name = field.name(false);
        let mut checks = TokenStream::new();

        for conflict in &field.conflict {
            let cfield = &id_map[conflict];
            let cond = cfield.is_set();
            let cname = cfield.name(false);
            let msg = format!(
                "`{name}` and `{cname}` are in conflict and they cannot be \
                used together."
            );

            checks.extend(quote! {
                if #cond {
                    return args.err_invalid().hint(#msg).err();
                }
            });
        }

        let cond = field.is_set();

        res.extend(quote! {
            if #cond {
                #checks
            }
        });
    }
}

fn validate_mutual_conflicts(
    res: &mut TokenStream,
    cfg: &FromArgsConfig,
    id_map: &IdMap,
) {
    for group in &cfg.conflict {
        for (i, field) in group.iter().enumerate() {
            if i + 1 >= group.len() {
                continue;
            }

            let field = id_map[field];
            let name = field.name(false);
            let mut checks = TokenStream::new();

            for conflict in &group[i + 1..] {
                let cfield = &id_map[conflict];
                let cond = cfield.is_set();
                let cname = cfield.name(false);
                let msg = format!(
                    "`{name}` and `{cname}` are in conflict and they cannot \
                    be used together."
                );

                checks.extend(quote! {
                    if #cond {
                        return args.err_invalid().hint(#msg).err();
                    }
                });
            }

            let cond = field.is_set();

            res.extend(quote! {
                if #cond {
                    #checks
                }
            });
        }
    }
}

fn validate_require<'a>(
    res: &mut TokenStream,
    fields: impl IntoIterator<Item = &'a FieldConfig>,
    id_map: &IdMap,
) {
    for field in fields {
        let name = field.name(false);
        let mut checks = TokenStream::new();

        for require in &field.require {
            let rfield = &id_map[require];
            let cond = rfield.is_set();
            let rname = rfield.name(false);
            let msg = format!(
                "When using `{name}`, `{rname}` has to be specified as well."
            );

            checks.extend(quote! {
                if !#cond {
                    return args.err_no_more_arguments().hint(#msg).err();
                }
            });
        }

        let cond = field.is_set();

        res.extend(quote! {
            if #cond {
                #checks
            }
        });
    }
}

fn validate_mutual_require(
    res: &mut TokenStream,
    cfg: &FromArgsConfig,
    id_map: &IdMap,
) {
    for group in &cfg.require {
        res.extend(quote! {
            let mut __cnt: usize = 0;
        });

        let mut msg = "Options ".to_string();

        for field in group {
            let field = id_map[field];
            let name = field.name(false);
            let cond = field.is_set();
            _ = write!(msg, "`{name}, `");
            res.extend(quote! {
                __cnt += #cond as usize;
            });
        }

        msg.pop();
        msg.pop();
        msg += "have to be all used together (either none or all).";
        let len = group.len();
        res.extend(quote! {
            if __cnt != #len {
                return args.err_no_more_arguments().hint(#msg).err();
            }
        });
    }
}

fn validate_field_check<'a>(
    res: &mut TokenStream,
    fields: impl IntoIterator<Item = &'a FieldConfig>,
) {
    for field in fields {
        let Some(check) = &field.check else {
            continue;
        };

        let id = &field.ident;

        let prefix = if field.collect.is_some() {
            quote! { !#id.is_empty() }
        } else {
            quote! { let Some(ref #id) = #id }
        };

        let name = field.name(false);
        let msg = format!("Argument `{name}` is allowed only if `{check}`.");

        res.extend(quote! {
            if #prefix && !(#check) {
                return args.err_invalid().hint(#msg).err();
            }
        });
    }
}

fn validate_check(res: &mut TokenStream, cfg: &FromArgsConfig) {
    for c in &cfg.check {
        let msg = format!("The check failed: `{c}`");
        res.extend(quote! {
            if !(#c) {
                return args.err_invalid().hint(#msg).err();
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
            if field.option {
                res.extend(quote! {
                    let #id = (!#id.is_empty()).then_some(#id);
                })
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
            None if field.positional => {
                let msg = format!("Missing positional argument for `{id}`.");
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

            if self.positional {
                res.extend(quote! {
                    __unnamed_bits |= #ubit;
                });
            }
        }

        quote! { { #res } }
    }

    pub fn is_set(&self) -> TokenStream {
        let id = &self.ident;
        if self.collect.is_some() {
            quote! { (!#id.is_empty()) }
        } else {
            quote! { #id.is_some() }
        }
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
        let mut positional = false;
        let mut collect = None;
        let mut no_rewrite = None;
        let mut option = false;
        let mut check = None;
        let mut conflict = vec![];
        let mut require = vec![];

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
                        "positional" => positional = true,
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
                        "check" => {
                            check = Some(a.right.into_token_stream());
                        }
                        "conflict" => {
                            let idents = parse2::<ExprArray>(
                                a.right.into_token_stream(),
                            )?;
                            let ids = idents.elems.into_iter().map(|a| {
                                parse2::<Ident>(a.into_token_stream())
                                    .map(|a| a.to_string())
                            });
                            for id in ids {
                                conflict.push(id?);
                            }
                        }
                        "require" => {
                            let idents = parse2::<ExprArray>(
                                a.right.into_token_stream(),
                            )?;
                            let ids = idents.elems.into_iter().map(|a| {
                                parse2::<Ident>(a.into_token_stream())
                                    .map(|a| a.to_string())
                            });
                            for id in ids {
                                require.push(id?);
                            }
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
            positional,
            collect,
            names,
            default,
            no_rewrite,
            option,
            check,
            conflict,
            require,
        })
    }
}

impl FromArgsConfig {
    pub fn parse(attrs: Vec<Attribute>) -> Result<Self> {
        let mut start_match = vec![];
        let mut end_match = vec![];
        let mut positional_guard = false;
        let mut no_rewrite = false;
        let mut check = vec![];
        let mut conflict = vec![];
        let mut require = vec![];

        for attr in attrs {
            if !attr.path().is_ident("from_args") {
                continue;
            }
            let vars = extract_attribute_list(attr)?;
            for v in vars {
                if let Ok(a) = parse2::<Ident>(v.clone()) {
                    match a.to_string().as_str() {
                        "positional_guard" => positional_guard = true,
                        "no_rewrite" => no_rewrite = true,
                        o => {
                            return Error::msg_span(
                                a.span(),
                                format!("Unknown option `{o}` for from_args."),
                            )
                            .err();
                        }
                    }
                } else if let Ok(a) = parse2::<ExprAssign>(v.clone()) {
                    let id: Ident = parse2(a.left.into_token_stream())?;
                    match id.to_string().as_str() {
                        "check" => {
                            check.push(a.right.into_token_stream());
                        }
                        "conflict" => {
                            let idents = parse2::<ExprArray>(
                                a.right.into_token_stream(),
                            )?;
                            let ids = idents
                                .elems
                                .into_iter()
                                .map(|a| {
                                    parse2::<Ident>(a.into_token_stream())
                                        .map(|a| a.to_string())
                                })
                                .collect::<Result<_, _>>()?;
                            conflict.push(ids);
                        }
                        "require" => {
                            let idents = parse2::<ExprArray>(
                                a.right.into_token_stream(),
                            )?;
                            let ids = idents
                                .elems
                                .into_iter()
                                .map(|a| {
                                    parse2::<Ident>(a.into_token_stream())
                                        .map(|a| a.to_string())
                                })
                                .collect::<Result<_, _>>()?;
                            require.push(ids);
                        }
                        o => {
                            return Error::msg_span(
                                id.span(),
                                format!("Unknown option `{o}` for from_args."),
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
                        "Unknown expression for FromArg.",
                    )
                    .err();
                }
            }
        }

        Ok(Self {
            start_match,
            end_match,
            positional_guard,
            no_rewrite,
            check,
            conflict,
            require,
        })
    }
}

fn arm_to_token_stream(arm: Arm) -> TokenStream {
    let pat = arm.pat;
    let body = arm.body;
    quote! { #pat => #body, }
}
