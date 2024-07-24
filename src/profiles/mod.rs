use lazy_static::lazy_static;
use yaml_rust2::YamlLoader;

pub use action_description::ActionDescription;
pub use profile::Profile;

mod action_description;
mod profile;

pub const DEFAULT_PROFILE_NAME: &str = "default";

lazy_static! {
    pub static ref PROFILES: Vec<Profile> = load_builtin_profiles();
    pub static ref OS_PROFILE: Profile = {
        let default_profile = PROFILES.iter()
            .find(|p| p.name() == DEFAULT_PROFILE_NAME)
            .unwrap();
        let os_profile = PROFILES.iter()
            .find(|p| p.name() == std::env::consts::OS);
        if let Some(os_profile) = os_profile {
            default_profile.combine(os_profile)
        } else {
            default_profile.clone()
        }
    };
}

fn load_builtin_profiles() -> Vec<Profile> {
    let profiles_yaml = include_str!("../assets/profiles.yaml");
    YamlLoader::load_from_str(profiles_yaml)
        .expect("Failed to parse built-in profiles")
        .iter()
        .map(Profile::from)
        .collect()
}