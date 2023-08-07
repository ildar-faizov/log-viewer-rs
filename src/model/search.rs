use std::rc::Rc;
use crossbeam_channel::Sender;
use fluent_integer::Integer;
use crate::actions::action::Action;
use crate::actions::search_next::SearchNextAction;
use crate::actions::search_prev::SearchPrevAction;
use crate::data_source::Direction;
use crate::interval::Interval;
use crate::model::model::ModelEvent;
use crate::search::navigable_searcher::NavigableSearcher;
use crate::search::searcher::{Occurrence, SearchError};
use crate::utils::event_emitter::EventEmitter;

pub struct Search {
    model_sender: Sender<ModelEvent>,
    searcher: Box<dyn NavigableSearcher>,
    occurrences: Option<Rc<Vec<Occurrence>>>,
    last_occurrence: Option<Occurrence>,
    last_request: Option<Interval<Integer>>,
}

impl Search {
    pub fn new(model_sender: Sender<ModelEvent>, searcher: Box<dyn NavigableSearcher>) -> Self {
        Search {
            model_sender,
            searcher,
            occurrences: None,
            last_occurrence: None,
            last_request: None,
        }
    }

    pub fn search(&mut self, direction: Direction) {
        let next_occurrence = self.occurrences.as_ref().zip(self.index_of_last_occurrence())
            .and_then(|(occurrences, p)| {
                match direction {
                    Direction::Forward => occurrences.get(p + 1),
                    Direction::Backward => if p > 0 { occurrences.get(p - 1) } else { None },
                }
            }).map(Clone::clone);
        let result = next_occurrence.ok_or(())
            .or_else(|_| self.searcher.next_occurrence(direction));
        if let Ok(last_occurrence) = &result {
            self.last_occurrence = Some(last_occurrence.clone());
        }
        let evt = ModelEvent::Search(result);
        self.model_sender.emit_event(evt);
    }

    pub fn get_current_occurrence(&mut self, viewport: Interval<Integer>) -> Result<(Rc<Vec<Occurrence>>, Option<usize>), SearchError> {
        if Some(viewport) == self.last_request {
            if let Some(v) = self.occurrences.clone() {
                return Ok((v, self.index_of_last_occurrence()));
            }
        }

        self.last_request = Some(viewport);
        let vec = Rc::new(self.searcher.find_all_in_range(viewport)?);
        self.occurrences = Some(vec.clone());
        Ok((vec, self.index_of_last_occurrence()))
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
        self.occurrences.as_ref().zip(self.last_occurrence.as_ref())
            .and_then(|(occurrences, occurrence)|
                occurrences.iter().position(|item| *item == *occurrence))
    }
}