use std::panic::Location;

use crate::error::CorruptedError;

/// Assert that a given error was reported
pub enum Assertion {
    ExactErrorWasReported(CorruptedError),
    ErrorMatchingPredicateWasReported(
        Box<dyn Send + Fn(&CorruptedError) -> bool>,
        &'static Location<'static>,
    ),
}

impl Assertion {
    pub fn exact_error_was_reported(error: CorruptedError) -> Self {
        Self::ExactErrorWasReported(error)
    }

    #[track_caller]
    pub fn error_matching_predicate_was_reported(
        predicate: impl Send + Fn(&CorruptedError) -> bool + 'static,
    ) -> Self {
        Self::ErrorMatchingPredicateWasReported(Box::new(predicate), Location::caller())
    }

    pub fn validate(&self, reported_errors: &[CorruptedError]) {
        match self {
            Self::ExactErrorWasReported(expected_error) => {
                if !reported_errors.contains(expected_error) {
                    panic!("Expected error {:?} was not reported", expected_error)
                }
            }
            Self::ErrorMatchingPredicateWasReported(predicate, location) => {
                if !reported_errors.iter().any(|error| predicate(error)) {
                    panic!("No error matching predicate was reported, location: {location:?}",)
                }
            }
        }
    }
}
