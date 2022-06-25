use fluent_integer::Integer;

#[derive(Debug)]
pub enum CursorShift {
    X ( Integer ),
    Y ( Integer ),
    TokenForward,
    TokenBackward
}

impl CursorShift {

    pub fn down_by_n(n: Integer) -> Self {
        Self::Y(n)
    }

    pub fn down() -> Self {
        Self::down_by_n(1.into())
    }

    pub fn up_by_n(n: Integer) -> Self {
        Self::Y(-1 * n)
    }

    pub fn up() -> Self {
        Self::up_by_n(1.into())
    }

    pub fn left_by_n(n: Integer) -> Self {
        Self::X(-1 * n)
    }

    pub fn left() -> Self {
        Self::left_by_n(1.into())
    }

    pub fn right_by_n(n: Integer) -> Self {
        Self::X(n)
    }

    pub fn right() -> Self {
        Self::right_by_n(1.into())
    }

    pub fn token_forward() -> Self {
        Self::TokenForward
    }

    pub fn token_backward() -> Self {
        Self::TokenBackward
    }
}
