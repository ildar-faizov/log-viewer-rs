use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use metrics::{Counter, Gauge, Histogram, Key, KeyName, Metadata, Recorder, SharedString, Unit};
use metrics_util::registry::{AtomicStorage, Registry};
use crate::shared::Shared;

pub struct ApplicationRecorder {
    registry: Rc<Registry<Key, AtomicStorage>>,
    descriptions: Shared<HashMap<KeyName, Description>>,
}

impl ApplicationRecorder {
    pub fn new() -> Self {
        Self {
            registry: Rc::new(Registry::atomic()),
            descriptions: Shared::new(HashMap::new()),
        }
    }

    pub fn get_registry(&self) -> Rc<Registry<Key, AtomicStorage>> {
        Rc::clone(&self.registry)
    }

    pub fn get_descriptions(&self) -> Shared<HashMap<KeyName, Description>> {
        self.descriptions.clone()
    }

    fn describe_metrics(&self, metric_type: MetricType, key: KeyName, unit: Option<Unit>, description: SharedString) {
        log::info!("Registering {:?} {:?} {:?} {:?}", metric_type, &key, unit, description);
        let descriptions = &mut *self.descriptions.get_mut_ref();
        descriptions.insert(key.clone(), Description::new(metric_type, key, unit, description));
    }
}

impl Recorder for ApplicationRecorder {
    fn describe_counter(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        self.describe_metrics(MetricType::Counter, key, unit, description)
    }

    fn describe_gauge(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        self.describe_metrics(MetricType::Gauge, key, unit, description)
    }

    fn describe_histogram(&self, key: KeyName, unit: Option<Unit>, description: SharedString) {
        self.describe_metrics(MetricType::Histogram, key, unit, description)
    }

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        self.registry.get_or_create_counter(key, |v| {
            Counter::from_arc(Arc::clone(v))
        })
    }

    fn register_gauge(&self, key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        self.registry.get_or_create_gauge(key, |v| {
            Gauge::from_arc(Arc::clone(v))
        })
    }

    fn register_histogram(&self, key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        self.registry.get_or_create_histogram(key, |v| {
            Histogram::from_arc(Arc::clone(v))
        })
    }
}

#[derive(Clone)]
pub struct Description {
    metric_type: MetricType,
    key: KeyName,
    unit: Option<Unit>,
    description: SharedString,
}

impl Description {
    fn new(metric_type: MetricType, key: KeyName, unit: Option<Unit>, description: SharedString) -> Self {
        Description {
            metric_type,
            key,
            unit,
            description,
        }
    }

    pub fn get_metric_type(&self) -> MetricType {
        self.metric_type
    }

    pub fn get_key(&self) -> &KeyName {
        &self.key
    }

    pub fn get_unit(&self) -> Option<&Unit> {
        self.unit.as_ref()
    }

    pub fn get_description(&self) -> &SharedString {
        &self.description
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}