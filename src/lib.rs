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
#[derive(Debug, Clone)]
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
    #[must_use = "The error doesnt print itself"]
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

macro_rules! wrpeat {
    ($to:ident, $n:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        for _ in 0..$n { write!($to, $fmt $(, $arg)*)? }
    };
}

impl<'s> std::fmt::Display for Error<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)?;
        let lines = self.source.0.lines().count();
        let width = lines.ilog10() as usize + 1;
        let space = " ";
        let mut labels = self.labels.clone();
        // label, width of message, width of ^^^
        let mut found: Vec<(Label, usize, usize)> = vec![];
        for (line, (code, line_span)) in self.source.spans().enumerate() {
            let mut i = 0;
            while i < labels.len() {
                if line_span.end >= labels[i].span.start && line_span.start <= labels[i].span.start
                {
                    let candidate = labels.swap_remove(i);

                    for (Label { span, .. }, ..) in &found {
                        if span.contains(&candidate.span.start) {
                            todo!("erorrs may not overlap")
                        }
                    }
                    // ^^^ length
                    let mut point = UnicodeWidthStr::width(
                        &self.source.0[candidate.span.start - line_span.start
                            ..candidate.span.end - line_span.start],
                    );
                    if candidate.span.end == candidate.span.start {
                        point += 1;
                    }
                    // ^^^ [<this part length>]
                    let msglen = UnicodeWidthStr::width(candidate.message.as_str());
                    found.push((candidate, msglen, point));
                } else {
                    i += 1;
                }
            }
            if found.is_empty() {
                continue;
            }
            writeln!(f, "\x1b[1;34;30m{line:width$} │ \x1b[0m{code}")?;
            write!(f, "\x1b[1;34;30m{space:width$} ¦ \x1b[0m")?;

            // sort by width
            found.sort_unstable_by(|(a, ..), (b, ..)| match a.span.start.cmp(&b.span.start) {
                core::cmp::Ordering::Equal => a.span.end.cmp(&b.span.end),
                ord => ord,
            });
            // keeps track of how many chars we have printed
            let mut position = 0;
            let mut middles = vec![];
            for (i, (l, msglen, about)) in found.iter().map(|(v, a, b)| (v, *a, *b)).enumerate() {
                let padding = UnicodeWidthStr::width(
                    &self.source.0[line_span.start + position..l.span.start],
                );
                wrpeat!(f, padding, " ");
                position += padding;

                if found
                    .iter()
                    .skip(i + 1)
                    // will this label "but into" any of the future ones if i place it here
                    .any(|(b, ..)| l.span.start + about + msglen + 1 > b.span.start)
                {
                    let p = about.saturating_sub(1);
                    let middle = (p + 1) / 2;
                    write!(f, "\x1b[1;34;31m")?;
                    wrpeat!(f, middle, "─");
                    write!(f, "┬")?;
                    wrpeat!(f, p - middle, "─");
                    write!(f, "\x1b[0m")?;
                    middles.push((l, middle, msglen));
                    position += about;
                    continue;
                }
                write!(f, "\x1b[1;34;31m")?;
                wrpeat!(f, about, "^");
                position += about;
                write!(f, "\x1b[0m ")?;
                position += 1;
                write!(f, "{}", l.message)?;
                position += msglen;
            }
            writeln!(f)?;
            extras(self, middles, line_span, f, width)?;
            fn extras(
                e: &Error,
                mut unfinished: Vec<(&Label, usize, usize)>,
                line_span: Span,
                f: &mut std::fmt::Formatter<'_>,
                width: usize,
            ) -> std::fmt::Result {
                if unfinished.is_empty() {
                    return Ok(());
                }
                write!(f, "\x1b[1;34;30m{:width$} ¦ \x1b[0m", " ")?;
                let mut position = 0;
                let mut i = 0;
                while i < unfinished.len() {
                    // connection is where we are expected to put our ╰
                    let (l, connection, msglen) = unfinished[i];

                    let padding = UnicodeWidthStr::width(
                        &e.source.0[line_span.start + position..l.span.start + connection],
                    );
                    wrpeat!(f, padding, " ");
                    position += padding;

                    if unfinished
                        .iter()
                        .skip(i + 1)
                        // will this label "but into" any of the future ones if i place it here
                        .any(|(b, ..)| l.span.start + connection + msglen + 2 > b.span.start)
                    {
                        // if it will, leave it for the next line (this is a recursive fn)
                        write!(f, "\x1b[1;34;31m│\x1b[0m ")?;
                        position += 2;
                        i += 1;
                        continue;
                    }
                    write!(f, "\x1b[1;34;31m╰\x1b[0m ")?;
                    position += 2;
                    write!(f, "{}", l.message)?;
                    position += msglen;
                    unfinished.remove(i);
                }
                writeln!(f)?;
                extras(e, unfinished, line_span, f, width)
            }

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
    assert_eq!(out, "\n\u{1b}[1;34;30m0 │ \u{1b}[0mvoid fn x(void) -> four {\n\u{1b}[1;34;30m  ¦ \u{1b}[0m                   \u{1b}[1;34;31m^^^^\u{1b}[0m what is 'four'?\n  \u{1b}[1;34;30m>\u{1b}[0m \u{1b}[1;34;32mhelp\u{1b}[0m: change it to 4\n  \u{1b}[1;34;30m>\u{1b}[0m \u{1b}[1;34;34mnote\u{1b}[0m: maybe python would be better for you\n");
}
#[test]
fn inline() {
    let out = Error::new("im out of this worl")
        .label((15..19, "forgot d"))
        .label((0..2, r#"forgot '"#))
        .to_string();
    println!("{out}");
    assert_eq!(out, "\n\u{1b}[1;34;30m0 │ \u{1b}[0mim out of this worl\n\u{1b}[1;34;30m  ¦ \u{1b}[0m\u{1b}[1;34;31m^^\u{1b}[0m forgot '    \u{1b}[1;34;31m^^^^\u{1b}[0m forgot d\n");
}
