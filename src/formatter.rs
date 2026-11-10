use std::io::{self, Write};

use console::Style;
use serde_json::ser::{Formatter, PrettyFormatter};

use crate::Theme;

struct TokenStyle {
    prefix: String,
    suffix: String,
}

impl TokenStyle {
    fn new(style: Style, enabled: bool) -> Self {
        // Only style metadata is buffered. Let console generate both wrappers so
        // streaming tokens need neither hand-written ANSI nor a buffered JSON string.
        let styled = style.force_styling(enabled).apply_to('\0').to_string();
        let (prefix, suffix) = styled
            .split_once('\0')
            .expect("console styles preserve the displayed marker");
        Self {
            prefix: prefix.to_owned(),
            suffix: suffix.to_owned(),
        }
    }

    fn write<W: Write + ?Sized>(
        &self,
        writer: &mut W,
        write: impl FnOnce(&mut W) -> io::Result<()>,
    ) -> io::Result<()> {
        writer.write_all(self.prefix.as_bytes())?;
        write(writer)?;
        writer.write_all(self.suffix.as_bytes())
    }
}

pub(crate) struct JsonFormatter {
    pretty: PrettyFormatter<'static>,
    keys: TokenStyle,
    strings: TokenStyle,
    numbers: TokenStyle,
    booleans: TokenStyle,
    null: TokenStyle,
    punctuation: TokenStyle,
    in_key: bool,
    in_string: bool,
}

impl JsonFormatter {
    pub(crate) fn new(theme: Theme, enabled: bool) -> Self {
        Self {
            pretty: PrettyFormatter::new(),
            keys: TokenStyle::new(theme.keys, enabled),
            strings: TokenStyle::new(theme.strings, enabled),
            numbers: TokenStyle::new(theme.numbers, enabled),
            booleans: TokenStyle::new(theme.booleans, enabled),
            null: TokenStyle::new(theme.null, enabled),
            punctuation: TokenStyle::new(theme.punctuation, enabled),
            in_key: false,
            in_string: false,
        }
    }

    fn string_style(&self) -> &TokenStyle {
        if self.in_key {
            &self.keys
        } else {
            &self.strings
        }
    }
}

// Delegate primitive encoding instead of duplicating Serde JSON's numeric rules.
macro_rules! styled_primitive {
    ($method:ident, $value:ty, $style:ident) => {
        fn $method<W: Write + ?Sized>(&mut self, writer: &mut W, value: $value) -> io::Result<()> {
            // Serde also uses primitive callbacks inside quoted map keys. Their
            // surrounding string already carries the key style.
            if self.in_string {
                return self.pretty.$method(writer, value);
            }
            self.$style
                .write(writer, |writer| self.pretty.$method(writer, value))
        }
    };
}

// PrettyFormatter owns indentation and container state, including empty containers.
macro_rules! punctuation {
    ($method:ident $(, $arg:ident: $type:ty)?) => {
        fn $method<W: Write + ?Sized>(
            &mut self,
            writer: &mut W,
            $($arg: $type,)?
        ) -> io::Result<()> {
            self.punctuation
                .write(writer, |writer| self.pretty.$method(writer $(, $arg)?))
        }
    };
}

impl Formatter for JsonFormatter {
    styled_primitive!(write_bool, bool, booleans);
    styled_primitive!(write_i8, i8, numbers);
    styled_primitive!(write_i16, i16, numbers);
    styled_primitive!(write_i32, i32, numbers);
    styled_primitive!(write_i64, i64, numbers);
    styled_primitive!(write_i128, i128, numbers);
    styled_primitive!(write_u8, u8, numbers);
    styled_primitive!(write_u16, u16, numbers);
    styled_primitive!(write_u32, u32, numbers);
    styled_primitive!(write_u64, u64, numbers);
    styled_primitive!(write_u128, u128, numbers);
    styled_primitive!(write_f32, f32, numbers);
    styled_primitive!(write_f64, f64, numbers);
    styled_primitive!(write_number_str, &str, numbers);

    fn write_null<W: Write + ?Sized>(&mut self, writer: &mut W) -> io::Result<()> {
        self.null
            .write(writer, |writer| self.pretty.write_null(writer))
    }

    fn begin_string<W: Write + ?Sized>(&mut self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self.string_style().prefix.as_bytes())?;
        self.in_string = true;
        self.pretty.begin_string(writer)
    }

    fn end_string<W: Write + ?Sized>(&mut self, writer: &mut W) -> io::Result<()> {
        self.pretty.end_string(writer)?;
        self.in_string = false;
        writer.write_all(self.string_style().suffix.as_bytes())
    }

    // The default string/escape callbacks already implement Serde JSON's rules;
    // leaving them intact also keeps a single style around the entire string.

    punctuation!(begin_array);
    punctuation!(end_array);
    punctuation!(begin_array_value, first: bool);
    punctuation!(end_array_value);
    punctuation!(begin_object);
    punctuation!(end_object);
    punctuation!(begin_object_value);
    punctuation!(end_object_value);

    fn begin_object_key<W: Write + ?Sized>(
        &mut self,
        writer: &mut W,
        first: bool,
    ) -> io::Result<()> {
        self.punctuation
            .write(writer, |writer| self.pretty.begin_object_key(writer, first))?;
        self.in_key = true;
        Ok(())
    }

    fn end_object_key<W: Write + ?Sized>(&mut self, writer: &mut W) -> io::Result<()> {
        self.in_key = false;
        self.punctuation
            .write(writer, |writer| self.pretty.end_object_key(writer))
    }
}
