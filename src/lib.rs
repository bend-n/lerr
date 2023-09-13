#![doc = include_str!("../README.md")]
#![warn(
    clippy::multiple_unsafe_ops_per_block,
    clippy::missing_const_for_fn,
    clippy::redundant_pub_crate,
    clippy::missing_safety_doc,
    clippy::imprecise_flops,
    unsafe_op_in_unsafe_fn,
    clippy::dbg_macro,
    missing_docs
)]
use std::ops::Range;

use unicode_width::UnicodeWidthStr;

/// Span of bytes in the source
pub type Span = Range<usize>;
/// Label around a [`Span`]
#[derive(Debug)]
pub struct Label {
    /// The span that this label will draw at
    pub span: Span,
    /// The message this label will draw with
    pub message: String,
}

impl<S: ToString> From<(Span, S)> for Label {
    fn from((span, m): (Span, S)) -> Self {
        Self {
            span,
            message: m.to_string(),
        }
    }
}

/// A note at the end of the diagnostic
#[derive(Debug)]
pub struct Note {
    /// The note
    pub message: String,
}

/// The source text that the spans "reference"
#[derive(Debug)]
pub struct Source<'s>(&'s str);

impl<'s> Source<'s> {
    fn spans(&self) -> impl Iterator<Item = (&'s str, Span)> {
        self.0.split_inclusive('\n').scan(0, |s, x| {
            let pos = *s;
            *s += x.as_bytes().len();
            let s = x.trim_matches('\n');
            Some((s, pos..pos + s.len()))
        })
    }
}

/// The error builder that this crate is all about
#[derive(Debug)]
pub struct Error<'s> {
    /// Source text
    pub source: Source<'s>,
    /// Labels we hold
    pub labels: Vec<Label>,
    /// Notes
    pub notes: Vec<Note>,
    /// The message
    pub message: String,
}

impl<'s> Error<'s> {
    /// Create a new error with source code attached
    pub fn new(source: &'s str) -> Self {
        Self {
            labels: vec![],
            source: Source(source),
            notes: vec![],
            message: String::new(),
        }
    }

    /// Add a message to this error
    pub fn message(&mut self, message: impl ToString) -> &mut Self {
        self.message = message.to_string();
        self
    }

    /// Add a label to this error
    pub fn label(&mut self, label: impl Into<Label>) -> &mut Self {
        let l = label.into();
        if self.source.0.len() < l.span.end {
            panic!("label must be in bounds");
        }
        self.labels.push(l);
        self
    }

    /// Note something down
    pub fn note(&mut self, note: impl ToString) -> &mut Self {
        self.notes.push(Note {
            message: note.to_string(),
        });
        self
    }
}

impl<'s> std::fmt::Display for Error<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)?;
        let lines = self.source.0.lines().count();
        let width = lines.ilog10() as usize + 1;
        let space = " ";
        let mut found = vec![];
        for (line, (code, span)) in self.source.spans().enumerate() {
            for label in &self.labels {
                if span.end >= label.span.start && span.start <= label.span.start {
                    found.push(label);
                }
            }
            if found.len() > 1 {
                todo!("only one error per line supported");
            }
            let Some(&label) = found.first() else {
                continue;
            };
            writeln!(f, "[1;34;30m{line:width$} | \x1b[0m{code}")?;
            let about = UnicodeWidthStr::width(
                &self.source.0[label.span.start - span.start..label.span.end - span.start],
            );
            let padding = UnicodeWidthStr::width(&self.source.0[span.start..label.span.start]);
            write!(f, "\x1b[1;34;30m{space:width$} ¦ \x1b[0m",)?;
            for _ in 0..padding {
                write!(f, " ")?;
            }
            write!(f, "\x1b[1;34;31m")?;
            for _ in 0..about {
                write!(f, "^")?;
            }
            if label.span.end == label.span.start {
                write!(f, "^")?;
            }
            write!(f, "\x1b[0m ")?;
            writeln!(f, "{}", label.message)?;
            found.clear();
        }
        for note in &self.notes {
            writeln!(f, "{space:width$} \x1b[1;34;30m>\x1b[0m {}", note.message)?;
        }
        Ok(())
    }
}

#[test]
fn display() {
    let out = Error::new("void fn x(void) -> four {\nwierd};")
        .label((19..23, "what is 'four'?"))
        .note("\x1b[1;34;32mhelp\x1b[0m: change it to 4")
        .note("\x1b[1;34;34mnote\x1b[0m: maybe python would be better for you")
        .to_string();
    println!("{out}");
    assert_eq!(out, "\n\u{1b}[1;34;30m0 | \u{1b}[0mvoid fn x(void) -> four {\n\u{1b}[1;34;30m  ¦ \u{1b}[0m                   \u{1b}[1;34;31m^^^^\u{1b}[0m what is 'four'?\n  \u{1b}[1;34;30m>\u{1b}[0m \u{1b}[1;34;32mhelp\u{1b}[0m: change it to 4\n  \u{1b}[1;34;30m>\u{1b}[0m \u{1b}[1;34;34mnote\u{1b}[0m: maybe python would be better for you\n");
}
