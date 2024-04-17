use proc_macro::TokenStream;

#[proc_macro_derive(FromArg)]
pub fn derive_from_arg(item: TokenStream) -> TokenStream {
    pareg_core::proc::from_arg::derive_from_arg(item.into()).into()
}
