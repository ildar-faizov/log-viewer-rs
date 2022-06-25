extern crate cursive;
extern crate clap;
extern crate log4rs;
extern crate stopwatch;

mod model;
mod ui;
mod data_source;
mod utils;
mod shared;
mod selection;
mod actions;
mod highlight;
mod test_extensions;

use cursive::{CursiveRunnable, CursiveRunner, View};
use cursive::views::{TextView, ViewRef, Canvas};

use clap::{Arg, App, ArgMatches};

use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::ui::{build_ui, UIElementName};
use crate::model::model::{ModelEvent, RootModel};
use crate::model::model::ModelEvent::*;
use cursive::direction::Direction;
use std::fs::OpenOptions;
use std::panic;
use cursive::event::Event;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log::LevelFilter;
use crate::shared::Shared;

fn main() {
	init_logging();
	init_panic_hook();

	let args = parse_args();

	let (sender, receiver) = unbounded();
	let model = create_model(args, sender);

    run_ui(receiver, model);
}

fn init_logging() {
	let file = OpenOptions::new().write(true).open("./logv.log");
	if let Ok(file) = file {
		file.set_len(0);
	}

	let logfile = FileAppender::builder()
		.encoder(Box::new(PatternEncoder::new("{d} {l} {t} - {m}{n}")))
		.build("./logv.log")
		.unwrap();

	let config = Config::builder()
		.appender(Appender::builder().build("logfile", Box::new(logfile)))
		.build(Root::builder()
			.appender("logfile")
			.build(LevelFilter::Debug)) // TODO change to Info
		.unwrap();

	log4rs::init_config(config).unwrap();

	// log::info!("=".repeat(25));
	log::info!("Logging from logv started");
}

fn init_panic_hook() {
	panic::set_hook(Box::new(|panic_info| {
		if let Some(location) = panic_info.location() {
			log::error!("panic occurred: {:?} at {} line {}:{}", panic_info, location.file(), location.line(), location.column());
		} else {
			log::error!("panic occurred: {:?}", panic_info);
		}
	}));
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
	} else {
		// TODO: sample only
		// model.set_file_name("/var/log/bootstrap.log".to_owned())
		model.set_file_name("./test.txt".to_owned())
	}
	model
}

fn run_ui(receiver: Receiver<ModelEvent>, model: RootModel) {
	// let mut model_ref = Rc::new(RefCell::new(model));
	let model_ref = Shared::new(model);

	let mut app = cursive::default().into_runner();
	app.clear_global_callbacks(Event::CtrlChar('c')); // Ctrl+C is for copy

	app.add_fullscreen_layer(build_ui(model_ref.clone()));
	app.set_user_data(model_ref);

	// cursive event loop
	app.refresh();
	while app.is_running() {
		app.step();
        let mut state_changed = false;
        for event in receiver.try_iter() {
            match handle_model_update(&mut app, event) {
                Ok(b) => state_changed = state_changed || b,
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
		DataUpdated => {
			let mut v: ViewRef<Canvas<Shared<RootModel>>> = app.find_name(&UIElementName::MainContent.to_string()).unwrap();
			v.take_focus(Direction::none());
			Ok(true)
		}
		Error(_err) => {
			// let mut v: ViewRef<TextView> = app.find_name(&UIElementName::MainContent.to_string()).unwrap();
			// v.set_content(format!("Error: {}", err));

			let mut v: ViewRef<Canvas<Shared<RootModel>>> = app.find_name(&UIElementName::MainContent.to_string()).unwrap();
			v.take_focus(Direction::none());

			Ok(true)
		},
		CursorMoved(_) => Ok(true),
		Quit => {
			app.quit();
			Ok(false)
		}
	}
}