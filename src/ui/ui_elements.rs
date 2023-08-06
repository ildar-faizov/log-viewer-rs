pub enum UIElementName {
    MainContent,
    StatusFile,
    StatusPosition,
    SearchField,
    SearchFromCursor,
    SearchBackward,
    SearchRegexp,
}

impl ToString for UIElementName {
    fn to_string(&self) -> String {
        match self {
            UIElementName::MainContent => "main_content".to_string(),
            UIElementName::StatusFile => "status_file".to_string(),
            UIElementName::StatusPosition => "status_position".to_string(),
            UIElementName::SearchField => "search_field".to_string(),
            UIElementName::SearchFromCursor => "search_from_cursor".to_string(),
            UIElementName::SearchBackward => "search_backward".to_string(),
            UIElementName::SearchRegexp => "search_regexp".to_string(),
        }
    }
}

impl From<UIElementName> for String {
    fn from(x: UIElementName) -> Self {
        x.to_string()
    }
}