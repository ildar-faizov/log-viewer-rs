pub fn sign(n: isize) -> (usize, i8) {
    if n >= 0 {
        (n as usize, 1)
    } else {
        (-n as usize, -1)
    }
}