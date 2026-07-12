//! Token model for the UEB Grade-2 encoder.
//!
//! A document is parsed into a flat token stream. The document engine walks
//! it, managing inter-word mode/indicators, while delegating intra-word
//! contraction to the `ContractionEngine`.

/// §9 typeform applied to a styled character (italic/bold/underline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Typeform {
    /// §9.x italic — symbol indicator `⠨⠆`.
    Italic,
    /// §9.x bold — symbol indicator `⠘⠆`.
    Bold,
    /// §9.x bold+italic — nested bold and italic indicators.
    BoldItalic,
    /// §9.x underline — symbol indicator `⠸⠆`.
    Underline,
    /// §9.x script — symbol indicator `⠈⠆`.
    Script,
    /// §9.5 first transcriber-defined typeform — e.g. typewriter font.
    Transcriber1,
    /// §9.5 second transcriber-defined typeform — e.g. double underline.
    Transcriber2,
    /// §9.5 third transcriber-defined typeform — e.g. crossed-out text.
    Transcriber3,
    /// §9.5 fourth transcriber-defined typeform — e.g. dotted underline.
    Transcriber4,
    /// §9.6.2 fifth transcriber-defined typeform — significant small capitals.
    Transcriber5,
    /// §9.8 italic text that is also underlined — nested indicators.
    ItalicUnderline,
    /// §9.8 bold text that is also underlined — nested indicators.
    BoldUnderline,
    /// §9.8 bold-italic text that is also underlined — nested indicators.
    BoldItalicUnderline,
}

/// A parsed unit of English source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnglishToken {
    /// A maximal run of ASCII letters (original case preserved).
    Word(Vec<char>),
    /// UEB §10.13: an originally unhyphenated word split by an explicit `\n`.
    WordDivision {
        /// The logical word with the newline removed.
        chars: Vec<char>,
        /// Character index at which the braille line breaks.
        break_at: usize,
    },
    /// A maximal run of ASCII digits.
    Number(Vec<char>),
    /// A single non-letter, non-digit, non-space character.
    Symbol(char),
    /// One unit of inter-word whitespace.
    Space,
    /// UEB §10.13 Note: line-end division after an existing hyphen/dash.
    LineBreak,
    /// §9: a single styled letter (its plain base char + typeform). It carries a
    /// symbol-level typeform indicator and acts as a contraction boundary, so the
    /// surrounding plain letters contract among themselves (`𝐛right` → bold-b then
    /// the `right` groupsign).
    Styled(char, Typeform),
    /// §11 technical notation delimited by `$...$` in testcase source. The delimiters
    /// are markup only; UEB owns the enclosed expression when the surrounding context
    /// is English.
    Technical(Vec<char>),
}
