pub trait PreAction {
    /// Internal identifier of an action
    ///
    /// Mainly used to match action in config
    fn id(&self) -> &str;
}

#[macro_export]
macro_rules! make_action {
    ($id: ident) => {
        paste! {
            pub struct [<$id:camel Action>] {}

            impl [<$id:camel Action>] {
                pub const fn new() -> Self {
                    Self {}
                }
            }

            impl crate::actions::pre_action::PreAction for [<$id:camel Action>] {
                fn id(&self) -> &str {
                    stringify!($id)
                }
            }

            pub const IMPL: [<$id:camel Action>] = [<$id:camel Action>]::new();
        }
    };
}
