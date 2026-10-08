use core::error::Error;

/// An error as a panel: the message, the numbered causes and a hint.
///
/// The message, the causes and the hint are plain text, never markup, so a bracket in an error
/// prints as written. Build one by hand or from any [`std::error::Error`] with
/// [`ErrorReport::from_error`], which takes the causes from its `source()` chain. Print it with
/// [`Console::print`](crate::Console::print), with `println!("{report}")`, or return it from
/// `main` through [`report`](crate::report).
///
/// ```
/// use hud::{Console, ErrorReport};
///
/// let report = ErrorReport::new("could not read config")
///     .cause("No such file or directory (os error 2)")
///     .hint("run again with --verbose for details");
/// let plain = Console::builder().width(52).plain().build().render_to_plain(&report);
/// assert_eq!(
///     plain,
///     "\
/// ╭───────────────────── Error ──────────────────────╮
/// │ could not read config                            │
/// │                                                  │
/// │ Caused by:                                       │
/// │     0: No such file or directory (os error 2)    │
/// │                                                  │
/// │ hint: run again with --verbose for details       │
/// ╰──────────────────────────────────────────────────╯
/// "
/// );
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ErrorReport {
    pub(crate) message: String,
    pub(crate) causes: Vec<String>,
    pub(crate) hint: Option<String>,
}

impl ErrorReport {
    /// A report with this message, no causes and no hint.
    pub fn new(message: impl Into<String>) -> ErrorReport {
        ErrorReport {
            message: message.into(),
            causes: Vec::new(),
            hint: None,
        }
    }

    /// A report of `error`: its `Display` is the message and every error down its `source()`
    /// chain is a cause, in order.
    ///
    /// ```
    /// use hud::ErrorReport;
    /// use std::{error::Error, fmt};
    ///
    /// #[derive(Debug)]
    /// struct Outer(std::num::ParseIntError);
    /// impl fmt::Display for Outer {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         f.write_str("bad port")
    ///     }
    /// }
    /// impl Error for Outer {
    ///     fn source(&self) -> Option<&(dyn Error + 'static)> {
    ///         Some(&self.0)
    ///     }
    /// }
    ///
    /// let err = Outer("80x".parse::<u16>().unwrap_err());
    /// assert_eq!(
    ///     ErrorReport::from_error(&err),
    ///     ErrorReport::new("bad port").cause("invalid digit found in string")
    /// );
    /// ```
    pub fn from_error(error: &(dyn Error + 'static)) -> ErrorReport {
        let mut report = ErrorReport::new(error.to_string());
        let mut source = error.source();
        while let Some(cause) = source {
            report.causes.push(cause.to_string());
            source = cause.source();
        }
        report
    }

    /// Adds a cause after the ones already in the report.
    #[must_use]
    pub fn cause(mut self, cause: impl Into<String>) -> ErrorReport {
        self.causes.push(cause.into());
        self
    }

    /// The line printed last, after `hint:`.
    #[must_use]
    pub fn hint(mut self, hint: impl Into<String>) -> ErrorReport {
        self.hint = Some(hint.into());
        self
    }
}
