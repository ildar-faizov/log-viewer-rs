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