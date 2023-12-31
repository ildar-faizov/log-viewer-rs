extern crate cursive;
extern crate cursive_table_view;
extern crate clap;
extern crate log4rs;
extern crate stopwatch;
extern crate metrics;
extern crate metrics_util;
extern crate kolmogorov_smirnov;
extern crate ordered_float;

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
mod immediate;
mod welcome;
mod application_metrics;
mod args;

use std::collections::HashMap;
use cursive::{Cursive, CursiveRunnable, CursiveRunner, View};
use cursive::views::{TextView, ViewRef, Canvas, Checkbox};

use clap::Parser;

use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::model::model::{ModelEvent, RootModel};
use crate::model::model::ModelEvent::*;
use cursive::direction::Direction;
use std::fs::OpenOptions;
use std::panic;
use std::rc::Rc;
use std::str::FromStr;
use anyhow::anyhow;
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
use metrics::{describe_histogram, KeyName, Unit};
use metrics_util::registry::{AtomicStorage, Registry};
use crate::application_metrics::{ApplicationRecorder, Description};
use crate::args::Args;
use crate::background_process::background_process_registry::BackgroundProcessRegistry;
use crate::model::help_model::HelpModelEvent;
use crate::model::metrics_model::MetricsHolder;
use crate::ui::error_dialog::build_error_dialog;
use crate::ui::go_to_date_dialog::build_go_to_date_dialog;
use crate::ui::go_to_dialog::build_go_to_dialog;
use crate::ui::help_dialog::HelpDialog;
use crate::ui::main_ui::build_ui;
use crate::ui::metrics_dialog::handle_metrics_model_event;
use crate::ui::open_file_dialog::{build_open_file_dialog, handle_open_file_model_event};
use crate::ui::search_ui::build_search_ui;
use crate::ui::with_root_model::WithRootModel;
use crate::ui::ui_elements::UIElementName;
use crate::utils::stat;

const METRIC_APP_CYCLE: &str = "app_cycle";

fn main() -> std::io::Result<()> {
	let args = Args::parse();

	init_logging(&args)?;
	init_panic_hook();
	let metrics = init_metrics();

	let (sender, receiver) = unbounded();
	let (model, background_process_registry) = create_model(&args, sender, metrics.ok());

    run_ui(receiver, model, background_process_registry);
	Ok(())
}

fn init_logging(args: &Args) -> std::io::Result<()> {
	let file = OpenOptions::new().write(true).open("./logv.log");
	if let Ok(file) = file {
		file.set_len(0)?;
	}

	let logfile = FileAppender::builder()
		.encoder(Box::new(PatternEncoder::new("{d} {l} {t} - {m}{n}")))
		.build("./logv.log")
		.unwrap();

	let level = match args.log_level.as_ref() {
		Some(loglevel) => loglevel.clone(),
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

fn init_metrics() -> anyhow::Result<(Rc<Registry<metrics::Key, AtomicStorage>>, Shared<HashMap<KeyName, Description>>)> {
	let recorder = ApplicationRecorder::new();
	let registry = recorder.get_registry();
	let descriptions = recorder.get_descriptions();
	metrics::set_global_recorder(recorder)
		.map(move |_| {
			log::info!("Metrics recorder initialized");
			(registry, descriptions)
		})
		.map_err(|err| {
			log::error!("Failed to initialize metrics: {:?}", &err);
			anyhow!(format!("{:?}", err))
		})
}

fn create_model(args: &Args, sender: Sender<ModelEvent>, metrics_holder: MetricsHolder) -> (Shared<RootModel>, Shared<BackgroundProcessRegistry>) {
	let background_process_registry = Shared::new(BackgroundProcessRegistry::new());
	let model = RootModel::new(sender, background_process_registry.clone(), metrics_holder);
	model.get_mut_ref().set_file_name(args.file.as_ref().map(|s| s.as_str()));
	(model, background_process_registry)
}

fn run_ui(receiver: Receiver<ModelEvent>, model_ref: Shared<RootModel>, background_process_registry: Shared<BackgroundProcessRegistry>) {
	describe_histogram!(METRIC_APP_CYCLE, Unit::Milliseconds, "Application cycle");

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
		stat(METRIC_APP_CYCLE, &Unit::Milliseconds, || {
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
		})
	}
}

fn handle_model_update(app: &mut CursiveRunner<CursiveRunnable>, model: Shared<RootModel>, event: ModelEvent) -> Result<bool, &'static str> {
	match event {
		OpenFileDialog(show) => {
			if show {
				let root_model = model.get_mut_ref();
				let open_file_model = &mut *root_model.get_open_file_model();
				let dialog = build_open_file_dialog(open_file_model);
				app.add_layer(dialog.view);
				(dialog.callback)(app);
			} else {
				app.pop_layer();
			}
			Ok(true)
		},
		OpenFileModelEventWrapper(evt) => {
			let callback = {
				let root_model = model.get_mut_ref();
				let open_file_model = &mut *root_model.get_open_file_model();
				handle_open_file_model_event(open_file_model, evt)
			};
			callback(app);
			Ok(true)
		},
		OpenFile(file_name) => {
			model.get_mut_ref().set_file_name(Some(&file_name));
			Ok(true)
		},
		FileName(file_name, file_size) => {
			let mut v: ViewRef<TextView> = app.find_name(&UIElementName::StatusFile.to_string()).unwrap();
			v.set_content(format!("{} {}", file_name, human_bytes(file_size as f64)));
			Ok(true)
		},
		Repaint => Ok(true),
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
		GoToOpen(open) => {
			if open {
				app.add_layer(build_go_to_dialog(&mut model.get_mut_ref()));
			} else {
				app.pop_layer();
			}
			Ok(true)
		}
		GoToDateOpen(open) => {
			if open {
				app.add_layer(build_go_to_date_dialog(&mut model.get_mut_ref())); // TODO: handle Esc
			} else {
				app.pop_layer();
			}
			Ok(true)
		},
		HelpEvent(help_model_event) => {
			match help_model_event {
				HelpModelEvent::Show => {
					let dialog = HelpDialog::build(&mut model.get_mut_ref().get_help_model());
					app.add_layer(dialog);
					Ok(true)
				},
				HelpModelEvent::Hide => {
					app.pop_layer();
					Ok(true)
				},
				HelpModelEvent::ListUpdated => HelpDialog::update(app, &mut model.get_mut_ref().get_help_model()),
			}
		},
		MetricsEvent(evt) => {
			handle_metrics_model_event(app, evt);
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