use proc_macro2::{Literal, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, Ident, LitChar, LitStr, Meta,
    Token, parse::ParseStream, parse2, punctuated::Punctuated,
    spanned::Spanned,
};

use crate::proc::{Error, Result};

#[derive(Default)]
struct FromArgConfig {
    exact: Option<bool>,
    split: Option<char>,
    matches: Vec<LitStr>,
    default: Option<Expr>,
}

/// Implementation of the derive proc macro for [`crate::FromArg`]
pub fn derive_from_arg(item: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse2(item)?;

    // Get the ident of the enum
    let ident = input.ident;

    if !input.generics.params.is_empty() {
        return Error::msg_span(
            input.generics.span(),
            "FromArg doens't support generics.",
        )
        .err();
    }

    let conf = FromArgConfig::parse(input.attrs)?;

    if let Some(s) = conf.matches.first() {
        return Error::msg_span(
            s.span(),
            "String matches may be used only on variants.",
        )
        .err();
    }

    // Check that it is enum
    let Data::Enum(input) = input.data else {
        return Error::msg_span(
            ident.span(),
            "FromArg derive macro may be used only on enums.",
        )
        .err();
    };

    let mut res = TokenStream::new();

    let mut variants = vec![];

    // Create match arms for all enum variants
    for v in input.variants {
        let ident = v.ident;

        let mut ors = TokenStream::new();

        let conf = FromArgConfig::parse(v.attrs)?.unite(&conf);
        if !conf.exact() {
            // Get the lowercase name of the enum as the first literal in the match
            let variant = ident.to_string().to_lowercase();
            Literal::string(&variant).to_tokens(&mut ors);
            variants.push(variant);
        }

        if conf.exact() && conf.matches.is_empty() {
            continue;
        }

        // Ensure the enum has at most one field.
        if v.fields.len() > 1 || matches!(v.fields, Fields::Named(_)) {
            return Error::msg_span(
                ident.span(),
                "From arg supports at most one unnamed field.",
            )
            .err();
        }

        let sep = if v.fields.is_empty() {
            quote! { | }
        } else {
            quote! { , }
        };

        let mut add_or = !conf.exact();

        // Add the variants from the '#[arg()]' attributes
        for m in &conf.matches {
            if add_or {
                sep.to_tokens(&mut ors);
            } else {
                add_or = true;
            }
            variants.push(m.value());
            if conf.exact() {
                m.to_tokens(&mut ors);
            } else {
                LitStr::new(&m.value().to_lowercase(), m.span())
                    .to_tokens(&mut ors);
            }
        }

        if v.fields.is_empty() {
            quote! { #ors => Ok(Self::#ident), }.to_tokens(&mut res);
        } else {
            let spl = LitChar::new(conf.split(), Span::call_site());
            if let Some(d) = conf.default {
                quote! {
                    v if pareg::has_any_key!(v, #spl, #ors) => Ok(Self::#ident(
                        pareg::mval_arg(v, #spl)?.unwrap_or_else(|| #d)
                    )),
                }
                .to_tokens(&mut res);
            } else {
                quote! {
                    v if pareg::has_any_key!(v, #spl, #ors) => Ok(Self::#ident(
                        pareg::val_arg(v, #spl)?
                    )),
                }
                .to_tokens(&mut res);
            }
        }
    }

    let mut hint = "Valid options are: ".to_string();
    for v in variants.iter().take(5) {
        hint += &format!("`{v}`, ");
    }
    if variants.len() > 5 {
        hint += "..."
    } else {
        hint.pop();
        hint.pop();
        hint.push('.');
    }
    let hint = Literal::string(&hint).to_token_stream();

    let expr = if conf.exact() {
        quote! { arg }
    } else {
        quote! { arg.trim().to_lowercase().as_str() }
    };

    let res = quote! {
        impl<'a> pareg::FromArg<'a> for #ident {
            fn from_arg(arg: &'a str) -> pareg::Result<Self> {
                match #expr {
                    #res
                    _ => {
                        pareg::ArgError::new(pareg::ArgErrCtx {
                            args: vec![arg.into()],
                            error_span: 0..arg.len(),
                            inline_msg: Some("Unknown option.".into()),
                            long_msg: Some(
                                format!("Unknown option `{arg}`.").into()
                            ),
                            hint: Some(#hint.into()),
                            ..pareg::ArgErrCtx::new(
                                pareg::ArgErrKind::FailedToParse
                            )
                        }).err()
                    },
                }
            }
        }
    };

    Ok(res)
}

impl FromArgConfig {
    pub fn parse(attrs: Vec<Attribute>) -> Result<Self> {
        let mut res = Self::default();

        // Add the variants from the '#[arg()]' attributes
        for attr in attrs.into_iter().filter(
            |a| matches!(&a.meta, Meta::List(l) if l.path.is_ident("arg")),
        ) {
            let vars = extract_attribute_list(attr)?;

            for v in vars {
                if let Ok(i) = parse2::<Ident>(v.clone()) {
                    match i.to_string().as_str() {
                        "exact" => res.exact = Some(true),
                        "default" => {
                            res.default = Some(
                                parse2(quote! { Default::default() }).unwrap(),
                            )
                        }
                        _ => {
                            return Error::msg_span(
                                i.span(),
                                "Unknown FromArg option.",
                            )
                            .err();
                        }
                    }
                } else if let Ok(ass) = parse2::<syn::ExprAssign>(v.clone()) {
                    let id: Ident = parse2(ass.left.into_token_stream())?;
                    match id.to_string().as_str() {
                        "split" => {
                            let chr: LitChar =
                                parse2(ass.right.into_token_stream())?;
                            res.split = Some(chr.value());
                        }
                        "default" => res.default = Some(*ass.right),
                        _ => {
                            return Error::msg_span(
                                id.span(),
                                "Unknown FromArg option.",
                            )
                            .err();
                        }
                    }
                } else {
                    res.matches.push(parse2(v)?);
                }
            }
        }

        Ok(res)
    }

    pub fn unite(mut self, other: &Self) -> Self {
        self.exact = self.exact.or(other.exact);
        self.split = self.split.or(other.split);
        self
    }

    pub fn exact(&self) -> bool {
        self.exact.unwrap_or_default()
    }

    pub fn split(&self) -> char {
        self.split.unwrap_or('=')
    }
}

pub fn extract_attribute_list(
    attribute: Attribute,
) -> Result<Punctuated<TokenStream, Token![,]>> {
    match attribute.meta {
        Meta::List(l) => Ok(l.parse_args_with(punctuated_streams)?),
        Meta::Path(_) => Ok(Punctuated::default()),
        _ => {
            Error::msg_span(attribute.span(), "Invalid attribute style.").err()
        }
    }
}

pub fn punctuated_streams(
    input: ParseStream,
) -> syn::Result<Punctuated<TokenStream, Token![,]>> {
    let mut res = Punctuated::new();
    loop {
        let mut tokens = TokenStream::new();

        while !input.is_empty() && !input.peek(Token![,]) {
            tokens.extend([input.parse::<TokenTree>()?]);
        }

        if tokens.is_empty() && input.is_empty() {
            break;
        }

        res.push_value(tokens);

        let Ok(c) = input.parse() else {
            break;
        };
        res.push_punct(c);
    }

    Ok(res)
}
