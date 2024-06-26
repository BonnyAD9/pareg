use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Data, DeriveInput, LitStr, Meta, Token};

pub fn derive_from_arg(item: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(item.into()).unwrap();

    // Ensure that there are no generics
    if !input.generics.params.is_empty() {
        panic!("Cannot implement FromArg macro for generic type");
    }

    // Get the ident of the enum
    let ident = input.ident;

    // Check that it is enum
    let Data::Enum(input) = input.data else {
        panic!("FromArg derive macro may be used only on enums.");
    };

    let mut res = TokenStream::new();

    // Create match arms for all enum variants
    res.extend(input.variants.into_iter().flat_map(|v| {
        // Ensure the enum has no fields.
        if !v.fields.is_empty() {
            panic!("Enum variants may not have any fields")
        }

        let ident = v.ident;

        // Get the lowercase name of the enum as the first literal in the match
        let mut res = Literal::string(&ident.to_string().to_lowercase())
            .into_token_stream();

        // Add the variants from the '#[arg()]' attributes
        for attr in v.attrs.into_iter().filter(
            |a| matches!(&a.meta, Meta::List(l) if l.path.is_ident("arg")),
        ) {
            let vars = attr
                .parse_args_with(
                    Punctuated::<LitStr, Token![|]>::parse_terminated,
                )
                .expect("Invalid arguments to the attribute '#[arg(...)]'");

            if !vars.is_empty() {
                quote! { | }.to_tokens(&mut res);
                vars.to_tokens(&mut res);
            }
        }

        quote! { => Ok(Self::#ident), }.to_tokens(&mut res);
        res.into_iter()
    }));

    quote! {
        impl<'a> pareg::FromArg<'a> for #ident {
            fn from_arg(arg: &'a str) -> pareg::Result<Self> {
                match arg.trim().to_lowercase().as_str() {
                    #res
                    _ => Err(pareg::ArgError::FailedToParse {
                        typ: core::any::type_name::<Self>(),
                        value: arg.to_owned().into(),
                        msg: Some(
                            "The value doesn't corespond to any enum variant"
                                .into()
                        ),
                    }),
                }
            }
        }
    }
}
