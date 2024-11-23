extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate syn;

// Required by parse_quote!{}
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use crate::define_action_impl::define_action_impl;

mod define_action_impl;

#[proc_macro_attribute]
pub fn define_action(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    define_action_impl(input)
}