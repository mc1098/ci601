use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

mod resolver;

use resolver::DeriveEntryInput;

#[proc_macro_derive(Entry, attributes(kind))]
pub fn derive_resolver(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveEntryInput);
    TokenStream::from(input.into_token_stream())
}
