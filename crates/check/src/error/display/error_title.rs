use console::style;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ErrorTitle {
    pub error_type: &'static str,
    pub error_message: &'static str,
}

impl Display for ErrorTitle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let title = format!("Error[{error_type}]", error_type = self.error_type);
        write!(
            f,
            "{title}: {error_message}\n",
            title = style(title).red().bold(),
            error_message = style(self.error_message).bold(),
        )
    }
}
