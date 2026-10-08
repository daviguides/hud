use core::fmt;

/// A style or color string could not be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleError {
    pub(crate) message: String,
}

impl StyleError {
    pub(crate) fn new(message: impl Into<String>) -> StyleError {
        StyleError {
            message: message.into(),
        }
    }
}

impl fmt::Display for StyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for StyleError {}

/// A markup string has a closing tag with nothing to close.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkupError {
    pub(crate) message: String,
}

impl MarkupError {
    pub(crate) fn new(message: impl Into<String>) -> MarkupError {
        MarkupError {
            message: message.into(),
        }
    }
}

impl fmt::Display for MarkupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MarkupError {}

/// A JSON document could not be read as a [`Node`](crate::Node): it is not valid JSON, or it is
/// not a `hud/1` document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeError {
    pub(crate) message: String,
}

impl NodeError {
    pub(crate) fn new(message: impl Into<String>) -> NodeError {
        NodeError {
            message: message.into(),
        }
    }
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for NodeError {}
