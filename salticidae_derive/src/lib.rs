extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
mod deserialize;
mod helpers;
mod serialize;

#[proc_macro_derive(Deserialize)]
pub fn deserialize_macro_derive(input: TokenStream) -> TokenStream {
    deserialize::deserialize_macro_derive(input)
}

#[proc_macro_derive(Serialize)]
pub fn serialize_macro_derive(input: TokenStream) -> TokenStream {
    serialize::serialize_macro_derive(input)
}
