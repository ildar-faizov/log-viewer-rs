pub enum UIElementName {
    MainContent,
    StatusFile,
    StatusPosition,
    StatusHint,
    SearchField,
    SearchFromCursor,
    SearchBackward,
    SearchRegexp,
    HelpTable,
}

impl ToString for UIElementName {
    fn to_string(&self) -> String {
        let str = match self {
            UIElementName::MainContent => "main_content",
            UIElementName::StatusFile => "status_file",
            UIElementName::StatusPosition => "status_position",
            UIElementName::StatusHint => "status_hint",
            UIElementName::SearchField => "search_field",
            UIElementName::SearchFromCursor => "search_from_cursor",
            UIElementName::SearchBackward => "search_backward",
            UIElementName::SearchRegexp => "search_regexp",
            UIElementName::HelpTable => "help_table",
        };
        str.to_string()
    }
}

impl From<UIElementName> for String {
    fn from(x: UIElementName) -> Self {
        x.to_string()
    }
}