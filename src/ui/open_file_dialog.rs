use std::time::SystemTime;
use chrono::{DateTime, Local};
use cursive::{Cursive, View};
use cursive::theme::{PaletteColor, Style};
use cursive::traits::{Nameable, Resizable, Scrollable};
use cursive::view::IntoBoxedView;
use cursive::views::{Dialog, LinearLayout, SelectView, TextContent, TextView};
use human_bytes::human_bytes;
use crate::model::open_file_model::{DirEntry0, OpenFileModel, OpenFileModelEvent};
use crate::ui::with_root_model::WithRootModel;

const BREADCRUMBS_LABEL: &str = "Breadcrumbs";
const FILE_LIST: &str = "FileList";
const FILE_LIST_SIZE: (usize, usize) = (40, 20);
const FILE_INFO_SIZE: &str = "FileInfoSize";
const FILE_INFO_CREATED_AT: &str = "FileInfoCreatedAt";
const FILE_INFO_MODIFIED_AT: &str = "FileInfoModifiedAt";

const DATE_FORMAT: &str = "%d %b %y %T";

type ViewUpdateCallback = Box<dyn FnOnce(&mut Cursive)>;

pub fn build_open_file_dialog(model: &mut OpenFileModel) -> Box<dyn View> {
    let mut content = LinearLayout::vertical();
    content.add_child(build_breadcrumbs(model));
    {
        let mut panel = LinearLayout::horizontal();
        panel.add_child(build_file_selector(model));
        panel.add_child(build_file_info_panel(model));
        content.add_child(panel);
    }
    let dialog = Dialog::new()
        .title("Open file")
        .padding_lrtb(1, 1, 1, 1)
        .content(content)
        .button("Open", on_open)
        .button("Cancel", close);
    Box::new(dialog)
}

// TODO return FnOnce(&mut Cursive) -> Result<bool, &'static str>
pub fn handle_open_file_model_event(model: &mut OpenFileModel, evt: OpenFileModelEvent) -> ViewUpdateCallback {
    if !model.is_open() {
        return Box::new(|app| {});
    }
    match evt {
        OpenFileModelEvent::LocationUpdated => {
            rebuild_breadcrumbs(model)
        },
        OpenFileModelEvent::FilesUpdated => {
            let callback = populate_file_selector(model);
            Box::new(|app| {
                app.call_on_name(FILE_LIST, move |list: &mut SelectView|
                    callback(list)
                );
            })
        },
        OpenFileModelEvent::EntryInfoUpdated => {
            let callback = update_file_info_panel(model);
            Box::new(|app| {
                let elements = [FILE_INFO_SIZE, FILE_INFO_CREATED_AT, FILE_INFO_MODIFIED_AT]
                    .map(|id| app.find_name::<TextView>(id).unwrap().get_shared_content());
                callback(elements);
            })
        },
        OpenFileModelEvent::Error(err) => {
            Box::new(move |app|
                app.get_root_model().set_error(Box::new(err))
            )
        },
    }
}

fn build_breadcrumbs(model: &mut OpenFileModel) -> Box<dyn View> {
    let breadcrumbs = model.get_current_location().to_str().unwrap();
    let label = TextView::new(breadcrumbs)
        .with_name(BREADCRUMBS_LABEL);
    Box::new(label)
}

fn rebuild_breadcrumbs(model: &OpenFileModel) -> ViewUpdateCallback {
    let text = model.get_current_location().to_str().unwrap().to_string();
    Box::new(move |app| {
        app.call_on_name(BREADCRUMBS_LABEL, |element: &mut TextView|
            element.set_content(text));
    })
}

fn build_file_selector(model: &mut OpenFileModel) -> Box<dyn View> {
    let mut list : SelectView<String> = SelectView::<String>::new()
        .on_select(|app, item| {
            let root_model = app.get_root_model();
            let mut open_file_model = &mut *root_model.get_open_file_model();
            open_file_model.set_current_file(Some(item));
        })
        .on_submit(select_file);
    populate_file_selector(model)(&mut list);
    list
        .with_name(FILE_LIST)
        .scrollable()
        .fixed_size(FILE_LIST_SIZE)
        .into_boxed_view()
}

fn populate_file_selector(model: &OpenFileModel) -> Box<dyn FnOnce(&mut SelectView)> {
    let files: Vec<DirEntry0> = model.get_files()
        .iter()
        .map(Clone::clone)
        .collect();
    Box::new(move |list: &mut SelectView| {
        list.clear();
        for dir_entry in files {
            let name = dir_entry.to_string();
            let label = match dir_entry {
                DirEntry0::Up => format!("\u{2B11} {}", &name),
                DirEntry0::Folder(_) => format!("\u{1F4C1} {}", &name),
                DirEntry0::File(_) => name.clone(),
            };
            list.add_item(label, name);
        }
    })
}

fn select_file(app: &mut Cursive, item: &str) {
    let root_model = app.get_root_model();
    let mut open_file_model = &mut *root_model.get_open_file_model();
    open_file_model.set_current_file(Some(item));
    open_file_model.submit_current_file();
}

fn on_open(app: &mut Cursive) {
    let list = app.find_name::<SelectView<String>>(FILE_LIST).unwrap();
    match list.selection() {
        Some(selection) => select_file(app, selection.as_ref()),
        None => {
            let mut root_model = app.get_root_model();
            root_model.set_error(Box::new("No item selected"));
        }
    }
}

fn build_file_info_panel(model: &OpenFileModel) -> Box<dyn View> {
    let mut file_size = TextView::new("");
    let mut created_at = TextView::new("");
    let mut modified_at = TextView::new("");

    let elements = [
        file_size.get_shared_content(),
        created_at.get_shared_content(),
        modified_at.get_shared_content(),
    ];
    update_file_info_panel(model)(elements);

    let mut panel = LinearLayout::vertical();
    panel.add_child(TextView::new("Size:").style(Style::from(PaletteColor::TitleSecondary)));
    panel.add_child(file_size.with_name(FILE_INFO_SIZE));
    panel.add_child(TextView::new("Created At:").style(Style::from(PaletteColor::TitleSecondary)));
    panel.add_child(created_at.with_name(FILE_INFO_CREATED_AT));
    panel.add_child(TextView::new("Modified At:").style(Style::from(PaletteColor::TitleSecondary)));
    panel.add_child(modified_at.with_name(FILE_INFO_MODIFIED_AT));
    Box::new(panel)
}

fn update_file_info_panel(model: &OpenFileModel) -> Box<dyn FnOnce([TextContent; 3])> {
    let entry_info = model.get_entry_info().map(Clone::clone);
    Box::new(move |text_views| {
        let size = entry_info.as_ref()
            .map(|e| e.size)
            .map(|size| human_bytes(size as f64))
            .unwrap_or_default();
        text_views[0].set_content(size);

        let created_at = entry_info.as_ref()
            .and_then(|e| e.created_at)
            .map(print_date)
            .unwrap_or(String::from("-"));
        text_views[1].set_content(created_at);

        let modified_at = entry_info.as_ref()
            .and_then(|e| e.modified_at)
            .map(print_date)
            .unwrap_or(String::from("-"));
        text_views[2].set_content(modified_at);
    })
}

fn print_date(time: SystemTime) -> String {
    let dt : DateTime<Local> = time.into();
    dt.format(DATE_FORMAT).to_string()
}

fn close(app: &mut Cursive) {
    let mut root_model = &mut *app.get_root_model();
    let mut open_file_model = &mut *root_model.get_open_file_model();
    open_file_model.set_open(false);
}

// TODO: macro?
// fn get_model(app: &mut Cursive) -> &mut OpenFileModel {
//     &mut *app.get_root_model().get_open_file_model()
// }