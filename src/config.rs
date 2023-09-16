//! allows configuration of how the error will appear.
/// characters used in printing the error.
#[derive(Debug, Clone, Copy)]
pub struct Charset {
    /// the line on the left
    pub column_line: char,
    /// the line on the left when theres a label there
    pub column_broken_line: char,
    /// the character shown below the error span for inline labels
    /// ```text
    /// 0 | problem
    ///     ^^^^^^^ issue
    ///     ^^^^^^^ these ones
    /// ```
    pub spanning: char,
    /// the character shown about the span, when the error is moved to a next line
    /// ```text
    /// 0 | problem
    ///     ───┬───
    ///     ^^^ ^^^ these ones
    /// ```
    pub spanning_out: char,
    /// the character shown in the middle of the span, when the error is moved to a next line, in the middle
    /// ```text
    /// 0 | problem
    ///     ───┬───
    ///        ^ this one
    /// ```
    pub spanning_mid: char,
    /// the character used to extend the label to yet another line
    /// ```text
    /// 0 | problem
    ///     ───┬───
    ///        │ < this one
    /// ```
    pub out_extension: char,
    /// the character used to end the label for a moved label
    /// ```text
    /// 0 | problem
    ///     ───┬───
    ///        ╰ issue
    ///        ^ this one
    /// ```
    pub out_end: char,
    /// the character used for a note
    /// ```text
    /// 0 | problem
    ///   > btw i must say you use the same text in the example alot
    ///   ^ this one
    /// ```
    pub note: char,
}

impl Charset {
    /// Produces a (pretty) unicode charset.
    pub const fn unicode() -> Self {
        Self {
            column_line: '|',
            column_broken_line: '¦',
            spanning: '^',
            spanning_out: '─',
            spanning_mid: '┬',
            out_extension: '│', // not a pipe btw
            out_end: '╰',
            note: '>',
        }
    }
    /// Produces a (ugly) ascii charset.
    pub const fn ascii() -> Self {
        Self {
            column_line: '|',
            column_broken_line: ':',
            spanning: '^',
            spanning_out: '-',
            spanning_mid: '.',
            out_extension: '|',
            out_end: '\\',
            note: '>',
        }
    }
}
