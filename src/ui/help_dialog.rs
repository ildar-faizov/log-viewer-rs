use crate::model::help_model::{ActionDescription, HelpModel};
use crate::ui::ui_elements::UIElementName;
use crate::ui::with_root_model::WithRootModel;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, EditView, LinearLayout};
use cursive::{Cursive, View};
use cursive_table_view::{TableView, TableViewItem};
use std::cmp::Ordering;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Hotkeys,
    Description,
}

impl TableViewItem<BasicColumn> for ActionDescription {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Hotkeys => self.hotkeys.clone(),
            BasicColumn::Description => self.description.clone(),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Hotkeys => self.hotkeys.cmp(&other.hotkeys),
            BasicColumn::Description => self.description.cmp(&other.description),
        }
    }
}

type ActionTable = TableView<ActionDescription, BasicColumn>;

pub struct HelpDialog {}

impl HelpDialog {
    pub fn build(model: &mut HelpModel) -> Box<dyn View> {
        let filter_box = EditView::new()
            .content(model.get_filter())
            .on_edit(|app, value, _| {
                let root_model = app.get_root_model();
                let mut help_model = root_model.get_help_model();
                help_model.set_filter(value);
            });

        let mut action_table = ActionTable::new()
            .column(BasicColumn::Hotkeys, "Hotkeys", |c| {
                c.ordering(Ordering::Equal).width_percent(20)
            })
            .column(BasicColumn::Description, "Description", |c| {
                c.ordering(Ordering::Equal)
            });
        action_table.set_items(model.get_filtered_actions());
        action_table.set_selected_row(0); // TODO there may be no items
        let action_table = action_table
            .with_name(UIElementName::HelpTable.to_string())
            .min_size((50, 15));

        let mut content = LinearLayout::vertical();
        content.add_child(filter_box);
        content.add_child(action_table);
        let dialog = Dialog::new()
            .title("Help")
            .padding_lrtb(1, 1, 1, 1)
            .content(content)
            .button("Ok", |app| {
                app.get_root_model().get_help_model().set_open(false);
            });
        Box::new(dialog)
    }

    pub fn update(app: &mut Cursive, model: &mut HelpModel) -> Result<bool, &'static str> {
        let filtered_actions = model.get_filtered_actions();
        app.call_on_name::<ActionTable, _, ()>(&UIElementName::HelpTable.to_string(), move |t| {
            t.set_items_stable(filtered_actions);
        })
        .map(|_| true)
        .ok_or("Failed to update Action Table")
    }
}
