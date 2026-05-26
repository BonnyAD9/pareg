use proc_macro2::{TokenStream, TokenTree};
use syn::{
    Attribute, Meta, Token, parse::ParseStream, punctuated::Punctuated,
    spanned::Spanned,
};

use crate::proc::{Error, Result};

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
