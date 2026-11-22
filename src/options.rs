use std::ffi::OsStr;

use console::Style;

pub(crate) fn stdout_auto_colors(is_terminal: bool) -> bool {
    // Destination safety and NO_COLOR take precedence over every automatic override.
    if !is_terminal || std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if let Some(force_color) = std::env::var_os("FORCE_COLOR")
        && !force_color.is_empty()
    {
        return force_color != OsStr::new("0");
    }
    // Probe lazily so explicit policy does not initialize console's cached detection.
    // A legacy CLICOLOR_FORCE must not bypass the automatic dumb-terminal gate.
    !console::is_dumb() && console::colors_enabled()
}

/// Whether JSON presentation styles should be emitted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    /// Detect stdout capabilities in stdout helpers; use plain JSON for generic writers.
    /// Any `NO_COLOR` disables automatic colors; otherwise nonempty `FORCE_COLOR`
    /// overrides capabilities (`0` disables colors, other values enable them).
    #[default]
    Auto,
    /// Force styles even when `NO_COLOR` or `FORCE_COLOR=0` is set. Stdout helpers
    /// still keep pipes and redirected output uncolored; generic writers always emit styles.
    Always,
    /// Disable every style, including styles individually forced by a custom theme.
    Never,
}

impl ColorMode {
    pub(crate) fn resolve(self, is_terminal: bool, supported: bool, no_color: bool) -> bool {
        match self {
            Self::Auto => is_terminal && supported && !no_color,
            Self::Always => true,
            Self::Never => false,
        }
    }
}

/// Token styles used while serializing JSON.
///
/// Styles never change the serialized data. The selected [`ColorMode`] overrides
/// any individual style's `force_styling` setting.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Object keys, including their quotes. Cyan by default.
    pub keys: Style,
    /// String values, including quotes and escapes. Green by default.
    pub strings: Style,
    /// Integer and floating-point values. Yellow by default.
    pub numbers: Style,
    /// Boolean values. Magenta by default.
    pub booleans: Style,
    /// Null values. Dim by default.
    pub null: Style,
    /// Container delimiters, separators, and indentation. Unstyled by default.
    pub punctuation: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            keys: Style::new().cyan(),
            strings: Style::new().green(),
            numbers: Style::new().yellow(),
            booleans: Style::new().magenta(),
            null: Style::new().dim(),
            punctuation: Style::new(),
        }
    }
}

/// Options for rendering JSON, defaulting to automatic colors and [`Theme::default`].
#[derive(Clone, Debug, Default)]
pub struct JsonOptions {
    pub(crate) color: ColorMode,
    pub(crate) theme: Theme,
}

impl JsonOptions {
    /// Choose how color support is resolved.
    #[must_use]
    pub fn color(mut self, color: ColorMode) -> Self {
        self.color = color;
        self
    }

    /// Replace the token styles.
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::ColorMode;

    #[test]
    fn automatic_colors_require_a_supported_terminal_without_no_color() {
        for terminal in [false, true] {
            for supported in [false, true] {
                for no_color in [false, true] {
                    assert_eq!(
                        ColorMode::Auto.resolve(terminal, supported, no_color),
                        terminal && supported && !no_color
                    );
                    assert!(ColorMode::Always.resolve(terminal, supported, no_color));
                    assert!(!ColorMode::Never.resolve(terminal, supported, no_color));
                }
            }
        }
    }
}
