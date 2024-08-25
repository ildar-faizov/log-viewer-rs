use std::collections::HashMap;

use itertools::Itertools;
use yaml_rust2::Yaml;

use crate::profiles::action_description::ActionDescription;

#[derive(Clone)]
pub struct Profile {
    name: String,
    actions: Vec<ActionDescription>
}

impl From<&Yaml> for Profile {

    /// Sample YAML fragment:
    /// ```yaml
    /// profile:
    ///   name: default
    ///   actions:
    ///     - id: open_file
    ///       description: Open file
    ///       hotkeys: [Ctrl+O]
    ///     - id: scroll_up
    ///       description: Scroll one line up
    ///       hotkeys: [Ctrl+UP]
    /// ```
    fn from(value: &Yaml) -> Self {
        let profile = &value["profile"];
        let name = profile["name"].as_str().unwrap().to_string();
        let actions: Vec<ActionDescription> = profile["actions"]
            .as_vec()
            .map(|arr| {
                arr.iter()
                    .map(|action| ActionDescription::from(action))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            name,
            actions,
        }
    }
}

impl Profile {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn actions(&self) -> &[ActionDescription] {
        &self.actions[..]
    }

    pub fn lookup(&self, id: &str) -> Option<&ActionDescription> {
        self.actions.iter().find(|ad| ad.id() == id)
    }

    pub fn combine(&self, rhs: &Profile) -> Profile {
        let mut right_actions = HashMap::with_capacity(rhs.actions.len());
        for ad in &rhs.actions {
            right_actions.insert(ad.id(), ad);
        }

        let actions = self.actions.iter()
            .map(|x| {
                let y = right_actions.get(x.id());
                match y {
                    None => x.clone(),
                    Some(y) => x.combine(*y),
                }
            }).collect_vec();
        Self {
            name: rhs.name.clone(),
            actions
        }
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;
    use trim_margin::MarginTrimmable;
    use yaml_rust2::YamlLoader;

    use crate::profiles::ActionDescription;

    use super::Profile;

    #[test]
    fn test_parsing() {
        let s = "
            |profile:
            |  name: Test Profile
            |  actions:
            |    - id: open_file
            |      description: Some action
            |      hotkeys: [Ctrl+o]
            |    - id: quit
            |      description: Quit
            |      hotkeys: [q]
            ".trim_margin().unwrap();
        let docs = YamlLoader::load_from_str(s.as_str()).unwrap();
        let doc = &docs[0];
        let actual = Profile::from(doc);
        assert_that!(actual.name).is_equal_to("Test Profile".to_string());
        assert_that!(actual.actions).has_length(2);
    }

    #[test]
    fn test_combine() {
        let base = Profile {
            name: "base".to_string(),
            actions: vec![
                ActionDescription::new("foo", Some("Basic description"), vec!["Ctrl+o"]),
                ActionDescription::new("bar", Some("Bar description"), vec!["Shift+pgdown"]),
            ]
        };
        let specific = Profile {
            name: "specific".to_string(),
            actions: vec![
                ActionDescription::new("foo", Some("Overridden description"), vec!["Shift+LEFT"]),
            ]
        };
        let composition = base.combine(&specific);

        let expected: [ActionDescription; 2] = [
            ActionDescription::new("foo", Some("Overridden description"), vec!["Shift+LEFT"]),
            ActionDescription::new("bar", Some("Bar description"), vec!["Shift+pgdown"]),
        ];

        assert_that!(composition.name()).is_equal_to("specific");
        assert_that!(composition.actions()).is_equal_to(&expected[..]);
    }
}