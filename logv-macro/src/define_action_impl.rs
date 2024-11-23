use proc_macro::TokenStream;
use syn::ItemFn;

pub fn define_action_impl(input: TokenStream) -> TokenStream {
    let function = parse_macro_input!(input as ItemFn);
    let name = &function.sig.ident;
    let id = name.to_string();

    quote!(
        #function

        pub const INSTANCE: crate::actions::action_impl::ActionImpl = crate::actions::action_impl::ActionImpl {
            id: #id,
            action_impl: #name,
        };
    ).into()
}
