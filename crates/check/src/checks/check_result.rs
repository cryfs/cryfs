use crate::{assertion::Assertion, CorruptedError};

pub struct CheckResult {
    errors: Vec<CorruptedError>,
    assertions: Vec<Assertion>,
}

impl CheckResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            assertions: Vec::new(),
        }
    }

    pub fn add_all(&mut self, other: CheckResult) {
        self.errors.extend(other.errors);
        self.assertions.extend(other.assertions);
    }

    pub fn add_error(&mut self, error: impl Into<CorruptedError>) {
        self.errors.push(error.into());
    }

    pub fn add_assertion(&mut self, assertion: Assertion) {
        self.assertions.push(assertion);
    }

    pub fn peek_errors(&self) -> &[CorruptedError] {
        &self.errors
    }

    pub fn finalize(self) -> Vec<CorruptedError> {
        for assertion in self.assertions {
            assertion.validate(&self.errors);
        }
        self.errors
    }
}
