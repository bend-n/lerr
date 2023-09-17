#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::missing_const_for_fn,
    clippy::redundant_pub_crate,
    clippy::imprecise_flops,
    clippy::dbg_macro,
    missing_docs
)]
use anstream::adapter::strip_str;
use comat::{cwrite, cwriteln};
use config::Charset;
use std::{fmt::Write, ops::Range};
use unicode_width::UnicodeWidthStr;

pub mod config;

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

impl<S: ToString> From<(&Span, S)> for Label {
    fn from((span, m): (&Span, S)) -> Self {
        Self {
            span: span.clone(),
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
#[non_exhaustive]
pub struct Error<'s> {
    /// The message
    pub message: String,
    /// Source text
    pub source: Source<'s>,
    /// Labels we hold
    pub labels: Vec<Label>,
    /// Notes
    pub notes: Vec<Note>,
    /// The config
    pub charset: Charset,
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
            charset: Charset::unicode(),
        }
    }

    /// Sets the charset
    pub fn charset(&mut self, charset: Charset) -> &mut Self {
        self.charset = charset;
        self
    }

    /// Add a message to this error
    pub fn message(&mut self, message: impl ToString) -> &mut Self {
        self.message = message.to_string();
        self
    }

    /// Add a label to this error
    pub fn label(&mut self, label: impl Into<Label>) -> &mut Self {
        let l = label.into();
        assert!(self.source.0.len() >= l.span.end, "label must be in bounds");
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

    #[cfg(test)]
    fn monochrome(&self) -> String {
        strip_str(&self.to_string()).to_string()
    }
}

macro_rules! wrpeat {
    ($to:ident, $n:expr, $fmt:expr) => {
        for _ in 0..$n {
            write!($to, "{}", $fmt)?
        }
    };
}

impl<'s> std::fmt::Display for Error<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        cwriteln!(f, "{:reset}", self.message)?;
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
                    let mut msglen = 0;
                    for chr in strip_str(candidate.message.as_str()) {
                        msglen += UnicodeWidthStr::width(chr);
                    }
                    found.push((candidate, msglen, point));
                } else {
                    i += 1;
                }
            }
            if found.is_empty() {
                continue;
            }
            cwriteln!(
                f,
                "{bold_black}{line:width$} {} {reset}{code}",
                self.charset.column_line
            )?;
            cwrite!(
                f,
                "{space:width$} {:bold_black} {reset}",
                self.charset.column_broken_line
            )?;

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
                    cwrite!(f, "{bold_red}")?;
                    wrpeat!(f, middle, self.charset.spanning_out);
                    f.write_char(self.charset.spanning_mid)?;
                    wrpeat!(f, p - middle, self.charset.spanning_out);
                    cwrite!(f, "{reset}")?;
                    middles.push((l, middle, msglen));
                    position += about;
                    continue;
                }
                cwrite!(f, "{bold_red}")?;
                wrpeat!(f, about, self.charset.spanning);
                position += about;
                cwrite!(f, " {:reset}", l.message)?;
                position += 1 + msglen;
            }
            writeln!(f)?;
            extras(self, middles, line_span, f, width, self.charset)?;
            fn extras(
                e: &Error,
                mut unfinished: Vec<(&Label, usize, usize)>,
                line_span: Span,
                f: &mut std::fmt::Formatter<'_>,
                width: usize,
                charset: Charset,
            ) -> std::fmt::Result {
                if unfinished.is_empty() {
                    return Ok(());
                }
                cwrite!(
                    f,
                    "{:width$} {:bold_black} ",
                    " ",
                    charset.column_broken_line
                )?;
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
                        cwrite!(f, "{:bold_red} ", charset.out_extension)?;
                        position += 2;
                        i += 1;
                        continue;
                    }
                    cwrite!(f, "{:bold_red} ", charset.out_end)?;
                    position += 2;
                    cwrite!(f, "{:reset}", l.message)?;
                    position += msglen;
                    unfinished.remove(i);
                }
                writeln!(f)?;
                extras(e, unfinished, line_span, f, width, charset)
            }

            found.clear();
        }

        for note in &self.notes {
            cwriteln!(f, "{space:width$} {bold_black}>{reset} {}", note.message)?;
        }
        Ok(())
    }
}

#[test]
fn display() {
    let out = Error::new("void fn x(void) -> four {\nwierd};")
        .message("attempted to use string as type")
        .label((19..23, "what is 'four'?"))
        .note("help: change it to 4")
        .note("note: maybe python would be better for you")
        .charset(Charset::ascii())
        .monochrome();
    println!("{out}");
    assert_eq!(
        out,
        r"attempted to use string as type
0 | void fn x(void) -> four {
  :                    ^^^^ what is 'four'?
  > help: change it to 4
  > note: maybe python would be better for you
"
    );
}
#[test]
fn inline() {
    let out = Error::new("im out of this worl")
        .message("such spelling")
        .label((15..19, "forgot d"))
        .label((0..2, r#"forgot '"#))
        .charset(Charset::ascii())
        .monochrome();
    println!("{out}");
    assert_eq!(
        out,
        r"such spelling
0 | im out of this worl
  : ^^ forgot '    ^^^^ forgot d
"
    );
}

#[test]
fn outline() {
    let e = Error::new("Strin::nouveau().i_like_tests(3.14158)")
        .message("unknown method String::new")
        .label((0..5, "you probably meant String"))
        .label((7..16, "use new()"))
        .label((17..18, "caps: I"))
        .label((30..37, "your π is bad"))
        .charset(Charset::ascii())
        .monochrome();
    println!("{e}");
    assert_eq!(
        e,
        r"unknown method String::new
0 | Strin::nouveau().i_like_tests(3.14158)
  : --.--  ----.---- ^ caps: I    ^^^^^^^ your π is bad
  :   |        \ use new()
  :   \ you probably meant String
"
    );
}
