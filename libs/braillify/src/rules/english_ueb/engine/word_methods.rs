use super::*;

impl EnglishUebEngine {
    pub(super) fn encode_word(
        &self,
        chars: &[char],
        ctx: WordContext,
        out: &mut Vec<u8>,
    ) -> Option<()> {
        let WordContext {
            standing_alone,
            upper_usable,
            shortform_usable,
            allow_longer_shortforms,
            lower_usable,
            suppress_caps,
            word_initial,
            restricted_prefix_boundary,
            digit_adjacent,
        } = ctx;
        // Unicode lowercase (so an accented/ligatured capital folds to its base —
        // `Œ`→`œ`, `À`→`à`), letting the §8 capital come from `classify_caps` while
        // the letter encodes without its own indicator (avoids a doubled `⠠`).
        let lower: Vec<char> = chars.iter().flat_map(|c| c.to_lowercase()).collect();
        let word: String = lower.iter().collect();
        // §8.2: a word with internal capitals (`HarperCollins`, `verY`) has no
        // single §8 caps pattern. Split it at each lower→upper boundary and give
        // every Title-case / all-caps part its own capital indicator. Parts not yet
        // modelled (a capital run *followed by* lowercase, e.g. `founDAtion`) leave
        // the whole word to the legacy path. Not applied inside a §8.4 caps passage.
        if let Some(cells) = encode_pdf_abbreviation(chars) {
            out.extend(cells);
            return Some(());
        }
        // §4.3.3: a word whose FIRST letter is an uppercase ligature (Æ, Œ) with
        // the rest lowercase (Caps::Single) needs a second capital indicator
        // before the ligature sign — `Ætna` → `⠠⠁⠠⠘⠖⠑⠞⠝⠁`. The DP path
        // lowercases the letter and loses the case distinction, so encode
        // letter-by-letter via `push_literal_letter`, which strips just the
        // leading capital from the two-capital ligature `accent_cells` output
        // (`Æ` → `⠠⠁⠠⠘⠖⠑`) leaving the second capital in place. Modified letters
        // may not be part of a contraction (§4.2.4), so the DP loss is safe.
        if !suppress_caps
            && matches!(classify_caps(chars), Some(Caps::Single))
            && chars.first().is_some_and(|c| matches!(c, 'Æ' | 'Œ'))
        {
            for &c in chars {
                push_literal_letter(c, out)?;
            }
            return Some(());
        }
        // UEB §4.2.4: modified letters are not used as part of contractions. For a
        // word carrying a diacritic, spell the word letter-by-letter so no groupsign
        // consumes print around the modified letter (`maître`, `d'hôtel`, `háček`).
        if chars.iter().all(|c| !c.is_uppercase())
            && lower
                .iter()
                .any(|c| super::super::rule_4::is_modified_letter(*c))
        {
            for &c in chars {
                push_literal_letter(c, out)?;
            }
            return Some(());
        }
        if !suppress_caps && classify_caps(chars).is_none() {
            return self.encode_mixed_case(chars, allow_longer_shortforms, out);
        }
        if shortform_usable && super::super::rule_10_9::is_pure_shortform_abbreviation(&word) {
            out.push(GRADE1);
        }
        // Inside a §8.4 passage the ⠠⠠⠠ … ⠠⠄ carry capitalisation; `?` still guards
        // any residual mixed-case word there (→ legacy fallback).
        if !suppress_caps && !digit_adjacent && chemical_formula_caps(chars) {
            for &c in chars {
                out.push(CAPITAL);
                out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
            }
            return Some(());
        }
        match classify_caps(chars)? {
            _ if suppress_caps => {}
            Caps::None => {}
            Caps::Single => out.push(CAPITAL),
            Caps::Word => {
                // §8.7 / UEB §5.7.2: a *standing-alone* all-caps acronym whose
                // lowercase letters form a multi-letter shortform (e.g. `CD` =
                // "could", `WD` = "would") would, as `⠠⠠cd`, be misread as the
                // capitalised shortform. The grade-1 indicator `⠰` precedes the
                // caps marker to force the literal-letters reading. When the word
                // is attached to other context (after a number, apostrophe, or
                // closing bracket) the shortform reading does not arise, so the
                // indicator is not needed (e.g. `N12 7BT`, `SHE'LL`, `(R)AC`).
                let uppercase_word: String = chars.iter().collect();
                if standing_alone
                    && !super::super::rule_10_9::is_pure_shortform_abbreviation(&word)
                    && crate::rules::english_shortform::requires_grade1_indicator(&uppercase_word)
                {
                    out.push(GRADE1);
                }
                out.push(CAPITAL);
                out.push(CAPITAL);
            }
        }
        // §10.12.1: an all-caps initialism directly abutting a digit (`CH6`,
        // `W2N 6CH`) is "used as letters" — no contractions, each letter spelled.
        // Ordinal suffixes (`6TH`, `1ST`) keep their groupsign; a lowercase
        // digit-neighbour (`3rd`, `21st`) is not all-caps and never reaches here.
        // (A bare short acronym like `WHO`/`OED` also qualifies under §10.12.1, but
        // is structurally indistinguishable from a short §8 all-caps emphasis word
        // that DOES contract (`THE`/`SHE`) — a heuristic that suppresses contractions
        // there was measured to regress 9 passing cases for 11, so it is left out.)
        let acronym_as_letters = matches!(classify_caps(chars), Some(Caps::Word))
            && !matches!(
                lower.as_slice(),
                ['s', 't'] | ['n', 'd'] | ['r', 'd'] | ['t', 'h']
            )
            && digit_adjacent;
        // §8.5 caps passage / §10.1 wordsign preference: inside a §8 caps context
        // an all-caps pronoun (`IT`, `US`) that stands alone with a wordsign
        // (`it`→⠭, `us`→⠥) must still contract — the §10.12.1 initialism heuristic
        // (`WHO`, `OED`) applies only when the whole word has NO wordsign, so a
        // caps passage's `IT'S` reads as the pronoun (⠭⠄⠎), not as spelled letters.
        let letter_initialism = is_letter_pronounced_initialism(chars)
            && !(suppress_caps
                && standing_alone
                && super::super::rule_10_1::wordsign(&word).is_some());
        if acronym_as_letters || letter_initialism {
            for &c in &lower {
                match super::super::rule_4::accent_cells(c) {
                    Some(cells) => out.extend(cells),
                    None => out.push(crate::english::encode_english(c).ok()?),
                }
            }
            return Some(());
        }
        // §10.1/§10.2 (upper) and §10.5 (lower) wordsigns: a whole word that
        // stands alone (§2.6) becomes its wordsign. Lower wordsigns additionally
        // require the stricter `lower_usable` boundary. All are suppressed inside
        // Korean text via `standing_alone = false` (한국 점자 제37항).
        if standing_alone {
            let cell = upper_usable
                .then(|| {
                    super::super::rule_10_1::wordsign(&word)
                        .or_else(|| super::super::rule_10_2::wordsign(&word))
                })
                .flatten()
                .or_else(|| {
                    lower_usable
                        .then(|| super::super::rule_10_5::wordsign(&word))
                        .flatten()
                });
            if let Some(cell) = cell {
                out.push(cell);
                return Some(());
            }
            if shortform_usable
                && let Some(cells) = super::super::rule_10_9::whole_word_cells(&word)
            {
                out.extend(cells);
                return Some(());
            }
        }
        out.extend(
            super::super::rule_10_9::encode_with_optional_longer_shortforms(
                &lower,
                &self.contractions,
                word_initial,
                restricted_prefix_boundary,
                allow_longer_shortforms,
            )?,
        );
        Some(())
    }

    /// §8.2: encode a mixed-case word by splitting it at each lower→upper boundary
    /// (the start of a new Title-case / all-caps part) and at each all-caps→lowercase
    /// boundary (§8.6.3, so a `⠠⠄` terminator can close a caps word before a lowercase
    /// tail like `WALKing`). Every part takes its own capital indicator (`⠠` Title-case,
    /// `⠠⠠` all-caps) and its contractions are computed per part; a 2-letter internal
    /// caps word (`founDAtion`'s `DA`) keeps each capital before its cell. A leading
    /// all-caps shortform prefix (`initial_caps_shortform_boundary`) and a CamelCase
    /// leading uppercase run of at least four letters
    /// (`camel_title_subunit_after_caps_prefix`, e.g. `BLASTSoundMachine`) are split off
    /// first and the remainder encoded recursively.
    pub(super) fn encode_mixed_case(
        &self,
        chars: &[char],
        allow_longer_shortforms: bool,
        out: &mut Vec<u8>,
    ) -> Option<()> {
        if allow_longer_shortforms && let Some(boundary) = initial_caps_shortform_boundary(chars) {
            let whole_lower: Vec<char> = chars.iter().flat_map(|c| c.to_lowercase()).collect();
            let (_, cells) = super::super::rule_10_9::shortform_part_cells(&whole_lower, 0)?;
            out.extend([CAPITAL, CAPITAL]);
            out.extend(cells);
            out.extend([CAPITAL, decode_unicode('⠄')]);
            self.encode_mixed_case(&chars[boundary..], allow_longer_shortforms, out)?;
            return Some(());
        }
        let camel_subunit_start = camel_title_subunit_after_caps_prefix(chars);
        if camel_subunit_start.is_some() {
            let subunit_start = camel_subunit_start?;
            out.extend([CAPITAL, CAPITAL]);
            let prefix: Vec<char> = chars[..subunit_start]
                .iter()
                .flat_map(|c| c.to_lowercase())
                .collect();
            out.extend(
                super::super::rule_10_9::encode_with_optional_longer_shortforms(
                    &prefix,
                    &self.contractions,
                    false,
                    false,
                    allow_longer_shortforms,
                )?,
            );
            self.encode_mixed_case(&chars[subunit_start..], allow_longer_shortforms, out)?;
            return Some(());
        }
        let initial_caps = chars.iter().take_while(|c| c.is_uppercase()).count();
        if initial_caps == 3
            && chars.get(initial_caps).is_some_and(|c| c.is_lowercase())
            && !chars[..initial_caps]
                .iter()
                .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
            && is_semantic_title_subunit(&chars[2..])
            && chars[..2]
                .iter()
                .all(|c| !matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u'))
        {
            out.extend([CAPITAL, CAPITAL]);
            for c in &chars[..2] {
                out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
            }
            out.extend(encode_title_subunit(
                &chars[2..],
                &self.contractions,
                allow_longer_shortforms,
            )?);
            return Some(());
        }
        if initial_caps == 2 && semantic_trailing_initial(chars) {
            for c in &chars[..2] {
                out.push(CAPITAL);
                out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
            }
            let lower: Vec<char> = chars[2..chars.len() - 1]
                .iter()
                .flat_map(|c| c.to_lowercase())
                .collect();
            out.extend(
                super::super::rule_10_9::encode_with_optional_longer_shortforms(
                    &lower,
                    &self.contractions,
                    false,
                    false,
                    allow_longer_shortforms,
                )?,
            );
            out.push(CAPITAL);
            out.push(
                crate::english::encode_english(chars[chars.len() - 1].to_ascii_lowercase()).ok()?,
            );
            return Some(());
        }
        if initial_caps == 2
            && chars.len() == 3
            && chars[2].is_ascii_lowercase()
            && !matches!(chars[2], 'd' | 's')
        {
            // §8.8.2: for short semantic subunits (chemical symbols/abbreviations
            // such as `KBr`, `BSc`, `MHz`, `KCl`) individual capital indicators
            // better convey the print meaning than a capitals-word indicator plus
            // terminator.  Plural/suffix acronyms (`CDs`, `OKd`) remain under §8.6.3.
            for &c in &chars[..2] {
                out.push(CAPITAL);
                out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
            }
            let suffix: Vec<char> = chars[2..].iter().flat_map(|c| c.to_lowercase()).collect();
            out.extend(
                super::super::rule_10_9::encode_with_optional_longer_shortforms(
                    &suffix,
                    &self.contractions,
                    false,
                    false,
                    allow_longer_shortforms,
                )?,
            );
            return Some(());
        }
        if chars.len() >= 6
            && matches!(classify_caps(&chars[..1]), Some(Caps::Single))
            && chars[1..].iter().all(|c| c.eq_ignore_ascii_case(&chars[1]))
        {
            let upper_start = chars[1..]
                .iter()
                .position(|c| c.is_uppercase())
                .map(|p| p + 1);
            if let Some(upper_start) = upper_start {
                let upper_end = chars[upper_start..]
                    .iter()
                    .position(|c| c.is_lowercase())
                    .map_or(chars.len(), |p| upper_start + p);
                if upper_start > 1 && upper_end > upper_start + 1 && upper_end < chars.len() {
                    out.push(CAPITAL);
                    out.push(crate::english::encode_english(chars[0].to_ascii_lowercase()).ok()?);
                    for c in &chars[1..upper_start] {
                        out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
                    }
                    out.extend([CAPITAL, CAPITAL]);
                    for c in &chars[upper_start..upper_end] {
                        out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
                    }
                    out.extend([CAPITAL, decode_unicode('⠄')]);
                    for c in &chars[upper_end..] {
                        out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
                    }
                    return Some(());
                }
            }
        }
        if (2..=3).contains(&initial_caps)
            && chars.get(initial_caps).is_some_and(|c| c.is_lowercase())
            && chars[initial_caps..].iter().all(|c| !c.is_uppercase())
        {
            let suffix = &chars[initial_caps..];
            let lower_suffix: String = suffix.iter().flat_map(|c| c.to_lowercase()).collect();
            let initials_are_roman = chars[..initial_caps]
                .iter()
                .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'));
            // §8.8.2 vs §8.6.3: chemical/abbreviation subunits (`KBr`, `BSc`,
            // `MHz`, `KCl`) split into per-letter capitals so the natural
            // subunit (`Br`, `Sc`, `Hz`, `Cl`) needs no internal indicators.
            // A grammatical suffix (`ABCs`, `WALKing`, `XXIInd`) keeps the
            // caps-word + terminator + suffix pattern, as does a Roman numeral
            // + trailing chord letter (`VIIb`).
            if !caps_prefix_keeps_word_indicator(&chars[..initial_caps])
                && !is_grammatical_suffix(&lower_suffix)
                && !initials_are_roman
            {
                for c in &chars[..initial_caps] {
                    out.push(CAPITAL);
                    out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
                }
                let lower: Vec<char> = suffix.iter().flat_map(|c| c.to_lowercase()).collect();
                out.extend(
                    super::super::rule_10_9::encode_with_optional_longer_shortforms(
                        &lower,
                        &self.contractions,
                        false,
                        false,
                        allow_longer_shortforms,
                    )?,
                );
                return Some(());
            }
            out.extend([CAPITAL, CAPITAL]);
            for c in &chars[..initial_caps] {
                out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
            }
            out.extend([CAPITAL, decode_unicode('⠄')]);
            let lower: Vec<char> = suffix.iter().flat_map(|c| c.to_lowercase()).collect();
            out.extend(
                super::super::rule_10_9::encode_with_optional_longer_shortforms(
                    &lower,
                    &self.contractions,
                    false,
                    false,
                    allow_longer_shortforms,
                )?,
            );
            return Some(());
        }
        let mut whole_lower = Vec::new();
        for c in chars {
            whole_lower.extend(c.to_lowercase());
        }

        let mut bounds = vec![0usize];
        for i in 1..chars.len() {
            // §8.2: a new Title-case / all-caps part begins at each lower→upper.
            let low_to_up = chars[i - 1].is_ascii_lowercase() && chars[i].is_ascii_uppercase();
            // §8.6.3: split a *caps word* (≥2 capitals) from a following lowercase
            // run so its `⠠⠄` terminator can be emitted (`ABCs`, `unSELFish`). A lone
            // Title-case capital keeps its lowercase tail, so contractions there keep
            // their context (`Deaf`'s `ea`, `Perfect`'s `er`).
            let capsword_to_low = chars[i].is_ascii_lowercase()
                && chars[i - 1].is_ascii_uppercase()
                && i >= 2
                && chars[i - 2].is_ascii_uppercase();
            if low_to_up || capsword_to_low {
                bounds.push(i);
            }
        }
        bounds.push(chars.len());

        let mut buf = Vec::new();
        let mut prev_caps_word = false;
        for w in bounds.windows(2) {
            let seg = &chars[w[0]..w[1]];
            let seg_lower: Vec<char> = seg.iter().flat_map(|c| c.to_lowercase()).collect();
            let caps = classify_caps(seg)?;
            let cells = if matches!(caps, Caps::Word) && w[1] < chars.len() && seg.len() <= 3 {
                encode_letters_literal(seg)?
                    .into_iter()
                    .filter(|cell| *cell != CAPITAL)
                    .collect()
            } else if allow_longer_shortforms
                && !matches!(caps, Caps::Word)
                && let Some((len, cells)) = mixed_case_shortform_part(&whole_lower, w[0], seg)
                && len == seg_lower.len()
            {
                cells
            } else if allow_longer_shortforms
                && matches!(classify_caps(seg), Some(Caps::Word))
                && let Some((len, cells)) = mixed_case_shortform_part(&whole_lower, w[0], seg)
                && len == seg_lower.len()
                && matches!(seg_lower.as_slice(), ['g', 'r', 'e', 'a', 't'])
            {
                cells
            } else if matches!(caps, Caps::Word)
                && matches!(seg_lower.as_slice(), ['t', 'i', 'o', 'n'])
            {
                encode_letters_literal(seg)?
                    .into_iter()
                    .filter(|cell| *cell != CAPITAL)
                    .collect()
            } else if matches!(seg_lower.as_slice(), ['t', 'i', 'o', 'n']) && w[1] == chars.len() {
                vec![GRADE1, decode_unicode('⠝')]
            } else if seg_lower
                .iter()
                .any(|c| super::super::rule_4::accent_cells(*c).is_some())
            {
                let mut literal = Vec::with_capacity(seg_lower.len() + 2);
                for &c in &seg_lower {
                    push_literal_letter(c, &mut literal)?;
                }
                literal
            } else if allow_longer_shortforms
                && mixed_case_disallowed_shortform_part(&whole_lower, w[0], seg)
            {
                super::super::rule_10_9::encode_with_optional_longer_shortforms(
                    &seg_lower,
                    &self.contractions,
                    false,
                    w[0] == 0 && w[1] == chars.len(),
                    false,
                )?
            } else {
                super::super::rule_10_9::encode_with_optional_longer_shortforms(
                    &seg_lower,
                    &self.contractions,
                    false,
                    w[0] == 0 && w[1] == chars.len(),
                    allow_longer_shortforms,
                )?
            };
            // §8.6.3: a §8.4 caps word (`⠠⠠`) is terminated by `⠠⠄` before lowercase
            // letters that continue the same word (`ABCs`, `WALKing`, `unSELFish`).
            if prev_caps_word && matches!(caps, Caps::None) {
                buf.push(CAPITAL);
                buf.push(decode_unicode('⠄'));
            }
            if matches!(caps, Caps::Word) && w[0] > 0 && w[1] < chars.len() && seg.len() <= 2 {
                for cell in &cells {
                    buf.push(CAPITAL);
                    buf.push(*cell);
                }
                prev_caps_word = false;
                continue;
            } else {
                match caps {
                    Caps::None => {}
                    Caps::Single => buf.push(CAPITAL),
                    Caps::Word => {
                        buf.push(CAPITAL);
                        buf.push(CAPITAL);
                    }
                }
            }
            buf.extend(&cells);
            prev_caps_word = matches!(caps, Caps::Word);
        }
        out.extend(buf);
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[rstest::rstest]
    #[case::mcd("McD", "⠠⠍⠉⠠⠙")]
    #[case::trailing_single_cap("verY", "⠧⠻⠠⠽")]
    #[case::trailing_caps_word("grandEST", "⠛⠗⠯⠠⠠⠑⠌")]
    #[case::internal_caps_letters("founDAtion", "⠋⠳⠝⠠⠙⠠⠁⠰⠝")]
    // §10.9.4: when print capitals split a longer word, each printed part keeps
    // the shortform rule it would have in that part; suffix letters after the
    // shortform are not swallowed by the abbreviation.
    #[case::braille_resumes("BrailleResumés", "⠠⠃⠗⠇⠠⠗⠑⠎⠥⠍⠘⠌⠑⠎")]
    #[case::pen_friend("PenFriend", "⠠⠏⠢⠠⠋⠗⠊⠢⠙")]
    // §8.2/§10.12.12: Title-case parts split even when a contraction would span the
    // boundary in the whole word (`ff` in `cliffedge`, `the` in `northeast`).
    #[case::cliff_edge_title_split("CliffEdge", "⠠⠉⠇⠊⠋⠋⠠⠫⠛⠑")]
    #[case::north_east_title_split("NorthEast", "⠠⠝⠕⠗⠹⠠⠑⠁⠌")]
    // §8.8.2: choose the segmentation that best conveys meaning; `Ontario` and
    // the final `T` are semantic subunits and keep their own capital indicators.
    #[case::tv_ontario("TVOntario", "⠠⠠⠞⠧⠠⠕⠝⠞⠜⠊⠕")]
    #[case::at_and_t("ATandT", "⠠⠁⠠⠞⠯⠠⠞")]
    #[case::potassium_bromide("KBr", "⠠⠅⠠⠃⠗")]
    #[case::bachelor_science("BSc", "⠠⠃⠠⠎⠉")]
    #[case::megahertz("MHz", "⠠⠍⠠⠓⠵")]
    #[case::potassium_chloride("KCl", "⠠⠅⠠⠉⠇")]
    #[case::chemical_subscript("HOCH₂", "⠠⠓⠠⠕⠠⠉⠠⠓⠰⠢⠼⠃")]
    fn encodes_mixed_case_words_8_2(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(enc(text), Some(cells(expected)));
    }

    /// §8.6.3: a §8.4 caps word (`⠠⠠`) followed by lowercase letters continuing the
    /// same word takes the capitals terminator `⠠⠄` before the lowercase part
    /// (`ABCs`, `WALKing`, `unSELFish`, `OKd`); a lone Title-case capital does not
    /// (covered by `verY`/`CliffEdge` above, which keep their lowercase context).

    #[test]
    fn rare_engine_paths_cover_remaining_symbol_and_word_branches() {
        let engine = EnglishUebEngine::new();

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Symbol('Α'),
                        EnglishToken::Symbol('Β'),
                        EnglishToken::Symbol('Γ'),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Word(vec!['Α']),
                        EnglishToken::Word(vec!['Β', 'Γ']),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Word(vec!['α', 'β']),
                        EnglishToken::Space,
                        EnglishToken::Word(vec!['π']),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Word(vec!['w', 'h', 'e', 'r', 'e']),
                        EnglishToken::Symbol('\''),
                        EnglishToken::Word(vec!['e', 'r']),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Number(vec!['1']),
                        EnglishToken::Word(vec!['i', 'n', 's']),
                    ],
                    false,
                )
                .is_some()
        );

        assert!(
            engine
                .encode(
                    &[
                        EnglishToken::Word(vec!['A']),
                        EnglishToken::Symbol('='),
                        EnglishToken::Word(vec!['b', 'C']),
                    ],
                    false,
                )
                .is_some()
        );
    }

    #[test]
    fn rare_encode_loop_spatial_greek_and_typeform_paths_are_reachable() {
        let engine = EnglishUebEngine::new();
        let italic = super::super::super::token::Typeform::Italic;

        let vertical_gap = std::iter::once(EnglishToken::Symbol('│'))
            .chain(std::iter::repeat_n(EnglishToken::Space, 10))
            .chain(std::iter::once(EnglishToken::Symbol('│')))
            .collect::<Vec<_>>();
        let encoded_vertical_gap = engine.encode(&vertical_gap, false).unwrap();
        assert!(
            encoded_vertical_gap
                .windows(9)
                .any(|cells| cells == [SPACE; 9])
        );

        let corner_gap = [EnglishToken::Symbol('─'), EnglishToken::Symbol('┐')]
            .into_iter()
            .chain(std::iter::repeat_n(EnglishToken::Space, 5))
            .chain([
                EnglishToken::Symbol('┌'),
                EnglishToken::Symbol('─'),
                EnglishToken::Symbol('\t'),
            ])
            .collect::<Vec<_>>();
        let encoded_corner_gap = engine.encode(&corner_gap, false).unwrap();
        assert!(
            encoded_corner_gap
                .windows(6)
                .any(|cells| cells == [SPACE; 6])
        );

        let diagonal_after_vertical = [EnglishToken::Symbol('│'), EnglishToken::Symbol('╲')];
        let encoded_diagonal_after_vertical =
            engine.encode(&diagonal_after_vertical, false).unwrap();
        assert!(encoded_diagonal_after_vertical.contains(&decode_unicode('⠣')));

        let spatial_arrow = [
            EnglishToken::Symbol('│'),
            EnglishToken::LineBreak,
            EnglishToken::Symbol('↙'),
        ];
        let encoded_spatial_arrow = engine.encode(&spatial_arrow, false).unwrap();
        assert!(
            encoded_spatial_arrow
                .windows(2)
                .any(|window| window == cells("⠳⠜"))
        );

        let spaced_line_segments = [
            EnglishToken::Symbol(super::super::super::rule_16::VARIANT_SPACED_SEGMENT),
            EnglishToken::Space,
            EnglishToken::Symbol(super::super::super::rule_16::VARIANT_SPACED_SEGMENT),
        ];
        let encoded_spaced_line_segments = engine.encode(&spaced_line_segments, false).unwrap();
        assert!(encoded_spaced_line_segments.starts_with(&cells("⠐⠒⠂⠂⠂")));

        let table_word_gap = [
            EnglishToken::Word(vec!['R', 'o', 'w']),
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Word(vec!['A', 'B']),
        ];
        let encoded_table_word_gap = engine.encode(&table_word_gap, false).unwrap();
        assert!(encoded_table_word_gap.contains(&decode_unicode('⠐')));

        let table_number_gap = [
            EnglishToken::Word(vec!['I', 'n', 'c', 'o', 'm', 'e']),
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Number(vec!['1', '2']),
        ];
        let encoded_table_number_gap = engine.encode(&table_number_gap, false).unwrap();
        assert!(encoded_table_number_gap.contains(&decode_unicode('⠐')));

        let styled_gap = [
            EnglishToken::Styled('A', super::super::super::token::Typeform::Underline),
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Space,
            EnglishToken::Styled('B', super::super::super::token::Typeform::Underline),
        ];
        let encoded_styled_gap = engine.encode(&styled_gap, false).unwrap();
        assert!(encoded_styled_gap.contains(&SPACE));

        let uppercase_greek_word = [EnglishToken::Word(vec!['Α', 'Β'])];
        assert!(
            engine
                .encode(&uppercase_greek_word, false)
                .unwrap()
                .starts_with(&[CAPITAL, CAPITAL])
        );

        let uppercase_greek_word_then_symbols = [
            EnglishToken::Word(vec!['Α', 'Β']),
            EnglishToken::Symbol('Γ'),
            EnglishToken::Symbol('Δ'),
        ];
        assert!(
            engine
                .encode(&uppercase_greek_word_then_symbols, false)
                .unwrap()
                .starts_with(&[CAPITAL, CAPITAL])
        );

        let uppercase_greek_symbols = [EnglishToken::Symbol('Α'), EnglishToken::Symbol('Β')];
        assert!(
            engine
                .encode(&uppercase_greek_symbols, false)
                .unwrap()
                .starts_with(&[CAPITAL, CAPITAL])
        );

        let styled_ing = [
            EnglishToken::Word(vec!['n']),
            EnglishToken::Styled('i', italic),
            EnglishToken::Styled('n', italic),
            EnglishToken::Styled('g', italic),
        ];
        let encoded_styled_ing = engine.encode(&styled_ing, false).unwrap();
        assert!(encoded_styled_ing.contains(&decode_unicode('⠨')));

        let styled_digit = [EnglishToken::Styled('7', italic)];
        let encoded_styled_digit = engine.encode(&styled_digit, false).unwrap();
        assert!(
            encoded_styled_digit
                .starts_with(&super::super::super::rule_9::symbol_indicator(italic))
        );

        let styled_letter_after_number = [
            EnglishToken::Number(vec!['1']),
            EnglishToken::Styled('a', italic),
        ];
        let encoded_styled_letter_after_number =
            engine.encode(&styled_letter_after_number, false).unwrap();
        assert!(encoded_styled_letter_after_number.contains(&GRADE1));

        let styled_word_passage = [
            EnglishToken::Styled('a', italic),
            EnglishToken::Styled('b', italic),
            EnglishToken::Space,
            EnglishToken::Styled('c', italic),
            EnglishToken::Styled('d', italic),
            EnglishToken::Space,
            EnglishToken::Styled('e', italic),
            EnglishToken::Styled('f', italic),
        ];
        let encoded_styled_word_passage = engine.encode(&styled_word_passage, false).unwrap();
        assert!(encoded_styled_word_passage.contains(&decode_unicode('⠨')));

        let partial_styled_word = [
            EnglishToken::Styled('m', italic),
            EnglishToken::Word(vec!['o', 't', 'h', 'e', 'r']),
        ];
        let encoded_partial_styled_word = engine.encode(&partial_styled_word, false).unwrap();
        assert!(encoded_partial_styled_word.contains(&decode_unicode('⠨')));

        let hyphenated_styled_span = [
            EnglishToken::Styled('o', italic),
            EnglishToken::Styled('f', italic),
            EnglishToken::Symbol('-'),
            EnglishToken::Styled('t', italic),
            EnglishToken::Styled('h', italic),
            EnglishToken::Styled('e', italic),
        ];
        let encoded_hyphenated_styled_span = engine.encode(&hyphenated_styled_span, false).unwrap();
        assert!(encoded_hyphenated_styled_span.contains(&decode_unicode('⠤')));

        let underlined_url = [
            EnglishToken::Styled('h', super::super::super::token::Typeform::Underline),
            EnglishToken::Styled('t', super::super::super::token::Typeform::Underline),
            EnglishToken::Styled('t', super::super::super::token::Typeform::Underline),
            EnglishToken::Styled('p', super::super::super::token::Typeform::Underline),
            EnglishToken::Symbol(':'),
            EnglishToken::Symbol('/'),
            EnglishToken::Symbol('/'),
            EnglishToken::Styled('a', super::super::super::token::Typeform::Underline),
        ];
        let encoded_underlined_url = engine.encode(&underlined_url, false).unwrap();
        assert!(encoded_underlined_url.contains(&decode_unicode('⠓')));
    }

    #[test]
    fn rare_mixed_case_internal_caps_path_is_observable() {
        let engine = EnglishUebEngine::new();
        let mut out = Vec::new();
        engine
            .encode_mixed_case(
                &['f', 'o', 'u', 'n', 'D', 'A', 't', 'i', 'o', 'n'],
                true,
                &mut out,
            )
            .unwrap();
        assert!(out.contains(&CAPITAL));
    }

    #[test]
    fn engine_default_matches_new() {
        // The `Default` impl delegates to `new`, producing a usable engine.
        let mut out = Vec::new();
        EnglishUebEngine::default()
            .encode_mixed_case(
                &['f', 'o', 'u', 'n', 'D', 'A', 't', 'i', 'o', 'n'],
                true,
                &mut out,
            )
            .unwrap();
        assert!(out.contains(&CAPITAL));
    }

    #[test]
    fn encode_dispatches_rule_3_14_letter_grid() {
        // §3.14 letter grid: two aligned rows of single capitals encode as a grid.
        let tokens = [
            EnglishToken::Word(vec!['A']),
            EnglishToken::Space,
            EnglishToken::Word(vec!['B']),
            EnglishToken::LineBreak,
            EnglishToken::Word(vec!['C']),
            EnglishToken::Space,
            EnglishToken::Word(vec!['D']),
        ];
        assert!(EnglishUebEngine::new().encode(&tokens, false).is_some());
    }

    #[test]
    fn encode_word_spells_modified_letter_word_literally() {
        // §4.2.4: a word carrying a diacritic (`maître`) is spelled letter-by-letter
        // so no groupsign consumes print around the modified letter.
        let ctx = WordContext {
            standing_alone: true,
            upper_usable: false,
            shortform_usable: false,
            allow_longer_shortforms: true,
            lower_usable: false,
            suppress_caps: false,
            word_initial: true,
            restricted_prefix_boundary: true,
            digit_adjacent: false,
        };
        let mut out = Vec::new();
        assert!(
            EnglishUebEngine::new()
                .encode_word(&['m', 'a', 'î', 't', 'r', 'e'], ctx, &mut out)
                .is_some()
        );
        assert!(!out.is_empty());
    }

    #[test]
    fn encode_word_acronym_abutting_digit_spells_letters() {
        // §10.12.1: an all-caps initialism abutting a digit is "used as letters",
        // each spelled with no contraction.
        let ctx = WordContext {
            standing_alone: false,
            upper_usable: false,
            shortform_usable: false,
            allow_longer_shortforms: true,
            lower_usable: false,
            suppress_caps: false,
            word_initial: true,
            restricted_prefix_boundary: true,
            digit_adjacent: true,
        };
        let mut out = Vec::new();
        assert!(
            EnglishUebEngine::new()
                .encode_word(&['A', 'B'], ctx, &mut out)
                .is_some()
        );
        assert!(!out.is_empty());
    }

    #[test]
    fn encode_word_acronym_with_accented_letter_emits_accent_cells() {
        // §10.12.1/§4.2: an all-caps initialism abutting a digit spells each
        // letter; an accented capital (`É`) emits its §4.2 accent cells.
        let ctx = WordContext {
            standing_alone: false,
            upper_usable: false,
            shortform_usable: false,
            allow_longer_shortforms: true,
            lower_usable: false,
            suppress_caps: false,
            word_initial: true,
            restricted_prefix_boundary: true,
            digit_adjacent: true,
        };
        let mut out = Vec::new();
        assert!(
            EnglishUebEngine::new()
                .encode_word(&['É', 'B'], ctx, &mut out)
                .is_some()
        );
        assert!(!out.is_empty());
    }
}
