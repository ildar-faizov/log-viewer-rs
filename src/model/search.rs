use crate::actions::action::Action;
use crate::actions::search_next::SearchNextAction;
use crate::actions::search_prev::SearchPrevAction;
use crate::background_process::background_process_handler::BackgroundProcessHandler;
use crate::background_process::background_process_registry::BackgroundProcessRegistry;
use crate::background_process::run_in_background::RunInBackground;
use crate::background_process::task_context::TaskContext;
use crate::data_source::Direction;
use crate::immediate::Immediate;
use crate::interval::Interval;
use crate::model::model::ModelEvent;
use crate::model::navigable_searcher_constructor::{
    NavigableSearcherConstructor, NavigableSearcherConstructorError,
};
use crate::search::searcher::{Occurrence, SearchError, SearchResult};
use crate::utils::event_emitter::EventEmitter;
use crate::utils::measure_l;
use anyhow::anyhow;
use crossbeam_channel::{Receiver, Sender};
use fluent_integer::Integer;
use std::rc::Rc;
use uuid::Uuid;

pub struct Search {
    id: Uuid,
    model_sender: Sender<ModelEvent>,
    occurrences: Option<Rc<Vec<Occurrence>>>,
    last_occurrence: Option<Occurrence>,
    last_request: Option<Interval<Integer>>,
    daemon_handler: BackgroundProcessHandler,
    search_request_sender: Sender<SearchRequest>,
}

pub type CurrentOccurrenceResult = Result<(Rc<Vec<Occurrence>>, Option<usize>), SearchError>;

impl Search {
    pub fn new(
        model_sender: Sender<ModelEvent>,
        constructor: NavigableSearcherConstructor,
        registry: &mut BackgroundProcessRegistry,
    ) -> Self {
        let id = Uuid::new_v4();
        let (search_request_sender, search_request_receiver) =
            crossbeam_channel::unbounded::<SearchRequest>();
        let daemon_handler = registry
            .background_process_builder::<SearchResponse, _, Result<(), DaemonError>, _>()
            .with_task(move |ctx| {
                log::info!("Search daemon started: {:?}", constructor);
                measure_l(
                    log::Level::Info,
                    format!("Search daemon {:?}", constructor).as_str(),
                    move || search_daemon(ctx, constructor, search_request_receiver, id),
                )
            })
            .with_listener(move |root_model, s| match s {
                Ok(Ok(())) => log::info!("Search finished"),
                Ok(Err(DaemonError::SearcherConstruction(e))) => {
                    root_model.set_current_search(None);
                    root_model.set_error(Box::new(e));
                },
                Err(response) => {
                    let mut current_search = root_model.get_current_search();
                    let mut err = None;
                    if let Some(search) = current_search.as_mut() {
                        let r = search.accept_search_response(response);
                        if let Err(e) = r {
                            err = Some(e);
                        }
                    }
                    drop(current_search);
                    if let Some(err) = err {
                        root_model.set_error(Box::new(err));
                    }
                }
            })
            .run();
        Search {
            id,
            model_sender,
            occurrences: None,
            last_occurrence: None,
            last_request: None,
            daemon_handler,
            search_request_sender,
        }
    }

    pub fn search(&mut self, direction: Direction) -> anyhow::Result<()> {
        let next_occurrence = self
            .occurrences
            .as_ref()
            .zip(self.index_of_last_occurrence())
            .and_then(|(occurrences, p)| match direction {
                Direction::Forward => occurrences.get(p + 1),
                Direction::Backward => {
                    if p > 0 {
                        occurrences.get(p - 1)
                    } else {
                        None
                    }
                }
            })
            .map(Clone::clone);
        if let Some(last_occurrence) = next_occurrence {
            self.last_occurrence = Some(last_occurrence.clone());
            self.model_sender
                .emit_event(ModelEvent::Search(Ok(last_occurrence)));
            Ok(())
        } else {
            self.search_request_sender
                .send(SearchRequest::Find(direction))
                .map_err(|_| anyhow!("Failed to send search request"))
        }
    }

    pub fn get_current_occurrence(
        &mut self,
        viewport: Interval<Integer>,
    ) -> Immediate<CurrentOccurrenceResult> {
        if Some(viewport) == self.last_request {
            return match &self.occurrences {
                Some(v) => {
                    Immediate::Immediate(Ok((Rc::clone(v), self.index_of_last_occurrence())))
                }
                None => Immediate::Delayed,
            };
        }

        self.occurrences = None;
        self.last_request = Some(viewport);
        self.search_request_sender.send(SearchRequest::FindAll(viewport))
            .map(|_| Immediate::Delayed)
            .unwrap_or_else(|_| Immediate::Immediate(Err(SearchError::NotFound)))
    }

    pub fn get_hint(&self) -> String {
        let next = SearchNextAction::default();
        let prev = SearchPrevAction::default();
        format!(
            "Use {}/{} for next/prev occurrence",
            next.print_hotkeys(),
            prev.print_hotkeys()
        )
    }

    fn index_of_last_occurrence(&self) -> Option<usize> {
        self.occurrences
            .as_ref()
            .zip(self.last_occurrence.as_ref())
            .and_then(|(occurrences, occurrence)| {
                occurrences.iter().position(|item| *item == *occurrence)
            })
    }

    fn accept_search_response(&mut self, response: SearchResponse) -> anyhow::Result<()> {
        if self.id != response.id {
            return Ok(());
        }
        match response.payload {
            SearchResponsePayload::Find(search_result) => {
                if let Ok(last_occurrence) = &search_result {
                    self.last_occurrence = Some(last_occurrence.clone());
                }
                self.model_sender
                    .emit_event(ModelEvent::Search(search_result));
                Ok(())
            }
            SearchResponsePayload::FindAll(viewport, data) => match data {
                Ok(occurrences) => {
                    self.last_request = Some(viewport);
                    self.occurrences = Some(Rc::new(occurrences));
                    self.model_sender.emit_event(ModelEvent::DataUpdated);
                    Ok(())
                }
                Err(e) => Err(anyhow!(format!("{:?}", e))),
            },
        }
    }
}

impl Drop for Search {
    fn drop(&mut self) {
        self.daemon_handler.interrupt();
    }
}

fn search_daemon(
    ctx: &mut TaskContext<SearchResponse, Result<(), DaemonError>>,
    constructor: NavigableSearcherConstructor,
    receiver: Receiver<SearchRequest>,
    id: Uuid,
) -> Result<(), DaemonError> {
    let t = constructor.construct_searcher();
    let mut searcher = match t {
        Ok(s) => s,
        Err(e) => return Err(DaemonError::SearcherConstruction(e)),
    };
    while !ctx.interrupted() {
        match receiver.recv() {
            Ok(req) => {
                let payload = match req {
                    SearchRequest::Find(direction) => {
                        let res = searcher.next_occurrence(direction);
                        SearchResponsePayload::Find(res)
                    }
                    SearchRequest::FindAll(range) => {
                        let res = searcher.find_all_in_range(range);
                        SearchResponsePayload::FindAll(range, res)
                    }
                };
                let response = SearchResponse::new(id, payload);
                ctx.send_message(response).expect("Failed to send response");
            }
            Err(_) => break,
        }
    }
    Ok(())
}

enum SearchRequest {
    Find(Direction),
    FindAll(Interval<Integer>),
}

struct SearchResponse {
    id: Uuid,
    payload: SearchResponsePayload,
}

impl SearchResponse {
    pub fn new(id: Uuid, payload: SearchResponsePayload) -> Self {
        SearchResponse { id, payload }
    }
}

enum SearchResponsePayload {
    Find(SearchResult),
    FindAll(Interval<Integer>, Result<Vec<Occurrence>, SearchError>),
}

enum DaemonError {
    SearcherConstruction(NavigableSearcherConstructorError),
}