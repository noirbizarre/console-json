//! Streaming JSON pretty printing with terminal-aware colors.
//!
//! Use [`write_json_stdout`] for automatic stdout terminal detection. Generic
//! writers always use pretty formatting and default to uncolored JSON because
//! [`std::io::Write`] cannot identify a TTY. Stdout helpers use pretty formatting
//! on terminals and compact, uncolored JSON for pipes and redirection.
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
//! Pretty output uses Serde JSON's two-space indentation. No output appends a
//! newline or flushes. Errors can leave partial output, including an unfinished style.
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

/// Write JSON to locked stdout with automatic terminal detection.
///
/// Any `NO_COLOR` disables automatic colors. Otherwise, nonempty `FORCE_COLOR`
/// overrides capability detection (`0` disables colors, other values enable them).
/// Without an override, colors require a supported terminal that is not dumb.
/// Terminals receive pretty JSON, even when colors are disabled. Redirected or
/// piped stdout receives compact, uncolored JSON.
///
/// # Errors
///
/// Returns [`Error::Render`] if serialization or writing fails.
pub fn write_json_stdout<T: Serialize + ?Sized>(value: &T) -> Result<()> {
    write_json_stdout_with_options(value, JsonOptions::default())
}

/// Write JSON to locked stdout with explicit color and theme options.
///
/// Terminals receive pretty JSON; pipes and redirected stdout always receive
/// compact, uncolored JSON, regardless of the selected color mode.
/// On terminals, any `NO_COLOR` disables automatic colors, including an empty value.
/// Otherwise, `FORCE_COLOR=0` disables colors and other nonempty `FORCE_COLOR`
/// values enable them, even with `TERM=dumb`. Absent or empty `FORCE_COLOR` falls
/// back to console's detection (including `CLICOLOR`), with dumb terminals disabled.
/// [`ColorMode::Always`] overrides capability detection, `NO_COLOR`, and `FORCE_COLOR`;
/// [`ColorMode::Never`] suppresses all styles without changing the layout.
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
    let is_terminal = writer.is_terminal();
    render_stdout(&mut writer, value, options, is_terminal)
}

fn render_stdout<W: Write + ?Sized, T: Serialize + ?Sized>(
    writer: &mut W,
    value: &T,
    options: JsonOptions,
    is_terminal: bool,
) -> Result<()> {
    if !is_terminal {
        // Machine consumers need compact JSON without ANSI, even with forced colors.
        serde_json::to_writer(writer, value)?;
        return Ok(());
    }
    // Explicit modes must not depend on environment detection or its cached state.
    let enabled = match options.color {
        ColorMode::Auto => options::stdout_auto_colors(is_terminal),
        ColorMode::Always => true,
        ColorMode::Never => false,
    };
    render(writer, value, options, enabled)
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

#[cfg(test)]
mod tests {
    use std::process::Command;

    use serde_json::json;

    use super::*;

    #[test]
    fn stdout_layout_and_colors_follow_destination_and_environment() {
        if let Ok(expected) = std::env::var("CONSOLE_JSON_TEST_STDOUT_COLORS") {
            assert_eq!(options::stdout_auto_colors(true), expected == "true");
            assert!(!options::stdout_auto_colors(false));
            let value = json!({"escaped\"key": ["café\n🦀", true, 42, null, {}, []]});
            for terminal in [false, true] {
                for mode in [ColorMode::Auto, ColorMode::Always, ColorMode::Never] {
                    let options = JsonOptions::default().color(mode);
                    let mut actual = Vec::new();
                    // Inject terminal status at the detection boundary so CI can
                    // exercise terminal rendering without a platform-specific PTY.
                    render_stdout(&mut actual, &value, options.clone(), terminal).unwrap();
                    let colored = terminal
                        && match mode {
                            ColorMode::Auto => expected == "true",
                            ColorMode::Always => true,
                            ColorMode::Never => false,
                        };
                    let mut expected = Vec::new();
                    if terminal {
                        render(&mut expected, &value, options, colored).unwrap();
                    } else {
                        expected = serde_json::to_vec(&value).unwrap();
                    }
                    assert_eq!(actual, expected, "terminal={terminal}, mode={mode:?}");
                    let actual = String::from_utf8(actual).unwrap();
                    assert_eq!(actual.contains('\x1b'), colored);
                    assert_eq!(
                        console::strip_ansi_codes(&actual),
                        if terminal {
                            serde_json::to_string_pretty(&value).unwrap()
                        } else {
                            serde_json::to_string(&value).unwrap()
                        }
                    );
                }
            }
            return;
        }

        // Fresh children isolate environment policy and console's capability cache.
        for no_color in [None, Some(""), Some("0"), Some("1")] {
            for force_color in [None, Some(""), Some("0"), Some("1"), Some("3")] {
                for (term, clicolor, clicolor_force, supported) in [
                    // Force console's fallback result because the child's actual stdout is a pipe.
                    ("xterm-256color", "1", "1", true),
                    ("xterm-256color", "0", "0", false),
                    // Our dumb-terminal gate must win over console's legacy force override.
                    ("dumb", "0", "1", false),
                ] {
                    let expected = no_color.is_none()
                        && match force_color {
                            Some("0") => false,
                            Some(value) if !value.is_empty() => true,
                            _ => supported,
                        };
                    let mut command = Command::new(std::env::current_exe().unwrap());
                    command
                        .args([
                            "--exact",
                            "tests::stdout_layout_and_colors_follow_destination_and_environment",
                            "--nocapture",
                        ])
                        .env("CONSOLE_JSON_TEST_STDOUT_COLORS", expected.to_string())
                        .env("TERM", term)
                        .env("CLICOLOR", clicolor)
                        .env("CLICOLOR_FORCE", clicolor_force)
                        .env_remove("FORCE_COLOR")
                        .env_remove("NO_COLOR");
                    if let Some(force_color) = force_color {
                        command.env("FORCE_COLOR", force_color);
                    }
                    if let Some(no_color) = no_color {
                        command.env("NO_COLOR", no_color);
                    }
                    let child = command.output().unwrap();
                    assert!(
                        child.status.success(),
                        "NO_COLOR={no_color:?}, FORCE_COLOR={force_color:?}, TERM={term}, CLICOLOR={clicolor}, CLICOLOR_FORCE={clicolor_force}: {}{}",
                        String::from_utf8_lossy(&child.stdout),
                        String::from_utf8_lossy(&child.stderr)
                    );
                }
            }
        }
    }

    struct Fails;

    impl Serialize for Fails {
        fn serialize<S: serde::Serializer>(&self, _: S) -> std::result::Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom(
                "deliberate serialization failure",
            ))
        }
    }

    #[test]
    fn stdout_serialization_errors_are_preserved_in_both_layouts() {
        for terminal in [false, true] {
            for mode in [ColorMode::Auto, ColorMode::Always, ColorMode::Never] {
                let error = render_stdout(
                    &mut Vec::new(),
                    &Fails,
                    JsonOptions::default().color(mode),
                    terminal,
                )
                .unwrap_err();
                let Error::Render(source) = error;
                assert!(source.is_data());
                assert_eq!(source.to_string(), "deliberate serialization failure");
                assert!(std::error::Error::source(&Error::Render(source)).is_some());
            }
        }
    }

    #[test]
    fn stdout_write_errors_retain_their_io_kind_in_both_layouts() {
        for terminal in [false, true] {
            for mode in [ColorMode::Auto, ColorMode::Always, ColorMode::Never] {
                // An exhausted slice deterministically rejects writes without
                // depending on the OS's broken-pipe signal handling.
                let mut writer: &mut [u8] = &mut [];
                let Error::Render(source) = render_stdout(
                    &mut writer,
                    &json!({"ready": true}),
                    JsonOptions::default().color(mode),
                    terminal,
                )
                .unwrap_err();
                assert_eq!(source.io_error_kind(), Some(io::ErrorKind::WriteZero));
            }
        }
    }
}
