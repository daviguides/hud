use core::fmt;

/// A terminal color.
///
/// ```
/// use hud::Color;
///
/// assert_eq!(Color::RED, Color::Standard(1));
/// assert_eq!(Color::indexed(208), Color::EightBit(208));
/// assert_eq!(Color::rgb(255, 136, 0).to_string(), "#ff8800");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    /// The terminal's own default foreground or background.
    Default,
    /// One of the 16 standard colors, numbered 0 to 15.
    Standard(u8),
    /// An entry of the 256-color palette, numbered 16 to 255 (0 to 15 are [`Color::Standard`]).
    EightBit(u8),
    /// 24-bit color as red, green and blue.
    TrueColor(u8, u8, u8),
}

impl Color {
    /// Standard black.
    pub const BLACK: Color = Color::Standard(0);
    /// Standard red.
    pub const RED: Color = Color::Standard(1);
    /// Standard green.
    pub const GREEN: Color = Color::Standard(2);
    /// Standard yellow.
    pub const YELLOW: Color = Color::Standard(3);
    /// Standard blue.
    pub const BLUE: Color = Color::Standard(4);
    /// Standard magenta.
    pub const MAGENTA: Color = Color::Standard(5);
    /// Standard cyan.
    pub const CYAN: Color = Color::Standard(6);
    /// Standard white.
    pub const WHITE: Color = Color::Standard(7);

    /// A 24-bit color.
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Color {
        Color::TrueColor(red, green, blue)
    }

    /// A palette color: numbers below 16 are standard colors, the rest 256-color entries.
    pub const fn indexed(number: u8) -> Color {
        if number < 16 {
            Color::Standard(number)
        } else {
            Color::EightBit(number)
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Color::Default => f.write_str("default"),
            Color::Standard(n) | Color::EightBit(n) => write!(f, "color({n})"),
            Color::TrueColor(r, g, b) => write!(f, "#{r:02x}{g:02x}{b:02x}"),
        }
    }
}
