use crate::data_source::filtered::offset_mapper::{IOffsetMapper, OffsetEvaluationResult, OffsetMapper, OriginalOffset, ProxyOffset};
use fluent_integer::Integer;
use spectral::prelude::*;
use spectral::{AssertionFailure, Spec};

#[test]
fn test_basic_operations_2_points() {
    let mut om = OffsetMapper::default();
    assert_that!(om.add(ProxyOffset::from(0), OriginalOffset::from(10))).is_ok();
    assert_that!(om.add(ProxyOffset::from(5), OriginalOffset::from(20))).is_ok();

    assert_that!(om.eval(ProxyOffset::from(0))).is_exact_containing(10);
    assert_that!(om.eval(ProxyOffset::from(1))).is_exact_containing(11);
    assert_that!(om.eval(ProxyOffset::from(4))).is_exact_containing(14);
    assert_that!(om.eval(ProxyOffset::from(5))).is_exact_containing(20);
    assert_that!(om.eval(ProxyOffset::from(6))).is_predicted(5, 20);
    assert_that!(om.eval(ProxyOffset::from(10))).is_predicted(5, 20);
}

#[test]
fn test_basic_operations_3_point() {
    let mut om = OffsetMapper::default();
    assert_that!(om.add(ProxyOffset::from(0), OriginalOffset::from(10))).is_ok();
    assert_that!(om.add(ProxyOffset::from(5), OriginalOffset::from(20))).is_ok();
    assert_that!(om.add(ProxyOffset::from(7), OriginalOffset::from(22))).is_ok();

    assert_that!(om.eval(ProxyOffset::from(0))).is_exact_containing(10);
    assert_that!(om.eval(ProxyOffset::from(1))).is_exact_containing(11);
    assert_that!(om.eval(ProxyOffset::from(4))).is_exact_containing(14);
    assert_that!(om.eval(ProxyOffset::from(5))).is_exact_containing(20);
    assert_that!(om.eval(ProxyOffset::from(6))).is_exact_containing(21);
    assert_that!(om.eval(ProxyOffset::from(7))).is_exact_containing(22);
    assert_that!(om.eval(ProxyOffset::from(10))).is_predicted(7, 22);
}

#[test]
fn test_empty() {
    let om = OffsetMapper::default();
    assert_that!(om.eval(ProxyOffset::from(0))).is_unpredictable();
    assert_that!(om.eval(ProxyOffset::from(10))).is_unpredictable();
}

#[test]
fn test_negative_input() {
    let mut om = OffsetMapper::default();
    assert_that!(om.add(ProxyOffset::from(0), OriginalOffset::from(10))).is_ok();
    assert_that!(om.add(ProxyOffset::from(5), OriginalOffset::from(20))).is_ok();

    assert_that!(om.eval(ProxyOffset::from(-1))).is_unpredictable();
}

#[test]
fn test_negative_input_on_empty_mapper() {
    let om = OffsetMapper::default();

    assert_that!(om.eval(ProxyOffset::from(-1))).is_unpredictable();
}

#[test]
fn test_non_monotonic_add() {
    let mut om = OffsetMapper::default();
    assert_that!(om.add(ProxyOffset::from(0), OriginalOffset::from(10))).is_ok();
    assert_that!(om.add(ProxyOffset::from(5), OriginalOffset::from(20))).is_ok();

    assert_that!(om.add(ProxyOffset::from(3), OriginalOffset::from(13))).is_err();
}

#[test]
fn test_confirm() {
    let mut om = OffsetMapper::default();
    assert_that!(om.add(ProxyOffset::from(0), OriginalOffset::from(10))).is_ok();
    assert_that!(om.add(ProxyOffset::from(5), OriginalOffset::from(20))).is_ok();
    om.confirm(ProxyOffset::from(7));

    assert_that!(om.eval(ProxyOffset::from(5))).is_exact_containing(20);
    assert_that!(om.eval(ProxyOffset::from(6))).is_exact_containing(21);
    assert_that!(om.eval(ProxyOffset::from(7))).is_exact_containing(22);
}

#[test]
fn test_consistent_add() {
    let mut om = OffsetMapper::default();
    assert_that!(om.add(ProxyOffset::from(0), OriginalOffset::from(10))).is_ok();
    assert_that!(om.add(ProxyOffset::from(5), OriginalOffset::from(20))).is_ok();
    om.confirm(ProxyOffset::from(7));

    assert_that!(om.add(ProxyOffset::from(7), OriginalOffset::from(22))).is_ok();
}

trait OffsetEvaluationResultAssert {
    fn is_exact_containing<I: Into<Integer>>(&self, expected: I);

    fn is_predicted<I: Into<Integer>, J: Into<Integer>>(&self, expected_proxy: I, expected_original: J);

    fn is_unpredictable(&self);
}

impl<'a> OffsetEvaluationResultAssert for Spec<'a, OffsetEvaluationResult> {
    fn is_exact_containing<I: Into<Integer>>(&self, expected: I) {
        let subject = self.subject;
        let expected = expected.into();
        let matches = match &subject {
            OffsetEvaluationResult::Exact(e) => **e == expected,
            _ => false,
        };
        if !matches {
            AssertionFailure::from_spec(self)
                .with_expected(format!("OffsetEvaluationResult::Exact({})", expected))
                .with_actual(format!("{:?}", subject))
                .fail();
        }
    }

    fn is_predicted<I: Into<Integer>, J: Into<Integer>>(&self, expected_proxy: I, expected_original: J) {
        let subject = self.subject;
        let expected_proxy = expected_proxy.into();
        let expected_original = expected_original.into();
        let matches = match &subject {
            OffsetEvaluationResult::LastConfirmed(p, o) => **p == expected_proxy && **o == expected_original,
            _ => false,
        };
        if !matches {
            AssertionFailure::from_spec(self)
                .with_expected(format!("OffsetEvaluationResult::LastConfirmed(proxy = {:?}, original = {:?})", expected_proxy, expected_original))
                .with_actual(format!("{:?}", subject))
                .fail();
        }
    }

    fn is_unpredictable(&self) {
        let subject = self.subject;
        let matches = matches!(&subject, OffsetEvaluationResult::Unpredictable);
        if !matches {
            AssertionFailure::from_spec(self)
                .with_expected(format!("{:?}", OffsetEvaluationResult::Unpredictable))
                .with_actual(format!("{:?}", subject))
                .fail();
        }
    }
}