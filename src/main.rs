extern crate cursive;
extern crate clap;

mod model;
mod ui;

use cursive::{CursiveRunnable, CursiveRunner};
use cursive::views::{TextView, ViewRef};

use clap::{Arg, App, ArgMatches};

use model::RootModel;

use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::ui::{build_ui, UIElementName};
use crate::model::ModelEvent;
use crate::model::ModelEvent::*;

fn main() {
	let args = parse_args();

	let (sender, receiver) = unbounded();
	let model = create_model(args, sender);

    run_ui(receiver, model);
}

fn parse_args<'a>() -> ArgMatches<'a> {
	App::new("Log Viewer")
		.version("0.1")
		.author("Ildar Faizov")
		.about("Log viewer")
		.arg(Arg::with_name("file")
			.short("f")
			.value_name("FILE")
			.help("Log file")
			.takes_value(true)
		)
		.get_matches()
}

fn create_model(args: ArgMatches, sender: Sender<ModelEvent>) -> RootModel {
	let mut model = RootModel::new(sender);
	if let Some(file_name) = args.value_of("file") {
		model.set_file_name(file_name.to_owned());
	}
	model
}

fn run_ui(receiver: Receiver<ModelEvent>, model: RootModel) {
	let mut app = cursive::default().into_runner();

	app.add_fullscreen_layer(build_ui());
	app.set_user_data(model);

	// cursive event loop
	app.refresh();
	while app.is_running() {
		app.step();
        let mut state_changed = false;
        for event in receiver.try_iter() {
            match handle_model_update(&mut app, event) {
                Ok(b) => state_changed = b,
                Err(err) => panic!("failed to handle model update: {}", err)
            }
        }

		if state_changed {
			app.refresh();
		}
	}
}

fn handle_model_update(app: &mut CursiveRunner<CursiveRunnable>, event: ModelEvent) -> Result<bool, &'static str> {
	match event {
		FileName(file_name) => {
			let mut v: ViewRef<TextView> = app.find_name(&UIElementName::Status.to_string()).unwrap();
			v.set_content(file_name);
			Ok(true)
		},
		FileContent => {
			let mut v: ViewRef<TextView> = app.find_name(&UIElementName::MainContent.to_string()).unwrap();
			let model: &RootModel = app.user_data().unwrap();
			let file_content = model.file_content();
			if let Some(file_content) = file_content {
				v.set_content(file_content);
			} else {
				v.set_content("");
			}
			Ok(true)
		},
		Error(err) => {
			let mut v: ViewRef<TextView> = app.find_name(&UIElementName::MainContent.to_string()).unwrap();
			v.set_content(format!("Error: {}", err));
			Ok(true)
		}
	}
}