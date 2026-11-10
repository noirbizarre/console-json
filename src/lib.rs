//! Streaming JSON pretty printing with terminal-aware colors.
//!
//! Use [`write_json_stdout`] for automatic stdout terminal detection. Generic
//! writers default to plain JSON because [`std::io::Write`] cannot identify a TTY.
//!
//! ```
//! use console_json::{ColorMode, JsonOptions, write_json, write_json_with_options};
//!
//! let value = serde_json::json!({"ready": true, "items": [1, 2]});
//! let mut plain = Vec::new();
//! write_json(&mut plain, &value)?;
//! assert_eq!(plain, serde_json::to_vec_pretty(&value)?);
//!
//! let mut colored = Vec::new();
//! write_json_with_options(
//!     &mut colored,
//!     &value,
//!     JsonOptions::default().color(ColorMode::Always),
//! )?;
//! # Ok::<(), console_json::Error>(())
//! ```
//!
//! Output uses Serde JSON's two-space indentation, without an appended newline
//! or flush. Errors can leave partial output, including an unfinished style.
//! Values are serialized directly; the document is never buffered or reparsed.

#![warn(missing_docs)]

use std::io::{self, IsTerminal, Write};

use serde::Serialize;

pub mod error;
mod formatter;
mod options;

pub use error::{Error, Result};
pub use options::{ColorMode, JsonOptions, Theme};

/// Write pretty JSON to an arbitrary writer, without automatic colors.
///
/// For automatic terminal detection, use [`write_json_stdout`].
///
/// # Errors
///
/// Returns [`Error::Render`] if serialization or writing fails.
pub fn write_json<W: Write + ?Sized, T: Serialize + ?Sized>(
    writer: &mut W,
    value: &T,
) -> Result<()> {
    write_json_with_options(writer, value, JsonOptions::default())
}

/// Write pretty JSON with a color mode and theme.
///
/// [`ColorMode::Auto`] is plain for arbitrary writers, even a stdout handle:
/// use [`write_json_stdout_with_options`] for destination-aware detection, or
/// [`ColorMode::Always`] to explicitly request colors.
///
/// # Errors
///
/// Returns [`Error::Render`] if serialization or writing fails.
pub fn write_json_with_options<W: Write + ?Sized, T: Serialize + ?Sized>(
    writer: &mut W,
    value: &T,
    options: JsonOptions,
) -> Result<()> {
    let enabled = options.color.resolve(false, false, false);
    render(writer, value, options, enabled)
}

/// Write pretty JSON to locked stdout with automatic terminal detection.
///
/// Colors require a supported terminal and no nonempty `NO_COLOR` variable.
/// Redirected or piped stdout is plain JSON.
///
/// # Errors
///
/// Returns [`Error::Render`] if serialization or writing fails.
pub fn write_json_stdout<T: Serialize + ?Sized>(value: &T) -> Result<()> {
    write_json_stdout_with_options(value, JsonOptions::default())
}

/// Write pretty JSON to locked stdout with explicit options.
///
/// Automatic detection respects terminal capabilities, `CLICOLOR`, and nonempty
/// `NO_COLOR`, and always disables colors for pipes. [`ColorMode::Always`]
/// overrides detection and `NO_COLOR`; [`ColorMode::Never`] suppresses all styles.
/// Console may cache capability and environment detection for the process.
///
/// # Errors
///
/// Returns [`Error::Render`] if serialization or writing fails.
pub fn write_json_stdout_with_options<T: Serialize + ?Sized>(
    value: &T,
    options: JsonOptions,
) -> Result<()> {
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    // Explicit modes must not depend on environment detection or its cached state.
    let enabled = match options.color {
        ColorMode::Auto => options::stdout_auto_colors(writer.is_terminal()),
        ColorMode::Always => true,
        ColorMode::Never => false,
    };
    render(&mut writer, value, options, enabled)
}

fn render<W: Write + ?Sized, T: Serialize + ?Sized>(
    writer: &mut W,
    value: &T,
    options: JsonOptions,
    enabled: bool,
) -> Result<()> {
    let formatter = formatter::JsonFormatter::new(options.theme, enabled);
    let mut serializer = serde_json::Serializer::with_formatter(writer, formatter);
    value.serialize(&mut serializer)?;
    Ok(())
}
