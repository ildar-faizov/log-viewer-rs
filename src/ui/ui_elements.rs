pub enum UIElementName {
    MainContent,
    StatusFile,
    StatusPosition,
    StatusHint,
    // StatusProgress,
    SearchField,
    SearchFromCursor,
    SearchBackward,
    SearchRegexp,
    GoToValue,
    GoToDateValue,
    HelpTable,
}

impl ToString for UIElementName {
    fn to_string(&self) -> String {
        let str = match self {
            UIElementName::MainContent => "main_content",
            UIElementName::StatusFile => "status_file",
            UIElementName::StatusPosition => "status_position",
            UIElementName::StatusHint => "status_hint",
            // UIElementName::StatusProgress => "status_progress",
            UIElementName::SearchField => "search_field",
            UIElementName::SearchFromCursor => "search_from_cursor",
            UIElementName::SearchBackward => "search_backward",
            UIElementName::SearchRegexp => "search_regexp",
            UIElementName::GoToValue => "go_to_value",
            UIElementName::GoToDateValue => "go_to_date_value",
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