use spectral::{assert_that, Spec};
use spectral::prelude::*;

pub trait ExtendedAssertions<'s, T> {
    fn item_at(&mut self, i: usize) -> Spec<'s, T>;
}

impl <'s, T> ExtendedAssertions<'s, T> for Spec<'s, Vec<T>> {
    fn item_at(&mut self, i: usize) -> Spec<'s, T> {
        assert_that(self.subject.get(i).unwrap())
    }
}

pub trait UniqueElementAssertions<'s, T> {
    fn has_only_element(&mut self) -> Spec<'s, T>;
}

impl <'s, T> UniqueElementAssertions<'s, T> for Spec<'s, Vec<T>> {
    fn has_only_element(&mut self) -> Spec<'s, T> {
        assert_that(self.subject).has_length(1);
        assert_that(self.subject.get(0).unwrap())
    }
}