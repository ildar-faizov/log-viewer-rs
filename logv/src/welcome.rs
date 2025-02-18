use lazy_static::lazy_static;

pub const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");


lazy_static! {
    pub static ref WELCOME: String = {
        let welcome = include_str!("assets/welcome.txt");
        let build_info = format!("{}@{}", CARGO_PKG_NAME, CARGO_PKG_VERSION);
        welcome.replace("{{BUILD_INFO}}", &build_info)
    };
}