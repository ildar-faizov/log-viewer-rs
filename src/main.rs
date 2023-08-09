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
mod advanced_io;
mod search;
mod interval;
mod background_process;

use cursive::{Cursive, CursiveRunnable, CursiveRunner, View};
use cursive::views::{TextView, ViewRef, Canvas, Checkbox};

use clap::{Arg, App, ArgMatches};

use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::model::model::{ModelEvent, RootModel};
use crate::model::model::ModelEvent::*;
use cursive::direction::Direction;
use std::fs::OpenOptions;
use std::panic;
use std::str::FromStr;
use cursive::event::Event;
use cursive::event::Event::Key;
use cursive::event::Key::Esc;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log::LevelFilter;
use crate::search::searcher::SearchError;
use crate::shared::Shared;

use human_bytes::human_bytes;
use crate::background_process::background_process_registry::BackgroundProcessRegistry;
use crate::ui::error_dialog::build_error_dialog;
use crate::ui::main_ui::build_ui;
use crate::ui::search_ui::build_search_ui;
use crate::ui::with_root_model::WithRootModel;
use crate::ui::ui_elements::UIElementName;

fn main() -> std::io::Result<()> {
	let args = parse_args();

	init_logging(&args)?;
	init_panic_hook();

	let (sender, receiver) = unbounded();
	let (model, background_process_registry) = create_model(&args, sender);

    run_ui(receiver, model, background_process_registry);
	Ok(())
}

fn init_logging(args: &ArgMatches) -> std::io::Result<()> {
	let file = OpenOptions::new().write(true).open("./logv.log");
	if let Ok(file) = file {
		file.set_len(0)?;
	}

	let logfile = FileAppender::builder()
		.encoder(Box::new(PatternEncoder::new("{d} {l} {t} - {m}{n}")))
		.build("./logv.log")
		.unwrap();

	let level = match args.value_of("loglevel") {
		Some(loglevel) => LevelFilter::from_str(loglevel).unwrap_or(LevelFilter::Info),
		None => LevelFilter::Info
	};

	let config = Config::builder()
		.appender(Appender::builder().build("logfile", Box::new(logfile)))
		.build(Root::builder()
			.appender("logfile")
			.build(level))
		.unwrap();

	log4rs::init_config(config).unwrap();

	// log::info!("=".repeat(25));
	log::info!("Logging from logv started. level = {}", level);

	Ok(())
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
		.arg(Arg::with_name("loglevel")
			.short("L")
			.value_name("LOGLEVEL")
			.help("Logging level")
			.default_value("info")
			.takes_value(true)
		)
		.get_matches()
}

fn create_model(args: &ArgMatches, sender: Sender<ModelEvent>) -> (Shared<RootModel>, Shared<BackgroundProcessRegistry>) {
	let background_process_registry = Shared::new(BackgroundProcessRegistry::new());
	let model = RootModel::new(sender, background_process_registry.clone());
	if let Some(file_name) = args.value_of("file") {
		model.get_mut_ref().set_file_name(file_name.to_owned());
	} else {
		// TODO: sample only
		// model.set_file_name("/var/log/bootstrap.log".to_owned())
		model.get_mut_ref().set_file_name("./test.txt".to_owned())
	}
	(model, background_process_registry)
}

fn run_ui(receiver: Receiver<ModelEvent>, model_ref: Shared<RootModel>, background_process_registry: Shared<BackgroundProcessRegistry>) {
	let mut app = cursive::default().into_runner();
	app.clear_global_callbacks(Event::CtrlChar('c')); // Ctrl+C is for copy

	app.add_global_callback(Key(Esc), |t: &mut Cursive| {
		let mut state = t.get_root_model();
		state.on_esc();
	});

	app.add_fullscreen_layer(build_ui(model_ref.clone()));
	app.set_user_data(model_ref.clone());

	// cursive event loop
	app.refresh();
	while app.is_running() {
		app.step();

		background_process_registry.get_mut_ref()
			.handle_events_from_background(model_ref.clone());

        let mut state_changed = false;
        for event in receiver.try_iter() {
            match handle_model_update(&mut app, model_ref.clone(), event) {
                Ok(b) => state_changed = state_changed || b,
                Err(err) => panic!("failed to handle model update: {}", err)
            }
        }

		if state_changed {
			app.refresh();
		}
	}
}

fn handle_model_update(app: &mut CursiveRunner<CursiveRunnable>, model: Shared<RootModel>, event: ModelEvent) -> Result<bool, &'static str> {
	match event {
		FileName(file_name, file_size) => {
			let mut v: ViewRef<TextView> = app.find_name(&UIElementName::StatusFile.to_string()).unwrap();
			v.set_content(format!("{} {}", file_name, human_bytes(file_size as f64)));
			Ok(true)
		},
		DataUpdated => {
			let mut v: ViewRef<Canvas<Shared<RootModel>>> = app.find_name(&UIElementName::MainContent.to_string()).unwrap();
			v.take_focus(Direction::none());
			Ok(true)
		},
		SearchOpen(show) => {
			if show	{
				app.add_layer(build_search_ui(model));
			} else {
				app.pop_layer();
			}
			Ok(true)
		},
		Search(result) => {
			match result {
				Ok(p) => Ok(model.get_mut_ref().move_cursor_to_offset(p.start, false)),
				Err(SearchError::NotFound) => {
					log::info!("Search finished");
					model.get_mut_ref().set_error(Box::new("Nothing found"));
					Ok(false)
				},
				Err(SearchError::IO(err)) => {
					log::error!("{}", err);
					Err("Search failed")
				},
			}
		},
		SearchFromCursor => {
			let is_from_cursor = model.get_mut_ref().get_search_model().is_from_cursor();
			app.call_on_name(&UIElementName::SearchBackward.to_string(), |chk: &mut Checkbox| {
				chk.set_enabled(is_from_cursor);
			});
			Ok(true)
		},
		Hint(hint) => {
			app.call_on_name(&UIElementName::StatusHint.to_string(), move |txt: &mut TextView| {
				txt.set_content(hint);
			});
			Ok(true)
		},
		Error(Some(err)) => {
			let error_dialog = build_error_dialog(err.as_str());
			app.add_layer(error_dialog);

			Ok(true)
		},
		Error(None) => {
			app.pop_layer();
			Ok(true)
		}
		CursorMoved(cursor_position) => {
			let mut v: ViewRef<TextView> = app.find_name(&UIElementName::StatusPosition.to_string()).unwrap();
			v.set_content(format!("L {}, C {}, O {}", cursor_position.line_no + 1, cursor_position.position_in_line + 1, cursor_position.offset));
			Ok(true)
		},
		Quit => {
			app.quit();
			Ok(false)
		}
	}
}