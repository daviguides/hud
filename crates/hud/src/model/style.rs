use super::color::Color;

/// A text attribute that can be switched on or explicitly off in a [`Style`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Attribute {
    /// Bold weight.
    Bold,
    /// Dim (faint) intensity.
    Dim,
    /// Italic.
    Italic,
    /// Underline.
    Underline,
    /// Slow blink.
    Blink,
    /// Rapid blink.
    Blink2,
    /// Swap foreground and background.
    Reverse,
    /// Hidden text.
    Conceal,
    /// Strikethrough.
    Strike,
    /// Double underline.
    Underline2,
    /// Framed.
    Frame,
    /// Encircled.
    Encircle,
    /// Overline.
    Overline,
}

impl Attribute {
    /// Every attribute, in the order escape codes are emitted.
    pub(crate) const ALL: [Attribute; 13] = [
        Attribute::Bold,
        Attribute::Dim,
        Attribute::Italic,
        Attribute::Underline,
        Attribute::Blink,
        Attribute::Blink2,
        Attribute::Reverse,
        Attribute::Conceal,
        Attribute::Strike,
        Attribute::Underline2,
        Attribute::Frame,
        Attribute::Encircle,
        Attribute::Overline,
    ];

    const fn bit(self) -> u16 {
        1 << self as u16
    }

    /// The SGR parameter that switches the attribute on.
    pub(crate) const fn sgr(self) -> &'static str {
        match self {
            Attribute::Bold => "1",
            Attribute::Dim => "2",
            Attribute::Italic => "3",
            Attribute::Underline => "4",
            Attribute::Blink => "5",
            Attribute::Blink2 => "6",
            Attribute::Reverse => "7",
            Attribute::Conceal => "8",
            Attribute::Strike => "9",
            Attribute::Underline2 => "21",
            Attribute::Frame => "51",
            Attribute::Encircle => "52",
            Attribute::Overline => "53",
        }
    }

    /// The word that names the attribute in a style string.
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Attribute::Bold => "bold",
            Attribute::Dim => "dim",
            Attribute::Italic => "italic",
            Attribute::Underline => "underline",
            Attribute::Blink => "blink",
            Attribute::Blink2 => "blink2",
            Attribute::Reverse => "reverse",
            Attribute::Conceal => "conceal",
            Attribute::Strike => "strike",
            Attribute::Underline2 => "underline2",
            Attribute::Frame => "frame",
            Attribute::Encircle => "encircle",
            Attribute::Overline => "overline",
        }
    }
}

/// How text looks: colors, attributes and an optional hyperlink.
///
/// Every attribute is tri-state: untouched, on, or explicitly off (`not bold`), so a style
/// layered over another can cancel what the lower one set. Build one with the methods, or
/// parse the string form (`"bold red on blue"`).
///
/// ```
/// use hud::{Color, Style};
///
/// let built = Style::new().bold().color(Color::RED).bgcolor(Color::BLUE);
/// let parsed: Style = "bold red on blue".parse()?;
/// assert_eq!(built, parsed);
/// # Ok::<(), hud::StyleError>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Style {
    pub(crate) fg: Option<Color>,
    pub(crate) bg: Option<Color>,
    pub(crate) set: u16,
    pub(crate) on: u16,
    pub(crate) link: Option<String>,
}

impl Style {
    /// A style that changes nothing.
    pub const fn new() -> Style {
        Style {
            fg: None,
            bg: None,
            set: 0,
            on: 0,
            link: None,
        }
    }

    /// Sets the foreground color.
    #[must_use]
    pub fn color(mut self, color: Color) -> Style {
        self.fg = Some(color);
        self
    }

    /// Sets the background color.
    #[must_use]
    pub fn bgcolor(mut self, color: Color) -> Style {
        self.bg = Some(color);
        self
    }

    /// Turns an attribute on or, with `false`, explicitly off.
    #[must_use]
    pub fn attribute(mut self, attribute: Attribute, on: bool) -> Style {
        let bit = attribute.bit();
        self.set |= bit;
        if on {
            self.on |= bit;
        } else {
            self.on &= !bit;
        }
        self
    }

    /// Bold.
    #[must_use]
    pub fn bold(self) -> Style {
        self.attribute(Attribute::Bold, true)
    }

    /// Dim.
    #[must_use]
    pub fn dim(self) -> Style {
        self.attribute(Attribute::Dim, true)
    }

    /// Italic.
    #[must_use]
    pub fn italic(self) -> Style {
        self.attribute(Attribute::Italic, true)
    }

    /// Underline.
    #[must_use]
    pub fn underline(self) -> Style {
        self.attribute(Attribute::Underline, true)
    }

    /// Strikethrough.
    #[must_use]
    pub fn strike(self) -> Style {
        self.attribute(Attribute::Strike, true)
    }

    /// Reverse video.
    #[must_use]
    pub fn reverse(self) -> Style {
        self.attribute(Attribute::Reverse, true)
    }

    /// Blink.
    #[must_use]
    pub fn blink(self) -> Style {
        self.attribute(Attribute::Blink, true)
    }

    /// Makes the text a hyperlink to `url` on terminals that support it.
    #[must_use]
    pub fn link(mut self, url: impl Into<String>) -> Style {
        self.link = Some(url.into());
        self
    }

    /// Whether the style changes nothing at all.
    pub fn is_null(&self) -> bool {
        self.fg.is_none() && self.bg.is_none() && self.set == 0 && self.link.is_none()
    }

    /// The foreground color, if one is set.
    pub fn fg(&self) -> Option<Color> {
        self.fg
    }

    /// The background color, if one is set.
    pub fn bg(&self) -> Option<Color> {
        self.bg
    }

    /// The hyperlink target, if one is set.
    pub fn link_url(&self) -> Option<&str> {
        self.link.as_deref()
    }

    /// `Some(true)` when the attribute is on, `Some(false)` when explicitly off, `None` when
    /// the style does not touch it.
    pub fn get(&self, attribute: Attribute) -> Option<bool> {
        let bit = attribute.bit();
        (self.set & bit != 0).then_some(self.on & bit != 0)
    }

    /// Layers `other` over `self`: what `other` sets wins, everything else is kept.
    ///
    /// ```
    /// use hud::{Color, Style};
    ///
    /// let base = Style::new().bold().color(Color::RED);
    /// let top = Style::new().color(Color::GREEN);
    /// assert_eq!(base.combine(&top), Style::new().bold().color(Color::GREEN));
    /// ```
    #[must_use]
    pub fn combine(&self, other: &Style) -> Style {
        Style {
            fg: other.fg.or(self.fg),
            bg: other.bg.or(self.bg),
            set: self.set | other.set,
            on: (self.on & !other.set) | (other.on & other.set),
            link: other.link.clone().or_else(|| self.link.clone()),
        }
    }
}

impl core::ops::Add for Style {
    type Output = Style;

    fn add(self, other: Style) -> Style {
        self.combine(&other)
    }
}
