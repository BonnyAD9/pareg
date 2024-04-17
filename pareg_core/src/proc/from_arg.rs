use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput};

pub fn derive_from_arg(item: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(item.into()).unwrap();

    if !input.generics.params.is_empty() {
        panic!("Cannot implement FromArg macro for generic type");
    }

    let ident = input.ident;

    let Data::Enum(input) = input.data else {
        panic!("FromArg derive macro may be used only on enums.");
    };

    let mut res = TokenStream::new();

    res.extend(input.variants.into_iter().map(|v| {
        if !v.fields.is_empty() {
            panic!("Enum variants may not have any fields")
        }
        let mut res = Literal::string(&v.ident.to_string().to_lowercase())
            .into_token_stream();
        let ident = v.ident;
        quote! { => Ok(Self::#ident), }.to_tokens(&mut res);
        res.into_iter()
    }).flatten());

    quote! {
        impl<'a> pareg::from_arg::FromArg<'a> for #ident {
            fn from_arg(arg: &'a str) -> pareg::err::Result<'a, Self> {
                match arg.trim().to_lowercase().as_str() {
                    #res
                    _ => Err(pareg::err::ArgError::FailedToParse {
                        typ: core::any::type_name::<Self>(),
                        value: arg.into(),
                        msg: Some(
                            "The value doesn't corespond to any enum variant"
                                .into()
                        ),
                    }),
                }
            }
        }
    }.into()
}
