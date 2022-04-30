use stopwatch::Stopwatch;
use fluent_integer::Integer;

pub fn sign(n: Integer) -> (Integer, i8) {
    if n >= 0 {
        (n, 1)
    } else {
        (-n, -1)
    }
}

pub fn trim_newline(s: &mut String) -> usize {
    let mut bytes_removed = 0;
    if s.ends_with('\n') {
        s.pop();
        bytes_removed += 1;
        if s.ends_with('\r') {
            s.pop();
            bytes_removed += 1;
        }
    }
    bytes_removed
}

/// Intervals of type [a, b)
pub fn disjoint_intervals<T>(intervals: &Vec<(Integer, Integer, T)>) -> Vec<(Integer, Integer, Vec<T>)>
where T: Copy {
    let mut bounds = vec![];
    for interval in intervals {
        bounds.push(interval.0);
        bounds.push(interval.1);
    }
    bounds.sort();
    bounds.dedup();
    let mut result = vec![];
    let mut s = None;
    for bound in bounds {
        if let Some(s) = s {
            let mut ids = vec![];
            for interval in intervals {
                if interval.0 < bound && interval.1 > s {
                    ids.push(interval.2);
                }
            }
            if !ids.is_empty() {
                result.push((s, bound, ids));
            }
        }
        s = Some(bound);
    }
    result
}

pub fn measure<R, F>(descr: &str, f: F) -> R where
        F: FnOnce() -> R {
    let sw = Stopwatch::start_new();
    let result = f();
    log::trace!("{} {:?}", descr, sw.elapsed());
    result
}
