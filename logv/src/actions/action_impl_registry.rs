include!(concat!(env!("OUT_DIR"), "/action_impl_registry.rs"));

// The following constant id populated at compile-time:
// pub static ref REGISTRY: Vec<crate::actions::action_impl::ActionImpl>