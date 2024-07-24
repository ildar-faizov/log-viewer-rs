use yaml_rust2::Yaml;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionDescription {
    id: String,
    description: Option<String>,
    hotkeys: Vec<String>,
}

impl From<&Yaml> for ActionDescription {

    /// Sample YAML fragment:
    /// ```yaml
    /// id: open_file
    /// description: Open file
    /// hotkeys: [Ctrl+O]
    /// ```
    fn from(value: &Yaml) -> Self {
        let id = value["id"].as_str().unwrap().to_string();
        let description = value["description"].as_str().map(|s| s.to_string());
        let hotkeys = value["hotkeys"].as_vec().map(|h| {
            h.iter().map(|hk| hk.as_str().unwrap().to_string()).collect()
        }).unwrap_or_default();
        ActionDescription {
            id,
            description,
            hotkeys,
        }
    }
}

impl ActionDescription {
    pub fn new(
        id: impl ToString,
        description: Option<impl ToString>,
        hotkeys: Vec<impl ToString>
    ) -> Self {
        Self {
            id: id.to_string(),
            description: description.map(|t| t.to_string()),
            hotkeys: hotkeys.into_iter().map(|t| t.to_string()).collect(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn hotkeys(&self) -> &Vec<String> {
        &self.hotkeys
    }

    pub fn combine(&self, rhs: &ActionDescription) -> ActionDescription {
        assert_eq!(self.id(), rhs.id());
        let hotkeys = if rhs.hotkeys.is_empty() {
            self.hotkeys.clone()
        } else {
            rhs.hotkeys.clone()
        };
        ActionDescription {
            id: self.id().to_string(),
            description: rhs.description().or(self.description()).map(String::from),
            hotkeys
        }
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;
    use trim_margin::MarginTrimmable;
    use yaml_rust2::YamlLoader;

    use crate::profiles::action_description::ActionDescription;
    use crate::profiles::profile::Profile;
    use crate::test_extensions::UniqueElementAssertions;

    #[test]
    fn test_parsing() {
        let s = "
            |id: word_forward
            |description: Move cursor one word forward
            |hotkeys: [Ctrl+RIGHT]
            ".trim_margin().unwrap();
        let docs = YamlLoader::load_from_str(s.as_str()).unwrap();
        let doc = &docs[0];
        let actual = ActionDescription::from(doc);
        assert_that!(actual.id).is_equal_to("word_forward".to_string());
        assert_that!(actual.description).contains_value("Move cursor one word forward".to_string());
        assert_that!(actual.hotkeys).has_only_element().is_equal_to("Ctrl+RIGHT".to_string());
    }

    #[test]
    fn test_combine_full() {
        let left = ActionDescription::new(
            "foo",
            Some("Basic description"),
            vec!["Ctrl+o"]
        );
        let right = ActionDescription::new(
            "foo",
            Some("Overridden description"),
            vec!["pgdown"]
        );
        let sum = left.combine(&right);
        assert_that!(sum.id()).is_equal_to("foo");
        assert_that!(sum.description()).contains("Overridden description");
        assert_that!(sum.hotkeys()).is_equal_to(&vec!["pgdown".to_string()]);
    }

    #[test]
    fn test_combine_fallback_description() {
        let left = ActionDescription::new(
            "foo",
            Some("Basic description"),
            vec!["Ctrl+o"]
        );
        let right = ActionDescription::new(
            "foo",
            Option::<String>::None,
            vec!["pgdown"]
        );
        let sum = left.combine(&right);
        assert_that!(sum.id()).is_equal_to("foo");
        assert_that!(sum.description()).contains("Basic description");
        assert_that!(sum.hotkeys()).is_equal_to(&vec!["pgdown".to_string()]);
    }

    #[test]
    fn test_combine_fallback_hotkeys() {
        let left = ActionDescription::new(
            "foo",
            Some("Basic description"),
            vec!["Ctrl+o"]
        );
        let right = ActionDescription::new(
            "foo",
            Some("Overridden description"),
            Vec::<String>::new()
        );
        let sum = left.combine(&right);
        assert_that!(sum.id()).is_equal_to("foo");
        assert_that!(sum.description()).contains("Overridden description");
        assert_that!(sum.hotkeys()).is_equal_to(&vec!["Ctrl+o".to_string()]);
    }
}