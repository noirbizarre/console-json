use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::Command;

use console::Style;
use console_json::{ColorMode, Error, JsonOptions, Theme, write_json, write_json_with_options};
use serde::{Serialize, Serializer};
use serde_json::json;

fn output<T: Serialize + ?Sized>(value: &T, mode: ColorMode) -> String {
    let mut bytes = Vec::new();
    write_json_with_options(&mut bytes, value, JsonOptions::default().color(mode)).unwrap();
    String::from_utf8(bytes).unwrap()
}

fn matches_serde<T: Serialize + ?Sized>(value: &T) {
    let expected = serde_json::to_string_pretty(value).unwrap();
    assert_eq!(output(value, ColorMode::Never), expected);
    assert_eq!(output(value, ColorMode::Auto), expected);
    assert_eq!(
        console::strip_ansi_codes(&output(value, ColorMode::Always)),
        expected
    );
}

#[test]
fn nested_json_preserves_escaping_unicode_structure_and_values() {
    let value = json!({
        "escaped\"key\n": ["line\n\t\r\u{0000}\u{001b}\\\"", "café 🦀", ""],
        "nested": {"integer": -42, "float": 1.25, "truth": true, "falsehood": false},
        "empty": [{}, [], null],
    });
    matches_serde(&value);
    let colored = output(&value, ColorMode::Always);
    let stripped = console::strip_ansi_codes(&colored);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stripped).unwrap(),
        value
    );
    assert!(!output(&value, ColorMode::Never).contains('\u{001b}'));
}

#[test]
fn root_values_and_all_numeric_types_match_serde_json() {
    matches_serde("Unicode 🦀 and \"escapes\"");
    matches_serde("");
    matches_serde(&());
    matches_serde(&true);
    matches_serde(&false);
    matches_serde(&json!({}));
    matches_serde(&json!([]));
    matches_serde(&(
        i8::MIN,
        i16::MIN,
        i32::MIN,
        i64::MIN,
        i128::MIN,
        u8::MAX,
        u16::MAX,
        u32::MAX,
        u64::MAX,
        u128::MAX,
    ));
    matches_serde(&(1.2345678_f32, -0.0_f32, f32::MIN, f32::MAX));
    matches_serde(&(1.2345678901234567_f64, -0.0_f64, f64::MIN, f64::MAX));
    matches_serde(&(f32::NAN, f32::INFINITY, f64::NEG_INFINITY));
}

#[test]
fn forced_colors_style_whole_tokens_and_leave_default_punctuation_plain() {
    let value = json!({"key": ["a\n\"🦀", 42, true, false, null]});
    let styled = |style: Style, text: &str| style.force_styling(true).apply_to(text).to_string();
    let expected = format!(
        "{{\n  {}: [\n    {},\n    {},\n    {},\n    {},\n    {}\n  ]\n}}",
        styled(Style::new().cyan(), "\"key\""),
        styled(Style::new().green(), "\"a\\n\\\"🦀\""),
        styled(Style::new().yellow(), "42"),
        styled(Style::new().magenta(), "true"),
        styled(Style::new().magenta(), "false"),
        styled(Style::new().dim(), "null"),
    );
    assert_eq!(output(&value, ColorMode::Always), expected);
}

#[test]
fn primitive_map_keys_keep_the_key_style_instead_of_the_value_style() {
    for value in [
        BTreeMap::from([(1_i128, "one"), (-2, "minus two")]),
        BTreeMap::from([(i128::MAX, "large")]),
    ] {
        matches_serde(&value);
        let colored = output(&value, ColorMode::Always);
        assert!(!colored.contains("\x1b[33m"));
        for key in value.keys() {
            let expected = Style::new()
                .cyan()
                .force_styling(true)
                .apply_to(format!("\"{key}\""))
                .to_string();
            assert!(colored.contains(&expected));
        }
    }
    let booleans = BTreeMap::from([(false, 0), (true, 1)]);
    matches_serde(&booleans);
    assert!(!output(&booleans, ColorMode::Always).contains("\x1b[35m"));
}

#[test]
fn custom_themes_obey_the_selected_color_mode() {
    let theme = Theme {
        keys: Style::new().red().bold().force_styling(false),
        strings: Style::new().blue().force_styling(true),
        punctuation: Style::new().underlined().force_styling(true),
        ..Theme::default()
    };
    let value = json!({"key": "value"});
    let mut plain = Vec::new();
    write_json_with_options(
        &mut plain,
        &value,
        JsonOptions::default()
            .color(ColorMode::Never)
            .theme(theme.clone()),
    )
    .unwrap();
    assert_eq!(plain, serde_json::to_vec_pretty(&value).unwrap());
    let mut colored = Vec::new();
    write_json_with_options(
        &mut colored,
        &value,
        JsonOptions::default().color(ColorMode::Always).theme(theme),
    )
    .unwrap();
    let colored = String::from_utf8(colored).unwrap();
    assert!(
        colored.contains(
            &Style::new()
                .red()
                .bold()
                .force_styling(true)
                .apply_to("\"key\"")
                .to_string()
        )
    );
    assert!(
        colored.contains(
            &Style::new()
                .blue()
                .force_styling(true)
                .apply_to("\"value\"")
                .to_string()
        )
    );
    assert!(
        colored.contains(
            &Style::new()
                .underlined()
                .force_styling(true)
                .apply_to("{")
                .to_string()
        )
    );
    assert_eq!(console::strip_ansi_codes(&colored).as_bytes(), plain);
}

struct Bytes;

impl Serialize for Bytes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&[0, 42, 255])
    }
}

#[test]
fn custom_byte_serialization_keeps_pretty_array_formatting() {
    matches_serde(&Bytes);
}

struct Fails;

impl Serialize for Fails {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(
            "deliberate serialization failure",
        ))
    }
}

#[test]
fn serialization_failures_retain_the_original_error_and_source() {
    let error = write_json(&mut Vec::new(), &Fails).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("deliberate serialization failure")
    );
    let Error::Render(source) = error else {
        panic!("expected render error")
    };
    assert!(source.is_data());
    assert!(std::error::Error::source(&Error::Render(source)).is_some());
}

#[test]
fn invalid_json_map_keys_propagate_serde_json_errors() {
    let value = BTreeMap::from([((1, 2), "invalid key")]);
    let expected = serde_json::to_vec_pretty(&value).unwrap_err();
    let Error::Render(actual) = write_json(&mut Vec::new(), &value).unwrap_err() else {
        panic!("expected render error")
    };
    assert_eq!(actual.to_string(), expected.to_string());
}

struct FailingWriter {
    remaining: usize,
}

impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "destination closed",
            ));
        }
        let written = bytes.len().min(self.remaining);
        self.remaining -= written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        panic!("rendering must not flush")
    }
}

#[test]
fn writer_errors_are_preserved_even_inside_colored_tokens() {
    for mode in [ColorMode::Never, ColorMode::Always] {
        for remaining in 0..30 {
            let error = write_json_with_options(
                &mut FailingWriter { remaining },
                &"a long string that exceeds the writer capacity",
                JsonOptions::default().color(mode),
            )
            .unwrap_err();
            let Error::Render(source) = error else {
                panic!("expected render error")
            };
            assert_eq!(source.io_error_kind(), Some(io::ErrorKind::BrokenPipe));
            assert!(source.to_string().contains("destination closed"));
        }
    }
}

struct ShortWriter(Vec<u8>);

impl Write for ShortWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let count = bytes.len().min(1);
        self.0.extend_from_slice(&bytes[..count]);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        panic!("rendering must not flush")
    }
}

#[test]
fn short_writes_and_trait_object_writers_do_not_truncate_output() {
    let value = json!({"🦀": [true, 42, null]});
    let mut writer = ShortWriter(Vec::new());
    let erased: &mut dyn Write = &mut writer;
    write_json_with_options(
        erased,
        &value,
        JsonOptions::default().color(ColorMode::Always),
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(writer.0).unwrap(),
        output(&value, ColorMode::Always)
    );
}

// Run this test alone in a child process to isolate console's cached detection
// and Rust's test-output capture without mutating the parent environment.
#[test]
fn stdout_can_be_rendered_in_an_isolated_process() {
    let Ok(mode) = std::env::var("CONSOLE_JSON_TEST_PROBE") else {
        return;
    };
    let mode = match mode.as_str() {
        "always" => ColorMode::Always,
        "never" => ColorMode::Never,
        _ => ColorMode::Auto,
    };
    println!("BEGIN_JSON");
    console_json::write_json_stdout_with_options(
        &json!({"ready": true}),
        JsonOptions::default().color(mode),
    )
    .unwrap();
    println!("\nEND_JSON");
    // Even in a color-capable child, a generic writer must remain plain.
    assert_eq!(output(&true, ColorMode::Auto), "true");
}

#[test]
fn redirected_stdout_stays_plain_unless_colors_are_explicitly_requested() {
    for mode in ["auto", "always", "never"] {
        for no_color in ["", "1"] {
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "stdout_can_be_rendered_in_an_isolated_process",
                    "--nocapture",
                ])
                .env("CONSOLE_JSON_TEST_PROBE", mode)
                .env("TERM", "xterm-256color")
                .env("CLICOLOR_FORCE", "1")
                .env("NO_COLOR", no_color)
                .output()
                .unwrap();
            assert!(
                child.status.success(),
                "{}",
                String::from_utf8_lossy(&child.stderr)
            );
            let stdout = String::from_utf8(child.stdout).unwrap();
            let rendered = stdout
                .split_once("BEGIN_JSON\n")
                .unwrap()
                .1
                .split_once("\nEND_JSON")
                .unwrap()
                .0;
            assert_eq!(rendered.contains('\x1b'), mode == "always");
            assert_eq!(
                console::strip_ansi_codes(rendered),
                "{\n  \"ready\": true\n}"
            );
        }
    }
}
