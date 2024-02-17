use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, Local};
use cursive::{Cursive, View};
use cursive::theme::{PaletteColor, Style};
use cursive::traits::{Nameable, Resizable, Scrollable};
use cursive::view::IntoBoxedView;
use cursive::views::{Button, Dialog, LinearLayout, NamedView, PaddedView, ScrollView, SelectView, TextContent, TextView};
use human_bytes::human_bytes;
use crate::model::open_file_model::{DirEntry0, OpenFileModel, OpenFileModelEvent};
use crate::ui::view_with_callback::{ViewUpdateCallback, ViewWithCallback};
use crate::ui::with_root_model::WithRootModel;

const BREADCRUMBS_PANEL: &str = "BreadcrumbsPanel";
const FILE_LIST: &str = "FileList";
const FILE_LIST_SCROLL: &str = "FileListScroll";
const FILE_LIST_SIZE: (usize, usize) = (40, 20);
const FILE_INFO_SIZE: &str = "FileInfoSize";
const FILE_INFO_CREATED_AT: &str = "FileInfoCreatedAt";
const FILE_INFO_MODIFIED_AT: &str = "FileInfoModifiedAt";
const FILE_INFO_PANEL_SIZE: (usize, usize) = (19, 10);
const PLACEHOLDER: &str = "-";
const PADDING: usize = 1;
const DIALOG_WIDTH: usize = PADDING + FILE_LIST_SIZE.0 + PADDING + FILE_INFO_PANEL_SIZE.0 + PADDING;

const DATE_FORMAT: &str = "%d %b %y %T";

type FileListScroll = ScrollView<NamedView<SelectView>>;

pub fn build_open_file_dialog(model: &mut OpenFileModel) -> ViewWithCallback {
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
        .button("Cancel", close)
        .into_boxed_view();
    let callback = Box::new(focus_file_list);
    ViewWithCallback::new(dialog, callback)
}

pub fn handle_open_file_model_event(model: &mut OpenFileModel, evt: OpenFileModelEvent) -> ViewUpdateCallback {
    if !model.is_open() {
        return Box::new(|_app| {});
    }
    match evt {
        OpenFileModelEvent::LocationUpdated => {
            let callback = rebuild_breadcrumbs(model);
            Box::new(|app| {
                app.call_on_name(BREADCRUMBS_PANEL, move |layout: &mut LinearLayout|
                    callback(layout)
                );
            })
        },
        OpenFileModelEvent::FilesUpdated => {
            let callback = populate_file_selector(model);
            Box::new(|app| {
                app.call_on_name(FILE_LIST_SCROLL, |scroll_view: &mut FileListScroll| {
                    callback(scroll_view)
                });
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
    let mut breadcrumbs_panel = LinearLayout::horizontal();
    rebuild_breadcrumbs(model)(&mut breadcrumbs_panel);
    let breadcrumbs_panel = breadcrumbs_panel
        .with_name(BREADCRUMBS_PANEL);
    let scroll_panel = ScrollView::new(breadcrumbs_panel)
        .scroll_x(true)
        .scroll_y(false)
        .max_width(DIALOG_WIDTH - 2 * PADDING);
    PaddedView::lrtb(1, 1, 0, 1, scroll_panel)
        .into_boxed_view()
}

fn rebuild_breadcrumbs(model: &OpenFileModel) -> Box<dyn FnOnce(&mut LinearLayout)> {
    let path = model.get_current_location().to_path_buf();
    Box::new(move |layout| {
        layout.clear();
        for (i, part) in path.as_path().iter().enumerate() {
            let label = part.to_string_lossy();
            let need_separator = !label.ends_with(MAIN_SEPARATOR);
            let btn = Button::new_raw(label, move |app| {
                focus_file_list(app);
                let root_model = app.get_root_model();
                let mut model = root_model.get_open_file_model();
                let mut path = PathBuf::new();
                model.get_current_location().iter().take(i + 1).for_each(|item| path.push(item));
                model.set_current_location(path);
            });
            layout.add_child(btn);
            if need_separator {
                layout.add_child(TextView::new(MAIN_SEPARATOR_STR));
            }
        }
    })
}

fn build_file_selector(model: &mut OpenFileModel) -> Box<dyn View> {
    let mut scroll_view = SelectView::<String>::new()
        .on_select(|app, item| {
            let root_model = app.get_root_model();
            let open_file_model = &mut *root_model.get_open_file_model();
            open_file_model.set_current_file(Some(item));
        })
        .on_submit(select_file)
        .with_name(FILE_LIST)
        .scrollable();
    populate_file_selector(model)(&mut scroll_view);
    let list = scroll_view
        .with_name(FILE_LIST_SCROLL)
        .fixed_size(FILE_LIST_SIZE);
    PaddedView::lrtb(1, 1, 0, 0, list)
        .into_boxed_view()
}

fn populate_file_selector(model: &OpenFileModel) -> Box<dyn FnOnce(&mut FileListScroll)> {
    let files: Vec<DirEntry0> = model.get_files()
        .iter()
        .map(Clone::clone)
        .collect();
    let file = model.get_current_file().map(String::from);
    Box::new(move |scroll_view: &mut FileListScroll| {
        let list = &mut *scroll_view.get_inner_mut().get_mut();
        list.clear();
        for (i, dir_entry) in files.iter().enumerate() {
            let name = dir_entry.to_string();
            let is_selected = file.as_ref().filter(|f| **f == name).is_some();
            let label = match dir_entry {
                DirEntry0::Up => format!("\u{2B11} {}", &name),
                DirEntry0::Folder(_) => format!("\u{1F4C1} {}", &name),
                DirEntry0::File(_) => name.clone(),
            };
            list.add_item(label, name);
            if is_selected {
                list.set_selection(i);
            }
        }
    })
}

fn select_file(app: &mut Cursive, item: &str) {
    let root_model = app.get_root_model();
    let open_file_model = &mut *root_model.get_open_file_model();
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
    let mut file_size = TextView::new(PLACEHOLDER);
    let mut created_at = TextView::new(PLACEHOLDER);
    let mut modified_at = TextView::new(PLACEHOLDER);

    let elements = [
        file_size.get_shared_content(),
        created_at.get_shared_content(),
        modified_at.get_shared_content(),
    ];
    update_file_info_panel(model)(elements);

    let panel = LinearLayout::vertical()
        .child(TextView::new("Size:").style(Style::from(PaletteColor::TitleSecondary)))
        .child(file_size.with_name(FILE_INFO_SIZE))
        .child(TextView::new("Created At:").style(Style::from(PaletteColor::TitleSecondary)))
        .child(created_at.with_name(FILE_INFO_CREATED_AT))
        .child(TextView::new("Modified At:").style(Style::from(PaletteColor::TitleSecondary)))
        .child(modified_at.with_name(FILE_INFO_MODIFIED_AT))
        .fixed_size(FILE_INFO_PANEL_SIZE);
    PaddedView::lrtb(1, 1, 0, 0, panel)
        .into_boxed_view()
}

fn update_file_info_panel(model: &OpenFileModel) -> Box<dyn FnOnce([TextContent; 3])> {
    let entry_info = model.get_entry_info().map(Clone::clone);
    Box::new(move |text_views| {
        let size = entry_info.as_ref()
            .map(|e| e.size)
            .map(|size| human_bytes(size as f64))
            .unwrap_or(String::from(PLACEHOLDER));
        text_views[0].set_content(size);

        let created_at = entry_info.as_ref()
            .and_then(|e| e.created_at)
            .map(print_date)
            .unwrap_or(String::from(PLACEHOLDER));
        text_views[1].set_content(created_at);

        let modified_at = entry_info.as_ref()
            .and_then(|e| e.modified_at)
            .map(print_date)
            .unwrap_or(String::from(PLACEHOLDER));
        text_views[2].set_content(modified_at);
    })
}

fn print_date(time: SystemTime) -> String {
    let dt : DateTime<Local> = time.into();
    dt.format(DATE_FORMAT).to_string()
}

fn close(app: &mut Cursive) {
    let root_model = &mut *app.get_root_model();
    let open_file_model = &mut *root_model.get_open_file_model();
    open_file_model.set_open(false);
}

fn focus_file_list(app: &mut Cursive) {
    app.focus_name(FILE_LIST).expect("File list cannot be focused");
}

// TODO: macro?
// fn get_model(app: &mut Cursive) -> &mut OpenFileModel {
//     &mut *app.get_root_model().get_open_file_model()
// }