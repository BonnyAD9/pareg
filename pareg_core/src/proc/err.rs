use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitStr;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Syn(#[from] syn::Error),
    #[error("{1}")]
    Msg(Span, Cow<'static, str>),
}

impl Error {
    pub fn msg_span(span: Span, msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Msg(span, msg.into())
    }

    pub fn err<T>(self) -> Result<T> {
        Err(self)
    }

    pub fn into_token_stream(self) -> TokenStream {
        match self {
            Self::Syn(e) => e.into_compile_error(),
            Self::Msg(span, msg) => {
                let msg = LitStr::new(&msg, span);
                quote! { compile_error!(#msg); }
            }
        }
    }
}

pub fn result_to_token_stream(r: Result<TokenStream>) -> TokenStream {
    match r {
        Ok(r) => r,
        Err(e) => e.into_token_stream(),
    }
}
