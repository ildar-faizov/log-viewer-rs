use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use crossbeam_channel::Sender;
use metrics::{Key, KeyName};
use metrics_util::AtomicBucket;
use metrics_util::registry::{AtomicStorage, Registry};
use num_traits::FromPrimitive;
use ordered_float::OrderedFloat;
use crate::application_metrics::{Description, MetricType};
use crate::shared::Shared;
use crate::utils::event_emitter::EventEmitter;
use super::model::ModelEvent;

const PERCENTILES: [u8; 4] = [50, 90, 99, 100];

pub struct MetricsModel {
    sender: Sender<ModelEvent>,
    registry: Option<Rc<Registry<Key, AtomicStorage>>>,
    descriptions: Option<Shared<HashMap<KeyName, Description>>>,
    is_open: bool,
}

#[derive(Debug)]
pub enum MetricsModelEvent {
    Open(bool),
}

pub type MetricsHolder = Option<(Rc<Registry<Key, AtomicStorage>>, Shared<HashMap<KeyName, Description>>)>;

#[derive(Clone, Debug)]
pub struct SingleMetrics {
    pub description: String,
    pub unit: Option<&'static str>,
    pub count: usize,
    pub p50: Option<f64>,
    pub p90: Option<f64>,
    pub p99: Option<f64>,
    pub max: Option<f64>,
}

impl MetricsModel {
    pub fn new(sender: Sender<ModelEvent>, metrics_holder: MetricsHolder) -> Self {
        let (registry, descriptions) = metrics_holder.unzip();
        Self {
            sender,
            registry,
            descriptions,
            is_open: false,
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn set_open(&mut self, is_open: bool) {
        if self.is_open != is_open {
            self.is_open = is_open;
            self.emit_event(MetricsModelEvent::Open(is_open));
        }
    }

    pub fn get_data(&self) -> Vec<SingleMetrics> {
        self.registry.as_ref().zip(self.descriptions.as_ref()).map(|(r, d)| {
            let histograms = r.get_histogram_handles();
            d.get_ref().iter().map(|(key, descr)| {
                match descr.get_metric_type() {
                    MetricType::Counter => todo!(),
                    MetricType::Gauge => todo!(),
                    MetricType::Histogram => {
                        let h = histograms.get(&Key::from_name(key.clone()));
                        SingleMetrics::from_hist(descr, h)
                    }
                }
            }).collect()
        }).unwrap_or_default()
    }

    fn emit_event(&self, evt: MetricsModelEvent) {
        self.sender.emit_event(ModelEvent::MetricsEvent(evt));
    }
}

impl SingleMetrics {
    fn from_hist(description: &Description, bucket: Option<&Arc<AtomicBucket<f64>>>) -> Self {
        let mut percentiles = HashMap::new();
        let mut count = 0;
        if let Some(bucket) = bucket {
            let data: Vec<OrderedFloat<f64>> = bucket.data().iter()
                .filter_map(|f| OrderedFloat::from_f64(*f))
                .collect();
            count = data.len();
            if !data.is_empty() {
                for p in PERCENTILES {
                    let percentile = kolmogorov_smirnov::ecdf::percentile(&data[..], p);
                    percentiles.insert(p, percentile);
                }
            }
        }

        SingleMetrics {
            description: description.get_description().to_string(),
            unit: description.get_unit().map(|unit| unit.as_canonical_label()),
            count,
            p50: percentiles.get(&PERCENTILES[0]).map(|v| v.0),
            p90: percentiles.get(&PERCENTILES[1]).map(|v| v.0),
            p99: percentiles.get(&PERCENTILES[2]).map(|v| v.0),
            max: percentiles.get(&PERCENTILES[3]).map(|v| v.0),
        }
    }
}