pub trait Highlighter<T> {
    fn process(&self, str: &str) -> Vec<Highlight<T>>;
}

pub struct Highlight<T> {
    start: usize,
    end: usize,
    payload: T
}

impl <T> Highlight<T> where T: Clone {
    pub fn new(start: usize, end: usize, payload: T) -> Self {
        Highlight {
            start, end, payload
        }
    }

    pub fn get_start(&self) -> usize {
        self.start
    }

    pub fn get_end(&self) -> usize {
        self.end
    }

    pub fn get_payload(&self) -> T {
        self.payload.clone()
    }
}