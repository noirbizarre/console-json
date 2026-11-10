use std::ffi::OsStr;

use console::Style;

pub(crate) fn stdout_auto_colors(is_terminal: bool) -> bool {
    let no_color = std::env::var_os("NO_COLOR");
    let supported = auto_color_support(
        console::colors_enabled(),
        no_color.as_deref(),
        console::is_dumb(),
        std::env::var_os("CLICOLOR").as_deref(),
        || is_terminal && empty_no_color_terminal_support(),
    );
    ColorMode::Auto.resolve(
        is_terminal,
        supported,
        no_color.is_some_and(|value| !value.is_empty()),
    )
}

fn auto_color_support(
    console_enabled: bool,
    no_color: Option<&OsStr>,
    is_dumb: bool,
    clicolor: Option<&OsStr>,
    platform_supported: impl FnOnce() -> bool,
) -> bool {
    // Console rejects even an empty NO_COLOR, so its false result alone cannot
    // decide this case. Recheck policy and capabilities without mutating the
    // environment or process-global console settings. Keep the capability
    // probe lazy: disabled output must not initialize native console modes.
    console_enabled
        || (no_color.is_some_and(OsStr::is_empty)
            && !is_dumb
            && clicolor != Some(OsStr::new("0"))
            && platform_supported())
}

#[cfg(not(windows))]
fn empty_no_color_terminal_support() -> bool {
    // On these platforms console's remaining policy is TERM/CLICOLOR alone.
    true
}

#[cfg(windows)]
fn empty_no_color_terminal_support() -> bool {
    use windows_sys::Win32::System::Console::{
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle, STD_OUTPUT_HANDLE,
        SetConsoleMode,
    };

    if console::Term::stdout().features().is_msys_tty() {
        // MSYS interprets ANSI itself and has no native mode to initialize.
        return true;
    }
    let mut mode = 0;
    // SAFETY: stdout's handle is borrowed, and mode is a valid writable DWORD.
    // Both APIs report invalid/non-console handles as failure. Preserve every
    // unrelated mode bit, leaving VT enabled just as console's normal probe does.
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        GetConsoleMode(handle, &mut mode) != 0
            && SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
    }
}

/// Whether JSON presentation styles should be emitted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    /// Detect stdout capabilities in stdout helpers; use plain JSON for generic writers.
    #[default]
    Auto,
    /// Force styles, including for pipes and when `NO_COLOR` is set.
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
    use std::ffi::OsStr;
    use std::process::Command;

    use super::{ColorMode, auto_color_support, stdout_auto_colors};

    #[test]
    fn only_nonempty_no_color_disables_automatic_colors_in_a_fresh_process() {
        if let Ok(expected) = std::env::var("CONSOLE_JSON_TEST_AUTO_COLOR") {
            // Supply terminal status directly so this environment regression
            // runs on CI without a PTY; console support is forced in the child.
            assert_eq!(stdout_auto_colors(true), expected == "true");
            assert!(!stdout_auto_colors(false));
            return;
        }

        // Isolate environment variables and console's cached state without
        // unsafe process-environment mutation in parallel Rust tests.
        for no_color in [None, Some(""), Some("0"), Some("1")] {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--exact",
                    "options::tests::only_nonempty_no_color_disables_automatic_colors_in_a_fresh_process",
                    "--nocapture",
                ])
                .env("CONSOLE_JSON_TEST_AUTO_COLOR", if no_color.is_none_or(str::is_empty) {
                    "true"
                } else {
                    "false"
                })
                .env("CLICOLOR_FORCE", "1")
                .env_remove("NO_COLOR");
            if let Some(value) = no_color {
                command.env("NO_COLOR", value);
            }
            let child = command.output().unwrap();
            assert!(
                child.status.success(),
                "{}",
                String::from_utf8_lossy(&child.stdout)
            );
        }
    }

    #[test]
    fn an_empty_no_color_does_not_disable_an_otherwise_supported_terminal() {
        for no_color in [None, Some(OsStr::new("")), Some(OsStr::new("1"))] {
            // Model console's presence-based check, including its empty-value
            // false negative, to ensure Auto compensates rather than trusting it.
            let supported = auto_color_support(no_color.is_none(), no_color, false, None, || true);
            let disabled = no_color.is_some_and(|value| !value.is_empty());
            assert_eq!(
                ColorMode::Auto.resolve(true, supported, disabled),
                !disabled
            );
            assert!(!ColorMode::Auto.resolve(false, supported, disabled));
        }
    }

    #[test]
    fn an_empty_no_color_still_respects_disabled_and_dumb_terminals() {
        let empty = Some(OsStr::new(""));
        assert!(!auto_color_support(false, empty, true, None, || panic!(
            "disabled probe"
        )));
        assert!(!auto_color_support(
            false,
            empty,
            false,
            Some(OsStr::new("0")),
            || panic!("disabled probe")
        ));
        assert!(auto_color_support(
            false,
            empty,
            false,
            Some(OsStr::new("1")),
            || true
        ));
        // A console force override may enable support, but the destination and
        // nonempty NO_COLOR gates are still applied by ColorMode::resolve.
        assert!(auto_color_support(
            true,
            empty,
            true,
            Some(OsStr::new("0")),
            || panic!("unnecessary probe")
        ));
        assert!(!auto_color_support(false, empty, false, None, || false));
    }

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

    #[cfg(windows)]
    #[test]
    fn empty_no_color_initializes_a_private_windows_console_before_styling() {
        use std::fs::OpenOptions;
        use std::os::windows::{io::AsRawHandle, process::CommandExt};

        use windows_sys::Win32::{
            Foundation::HANDLE,
            System::{
                Console::{
                    ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle,
                    STD_OUTPUT_HANDLE, SetConsoleMode, SetStdHandle,
                },
                Threading::CREATE_NEW_CONSOLE,
            },
        };

        if let Ok(expected) = std::env::var("CONSOLE_JSON_TEST_NATIVE_COLOR") {
            // A captured child stdout is still a pipe even with CREATE_NEW_CONSOLE.
            // Bind only this isolated child's stdout to its private console.
            let output = OpenOptions::new()
                .read(true)
                .write(true)
                .open("CONOUT$")
                .expect("the Windows regression requires a private native console");
            let handle = output.as_raw_handle();
            let mut mode = 0;
            // SAFETY: the file owns a live console handle and mode is writable.
            assert_ne!(unsafe { GetConsoleMode(handle, &mut mode) }, 0);

            struct RestoreConsole {
                handle: HANDLE,
                original_stdout: HANDLE,
                mode: u32,
            }
            impl Drop for RestoreConsole {
                fn drop(&mut self) {
                    // SAFETY: the file outlives this guard; the original borrowed
                    // stdout remains owned by the child's standard-handle setup.
                    unsafe {
                        SetConsoleMode(self.handle, self.mode);
                        SetStdHandle(STD_OUTPUT_HANDLE, self.original_stdout);
                    }
                }
            }

            let _restore = RestoreConsole {
                handle,
                // SAFETY: reading the process's borrowed standard handle.
                original_stdout: unsafe { GetStdHandle(STD_OUTPUT_HANDLE) },
                mode,
            };
            // SAFETY: the live private console handle is valid for mode changes.
            unsafe {
                assert_ne!(SetStdHandle(STD_OUTPUT_HANDLE, handle), 0);
                assert_ne!(
                    SetConsoleMode(handle, mode & !ENABLE_VIRTUAL_TERMINAL_PROCESSING),
                    0
                );
            }
            let expected = expected == "true";
            assert_eq!(stdout_auto_colors(true), expected);
            let mut initialized = 0;
            // SAFETY: the console file remains open, and initialized is writable.
            assert_ne!(unsafe { GetConsoleMode(handle, &mut initialized) }, 0);
            assert_eq!(
                initialized & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0,
                expected
            );
            return;
        }

        for (no_color, clicolor, term, expected) in [
            (None, None, None, true),
            (Some(""), None, None, true),
            (Some("1"), None, None, false),
            (Some(""), Some("0"), None, false),
            (Some(""), None, Some("dumb"), false),
        ] {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args([
                    "--exact",
                    "options::tests::empty_no_color_initializes_a_private_windows_console_before_styling",
                    "--nocapture",
                ])
                .creation_flags(CREATE_NEW_CONSOLE)
                .env("CONSOLE_JSON_TEST_NATIVE_COLOR", expected.to_string())
                .env_remove("NO_COLOR")
                .env_remove("CLICOLOR")
                .env_remove("CLICOLOR_FORCE")
                .env_remove("TERM");
            for (name, value) in [
                ("NO_COLOR", no_color),
                ("CLICOLOR", clicolor),
                ("TERM", term),
            ] {
                if let Some(value) = value {
                    command.env(name, value);
                }
            }
            let child = command.output().unwrap();
            assert!(
                child.status.success(),
                "native console case {no_color:?}/{clicolor:?}/{term:?}: {}{}",
                String::from_utf8_lossy(&child.stdout),
                String::from_utf8_lossy(&child.stderr)
            );
        }
    }
}
