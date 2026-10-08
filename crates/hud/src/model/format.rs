use core::fmt;

/// How a [`Console`](crate::Console) writes what it prints.
///
/// The choice comes, in this order, from [`ConsoleBuilder::format`](crate::ConsoleBuilder::format),
/// from the environment variable `HUD_FORMAT` (`rich`, `plain` or `json`, in any ASCII case; any
/// other value is ignored) and otherwise it is [`Format::Rich`]. JSON is never the default.
///
/// ```
/// use hud::{Console, Format, Table};
///
/// let table = Table::new().column("Name").row(["api"]);
/// let console = Console::builder().width(20).build();
/// let plain = console.render_as(&table, Format::Plain);
/// let json = console.render_as(&table, Format::Json);
/// assert!(!plain.contains('\x1b'));
/// assert!(json.starts_with("{\n  \"schema\": \"hud/1\""));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Format {
    /// Styled text with the escape sequences the console's capabilities allow: what a console
    /// wrote before formats existed. On a pipe that is text with none. The default.
    #[default]
    Rich,
    /// The same lines with no escape sequence at all, whatever the capabilities say. The escape
    /// character and the C1 controls inside the data are left out too.
    Plain,
    /// A JSON document, schema `hud/1`, that says what the value contains and never how it
    /// looks: no style, width or wrapping, the same bytes on every console.
    Json,
}

impl Format {
    /// Reads `rich`, `plain` or `json`, ignoring ASCII case. Anything else is `None`.
    ///
    /// ```
    /// use hud::Format;
    ///
    /// assert_eq!(Format::parse("JSON"), Some(Format::Json));
    /// assert_eq!(Format::parse("yaml"), None);
    /// ```
    pub fn parse(value: &str) -> Option<Format> {
        [Format::Rich, Format::Plain, Format::Json]
            .into_iter()
            .find(|format| value.eq_ignore_ascii_case(format.name()))
    }

    fn name(self) -> &'static str {
        match self {
            Format::Rich => "rich",
            Format::Plain => "plain",
            Format::Json => "json",
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
