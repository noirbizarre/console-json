<h1 align="center">console-json</h1>

<p align="center"><strong>A lightweight, terminal-aware JSON pretty printer for Rust, built on console-rs and Serde.</strong></p>

<p align="center">
  <a href="https://github.com/noirbizarre/console-json/actions/workflows/ci.yaml">
    <img src="https://github.com/noirbizarre/console-json/actions/workflows/ci.yaml/badge.svg" alt="CI">
  </a>
  <a href="https://codecov.io/gh/noirbizarre/console-json">
    <img src="https://codecov.io/gh/noirbizarre/console-json/graph/badge.svg" alt="Codecov">
  </a>
  <a href="https://crates.io/crates/console-json">
    <img src="https://img.shields.io/crates/v/console-json" alt="crates.io">
  </a>
  <img src="https://img.shields.io/github/v/release/noirbizarre/console-json" alt="Release">
  <img src="https://img.shields.io/github/license/noirbizarre/console-json" alt="License">
</p>

---

Readable JSON for humans, compact JSON for scripts. `console-json` streams any
Serde-serializable value to a writer, using `serde_json` for serialization and
`console` for configurable terminal styles. It preserves JSON data and escaping
without buffering or reparsing the document.

## Installation

```bash
cargo add console-json
```

## Usage

```rust
let value = serde_json::json!({"ready": true, "items": [1, 2]});
console_json::write_json_stdout(&value)?;
# Ok::<(), console_json::Error>(())
```

The stdout helper locks stdout and selects presentation for its destination:

| Destination | Layout | Colors (default) |
| --- | --- | --- |
| Terminal | Pretty, two-space indentation | When supported |
| Terminal with nonempty `NO_COLOR` | Pretty, two-space indentation | Disabled |
| Pipe or redirected stdout | Compact | Always disabled |

### Arbitrary writers and explicit colors

```rust
use console_json::{ColorMode, JsonOptions, write_json, write_json_with_options};

let value = serde_json::json!({"ready": true});
let mut plain = Vec::new();
write_json(&mut plain, &value)?;
assert_eq!(plain, serde_json::to_vec_pretty(&value)?);

let mut colored = Vec::new();
write_json_with_options(
    &mut colored,
    &value,
    JsonOptions::default().color(ColorMode::Always),
)?;
# Ok::<(), console_json::Error>(())
```

| Mode | Stdout helpers | Generic writers |
| --- | --- | --- |
| `Auto` (default) | Color-capable terminal only | Plain JSON |
| `Always` | Force styles on terminals only | Force styles |
| `Never` | Plain JSON | Plain JSON |

A generic writer always uses pretty formatting, independent of color mode.
A generic `Write` cannot reliably identify its destination, including when a
stdout handle is passed directly. Use `write_json_stdout` or
`write_json_stdout_with_options` for automatic stdout detection.

Automatic stdout colors use `console`'s detection and respect `CLICOLOR` and
nonempty `NO_COLOR`. An unset or empty `NO_COLOR` does not disable colors on an
otherwise supported terminal. For an empty value, terminal and `CLICOLOR` checks
compensate for console's presence-based detection. Even `CLICOLOR_FORCE` cannot
color a pipe or override `NO_COLOR` in `Auto`. On terminals, `Always` overrides
capability detection and `NO_COLOR`; `Never` suppresses every style. Stdout helpers
keep pipes and redirected output compact and uncolored even in `Always` mode.
Color settings never change the selected layout. The library does not change global
console settings. Console may cache capability/environment detection, so set
environment variables before the first use.

### Themes

| Token | Default style |
| --- | --- |
| Object keys (including quotes) | Cyan |
| String values (including quotes and escapes) | Green |
| Numbers | Yellow |
| Booleans | Magenta |
| `null` | Dim |
| Punctuation, separators, and indentation | Unstyled |

Customize the public `Theme` fields using `console::Style`:

```rust
use console_json::{ColorMode, JsonOptions, Theme, write_json_with_options};

let theme = Theme {
    keys: console::Style::new().blue().bold(),
    ..Theme::default()
};
write_json_with_options(
    &mut Vec::new(),
    &serde_json::json!({"ready": true}),
    JsonOptions::default().color(ColorMode::Always).theme(theme),
)?;
# Ok::<(), console_json::Error>(())
```

The selected color mode overrides individual styles' `force_styling` settings.
Removing ANSI sequences yields exactly the same text as plain output. Raw JSON
fragments supplied through Serde JSON's optional `raw_value` feature are passed
through unchanged rather than parsed or highlighted.

### Output and errors

Terminal stdout and generic writers follow `serde_json`'s two-space pretty
formatting; redirected stdout follows its compact formatting. No trailing newline
is added and the writer is not flushed; the caller controls both. Existing
`serde_json::Value` instances use the same generic functions as other
`serde::Serialize` values.

Map entries are emitted in the order supplied by `Serialize`. This crate uses
`serde_json`'s default features, so `serde_json::Value` objects have sorted keys
unless the consuming application enables `serde_json/preserve_order` itself.

Serialization and write failures return `Error::Render`, preserving the original
`serde_json::Error` (including I/O error kinds) through `thiserror` source
chaining. Inspect the source to correct an unsupported value or retry with a
writable destination. As with Serde JSON, a failure can leave partial output;
colored output may also contain an unfinished style.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT — see [LICENSE](LICENSE).
