use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Expr, Ident, LitStr, Token, parse::Parser, parse2, punctuated::Punctuated,
};

pub fn proc_parsef(args: TokenStream, part: bool) -> TokenStream {
    let mut input =
        Parser::parse2(Punctuated::<Expr, Token![,]>::parse_terminated, args)
            .unwrap()
            .into_iter();
    let reader = input.next().expect("Missing reader as first argument");
    let pattern: LitStr = parse2(
        input
            .next()
            .expect("Missing literal as second argument.")
            .to_token_stream(),
    )
    .unwrap();
    let span = pattern.span();
    let pattern = pattern.value();
    let mut p = pattern.as_str();

    let mut args = TokenStream::new();

    while !p.is_empty() {
        let Some(pos) = p.find(['{', '}']) else {
            let lit = LitStr::new(p, span);
            args.extend(quote! { pareg::ParseFArg::Str(#lit.into()), });
            p = &p[p.len()..];
            continue;
        };

        if p[pos..].starts_with("{{") || p[pos..].starts_with("}}") {
            let lit = LitStr::new(&p[..=pos], span);
            args.extend(quote! { pareg::ParseFArg::Str(#lit.into()), });
            p = &p[pos + 2..];
            continue;
        }

        if p[pos..].starts_with('}') {
            panic!("Invalid closing bracket.");
        }

        let lit = LitStr::new(&p[..pos], span);
        args.extend(quote! { pareg::ParseFArg::Str(#lit.into()), });
        p = &p[pos + 1..];

        let Some(pos) = p.find("}") else {
            panic!("Missing closing '}}'");
        };

        let inline = &p[..pos];
        let (id, fmt) = if let Some((i, f)) = inline.split_once(':') {
            (i.trim(), f)
        } else {
            (inline.trim(), "")
        };

        if id.is_empty() {
            let arg = input.next();
            if let Some(fid) = fmt.strip_prefix('$') {
                let fid = Ident::new(fid, span);
                args.extend(quote! { pareg::ParseFArg::Arg(#arg, &#fid), });
            } else {
                args.extend(
                    quote! { pareg::ParseFArg::Arg(#arg, &#fmt.into()), },
                );
            }
        } else {
            let id = Ident::new(id, span);
            if let Some(fid) = fmt.strip_prefix('$') {
                let fid = Ident::new(fid, span);
                args.extend(
                    quote! { pareg::ParseFArg::Arg(&mut #id, &#fid), },
                );
            } else {
                args.extend(
                    quote! { pareg::ParseFArg::Arg(&mut #id, &#fmt.into()), },
                );
            }
        }

        p = &p[pos + 1..];
    }

    if part {
        quote! {
            pareg::parsef_part(#reader, [#args])
        }
    } else {
        quote! {
            pareg::parsef(#reader, [#args])
        }
    }
}
