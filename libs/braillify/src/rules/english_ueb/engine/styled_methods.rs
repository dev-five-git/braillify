use super::*;

impl EnglishUebEngine {
    /// §9: encode a styled word's base letters as an ordinary word (caps +
    /// contractions, with its standing-alone context taken from `tokens[i-1]` and
    /// `tokens[j]`) — the typeform indicator is emitted separately by the caller.
    pub(super) fn encode_styled_word(
        &self,
        chars: &[char],
        i: usize,
        j: usize,
        ctx: StyledContext<'_>,
        out: &mut Vec<u8>,
    ) -> Option<()> {
        let lower_word: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
        if super::super::rule_10_9::whole_word_cells(&lower_word).is_some() {
            let prev = i.checked_sub(1).map(|p| &ctx.tokens[p]);
            let next = ctx.tokens.get(j);
            let standing_alone = is_standing_alone(prev, next);
            return self.encode_word(
                chars,
                WordContext {
                    standing_alone,
                    upper_usable: false,
                    shortform_usable: standing_alone,
                    allow_longer_shortforms: true,
                    lower_usable: false,
                    suppress_caps: ctx.suppress_caps,
                    word_initial: word_initial_boundary(prev),
                    restricted_prefix_boundary: word_initial_boundary(prev),
                    digit_adjacent: false,
                },
                out,
            );
        }
        if let Some((accent_code, spanish)) = ctx.foreign_scope {
            let cells =
                super::super::rule_13::encode_uncontracted_word(chars, accent_code, spanish)?;
            out.extend(cells);
            return Some(());
        }
        if chars.iter().all(|c| c.is_ascii_digit()) {
            out.extend(super::super::rule_6::encode_number(chars)?);
            return Some(());
        }
        if chars.len() == 1 && !chars[0].is_ascii_alphabetic() {
            encode_styled_nonword_symbol(chars[0], out)?;
            return Some(());
        }
        let prev = i.checked_sub(1).map(|p| &ctx.tokens[p]);
        let next = ctx.tokens.get(j);
        if super::super::rule_10_9::is_pure_shortform_abbreviation(&lower_word) {
            let standing_alone = is_standing_alone(prev, next);
            return self.encode_word(
                chars,
                WordContext {
                    standing_alone,
                    upper_usable: false,
                    shortform_usable: standing_alone,
                    allow_longer_shortforms: true,
                    lower_usable: false,
                    suppress_caps: ctx.suppress_caps,
                    word_initial: word_initial_boundary(prev),
                    restricted_prefix_boundary: word_initial_boundary(prev),
                    digit_adjacent: false,
                },
                out,
            );
        }
        if !ctx.suppress_caps
            && let Some((accent_code, spanish)) =
                styled_phrase_foreign_scope(ctx.tokens, i, styled_form_at(ctx.tokens, i)?)
        {
            out.extend(super::super::rule_13::encode_uncontracted_word(
                chars,
                accent_code,
                spanish,
            )?);
            return Some(());
        }
        let form = styled_form_at(ctx.tokens, i)?;
        if super::super::rule_10_9::whole_word_cells(&lower_word).is_none()
            && styled_titlecase_phrase_from_named_place(ctx.tokens, i)
        {
            out.extend(super::super::rule_13::encode_uncontracted_word(
                chars,
                super::super::rule_13::AccentCode::Ueb,
                false,
            )?);
            return Some(());
        }
        if !ctx.suppress_caps
            && !styled_word_in_english_title(ctx.tokens, i, form)
            && !styled_word_in_lowercase_phrase_before_word(ctx.tokens, i, form, "of")
            && !domain_component_context(ctx.tokens, i)
            && styled_single_word_is_foreign(chars)
        {
            let doc_letters = document_letters(ctx.tokens);
            let accent_code = if super::super::rule_13::has_foreign_code_signal(&doc_letters) {
                super::super::rule_13::AccentCode::Foreign
            } else {
                super::super::rule_13::AccentCode::Ueb
            };
            out.extend(super::super::rule_13::encode_uncontracted_word(
                chars,
                accent_code,
                super::super::rule_13::spanish_context(&doc_letters),
            )?);
            return Some(());
        }
        if let Some(cells) =
            lower_sequence_before_apostrophe_cells(chars, &self.contractions, prev, next, true)
        {
            out.extend(lower_sequence_word_cells(chars, &cells)?);
            return Some(());
        }
        let standing_alone = is_standing_alone(prev, next);
        let shortform_usable =
            standing_alone && !matches!(next, Some(EnglishToken::Symbol('@' | '/')));
        let lower_usable = standing_alone
            && styled_lower_wordsign_usable(&lower_word, prev, next)
            && !styled_scansion_word(ctx.tokens, &lower_word);
        self.encode_word(
            chars,
            WordContext {
                standing_alone,
                upper_usable: standing_alone
                    && !matches!(prev, Some(EnglishToken::Symbol('/')))
                    && !matches!(next, Some(EnglishToken::Symbol('/'))),
                shortform_usable,
                allow_longer_shortforms: true,
                lower_usable,
                suppress_caps: ctx.suppress_caps,
                word_initial: word_initial_boundary(prev),
                restricted_prefix_boundary: word_initial_boundary(prev),
                digit_adjacent: false,
            },
            out,
        )
    }

    /// §9.5: encode a *multi-segment* styled word — its same-`form` styled letter
    /// runs (each as an ordinary word with its own §2.6 standing-alone context)
    /// and the symbols attached between them (`𝑜𝑓-𝑡ℎ𝑒` → `⠷⠤⠮`, `ℎ𝑡𝑡𝑝://…` →
    /// `⠓⠞⠞⠏⠒⠸⠌⠸⠌…`) — under the single typeform indicator emitted by the caller.
    pub(super) fn encode_styled_span(
        &self,
        start: usize,
        span_end: usize,
        form: super::super::token::Typeform,
        ctx: StyledContext<'_>,
        out: &mut Vec<u8>,
    ) -> Option<()> {
        // §13.2.1 hyphenated foreign compound: if ANY segment in the span is
        // foreign (`foih-𝑐ℎ𝑎𝑖`, `𝑙'𝑜𝑒𝑖𝑙-𝑑𝑒-𝑏𝑜𝑒𝑢𝑓`), every segment is uncontracted
        // — a single foreign word can be spelled across hyphens and its
        // anglicised-looking sub-segment (`chai`, `de`) shares the foreign
        // context. This is a span-level context that the per-segment
        // `styled_word_is_foreign` check cannot see.
        let span_foreign_scope = if ctx.foreign_scope.is_some() {
            ctx.foreign_scope
        } else {
            let mut any_foreign = false;
            let mut kk = start;
            while kk < span_end {
                if let Some(EnglishToken::Styled(_, f)) = ctx.tokens.get(kk)
                    && *f == form
                {
                    let seg_start = kk;
                    let mut seg = Vec::new();
                    while kk < span_end
                        && matches!(&ctx.tokens.get(kk), Some(EnglishToken::Styled(_, g)) if *g == form)
                    {
                        if let Some(EnglishToken::Styled(c, _)) = ctx.tokens.get(kk) {
                            seg.push(*c);
                        }
                        kk += 1;
                    }
                    // §13.2.1: any segment carrying explicit foreign evidence
                    // OR a segment that is not itself a recorded English word
                    // (`chai`, `foih`, `boeuf`) makes the whole hyphenated span
                    // foreign — the surrounding italic marks the compound as
                    // foreign per §13.1.2 and contractions are suppressed for
                    // every segment.
                    if !domain_component_context(ctx.tokens, seg_start)
                        && (styled_word_has_foreign_signal(&seg)
                            || styled_single_word_is_foreign(&seg))
                    {
                        any_foreign = true;
                        break;
                    }
                } else {
                    kk += 1;
                }
            }
            if any_foreign {
                let doc_letters = document_letters(ctx.tokens);
                let accent_code = if super::super::rule_13::has_foreign_code_signal(&doc_letters) {
                    super::super::rule_13::AccentCode::Foreign
                } else {
                    super::super::rule_13::AccentCode::Ueb
                };
                Some((
                    accent_code,
                    super::super::rule_13::spanish_context(&doc_letters),
                ))
            } else {
                None
            }
        };
        let mut k = start;
        while k < span_end {
            match &ctx.tokens[k] {
                EnglishToken::Styled(_, f) if *f == form => {
                    let seg_start = k;
                    let mut seg_chars = Vec::new();
                    while k < span_end
                        && matches!(&ctx.tokens[k], EnglishToken::Styled(_, g) if *g == form)
                    {
                        if let EnglishToken::Styled(c, _) = &ctx.tokens[k] {
                            seg_chars.push(*c);
                        }
                        k += 1;
                    }
                    self.encode_styled_word(
                        &seg_chars,
                        seg_start,
                        k,
                        StyledContext {
                            tokens: ctx.tokens,
                            suppress_caps: ctx.suppress_caps,
                            foreign_scope: span_foreign_scope,
                        },
                        out,
                    )?;
                }
                EnglishToken::Symbol(c) => {
                    let cells = super::super::rule_7::encode_punctuation(*c)
                        .or_else(|| super::super::rule_3::encode_symbol(*c))?;
                    out.extend(cells);
                    k += 1;
                }
                EnglishToken::LineBreak => {
                    super::super::rule_10_13::append_break(out, false);
                    k += 1;
                }
                _ => return None,
            }
        }
        Some(())
    }

    /// UEB §9.1.3: encode a URL-shaped underlined span with its typeform omitted
    /// because the underline is a hyperlink enhancement, not significant emphasis.
    pub(super) fn encode_styled_as_unstyled_span(
        &self,
        start: usize,
        span_end: usize,
        form: super::super::token::Typeform,
        ctx: StyledContext<'_>,
        out: &mut Vec<u8>,
    ) -> Option<()> {
        let mut k = start;
        while k < span_end {
            match &ctx.tokens[k] {
                EnglishToken::Styled(c, f) if *f == form && c.is_ascii_alphabetic() => {
                    let seg_start = k;
                    let mut seg_chars = Vec::new();
                    while k < span_end
                        && matches!(&ctx.tokens[k], EnglishToken::Styled(ch, g) if *g == form && ch.is_ascii_alphabetic())
                    {
                        if let EnglishToken::Styled(ch, _) = &ctx.tokens[k] {
                            seg_chars.push(*ch);
                        }
                        k += 1;
                    }
                    self.encode_styled_word(
                        &seg_chars,
                        seg_start,
                        k,
                        StyledContext {
                            tokens: ctx.tokens,
                            suppress_caps: ctx.suppress_caps,
                            foreign_scope: ctx.foreign_scope,
                        },
                        out,
                    )?;
                }
                EnglishToken::Styled(c, f) if *f == form && c.is_ascii_digit() => {
                    let mut digits = Vec::new();
                    while k < span_end
                        && matches!(&ctx.tokens[k], EnglishToken::Styled(ch, g) if *g == form && ch.is_ascii_digit())
                    {
                        if let EnglishToken::Styled(ch, _) = &ctx.tokens[k] {
                            digits.push(*ch);
                        }
                        k += 1;
                    }
                    out.extend(super::super::rule_6::encode_number(&digits)?);
                }
                EnglishToken::Styled(c, f) if *f == form => {
                    encode_styled_nonword_symbol(*c, out)?;
                    k += 1;
                }
                EnglishToken::Symbol(c) => {
                    let cells = super::super::rule_7::encode_punctuation(*c)
                        .or_else(|| super::super::rule_3::encode_symbol(*c))?;
                    out.extend(cells);
                    k += 1;
                }
                EnglishToken::LineBreak => {
                    super::super::rule_10_13::append_break(out, false);
                    k += 1;
                }
                _ => return None,
            }
        }
        Some(())
    }

    /// UEB §10.13.1-§10.13.12: encode an originally unhyphenated word with an
    /// explicit line-division point, never allowing a contraction to span it.
    pub(super) fn encode_divided_word(
        &self,
        chars: &[char],
        break_at: usize,
        suppress_caps: bool,
        out: &mut Vec<u8>,
    ) -> Option<()> {
        if break_at == 0 || break_at >= chars.len() {
            return None;
        }
        if classify_caps(chars).is_none() {
            self.encode_divided_mixed_case(chars, break_at, out)?;
            return Some(());
        }

        let lower: Vec<char> = chars.iter().flat_map(|c| c.to_lowercase()).collect();
        let mut first_line_has_upper_prefix = false;
        match classify_caps(chars)? {
            _ if suppress_caps => {}
            Caps::None => {}
            Caps::Single => {
                out.push(CAPITAL);
                first_line_has_upper_prefix = true;
            }
            Caps::Word => {
                out.push(CAPITAL);
                out.push(CAPITAL);
                first_line_has_upper_prefix = true;
            }
        }
        let cells = super::super::rule_10_9::encode_with_division(
            &lower,
            &self.contractions,
            super::super::rule_10_13::WordDivision { index: break_at },
            first_line_has_upper_prefix,
        )?;
        out.extend(cells);
        Some(())
    }

    /// §10.13 with §8.2: a mixed-case divided word is split into its printed line
    /// segments, so a capital at the start of line two keeps its own indicator.
    pub(super) fn encode_divided_mixed_case(
        &self,
        chars: &[char],
        break_at: usize,
        out: &mut Vec<u8>,
    ) -> Option<()> {
        self.encode_word(
            &chars[..break_at],
            WordContext {
                standing_alone: false,
                upper_usable: false,
                shortform_usable: false,
                allow_longer_shortforms: true,
                lower_usable: false,
                suppress_caps: false,
                word_initial: false,
                restricted_prefix_boundary: false,
                digit_adjacent: false,
            },
            out,
        )?;
        super::super::rule_10_13::append_break(out, true);
        self.encode_word(
            &chars[break_at..],
            WordContext {
                standing_alone: false,
                upper_usable: false,
                shortform_usable: false,
                allow_longer_shortforms: true,
                lower_usable: false,
                suppress_caps: false,
                word_initial: true,
                restricted_prefix_boundary: true,
                digit_adjacent: false,
            },
            out,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cells, enc};
    use super::*;

    #[test]
    fn styled_unstyled_span_helper_encodes_all_token_kinds() {
        let engine = EnglishUebEngine::new();
        let form = super::super::super::token::Typeform::Underline;
        let tokens = [
            EnglishToken::Styled('a', form),
            EnglishToken::Styled('b', form),
            EnglishToken::Symbol('/'),
            EnglishToken::Styled('1', form),
            EnglishToken::Styled('2', form),
            EnglishToken::LineBreak,
            EnglishToken::Styled('?', form),
        ];
        let mut out = Vec::new();
        engine
            .encode_styled_as_unstyled_span(
                0,
                tokens.len(),
                form,
                StyledContext {
                    tokens: &tokens,
                    suppress_caps: false,
                    foreign_scope: None,
                },
                &mut out,
            )
            .unwrap();
        assert_eq!(out, cells("⠁⠃⠸⠌⠼⠁⠃\n⠰⠦"));

        let wrong_form = [EnglishToken::Styled(
            'a',
            super::super::super::token::Typeform::Italic,
        )];
        let mut rejected = Vec::new();
        assert_eq!(
            engine.encode_styled_as_unstyled_span(
                0,
                wrong_form.len(),
                form,
                StyledContext {
                    tokens: &wrong_form,
                    suppress_caps: false,
                    foreign_scope: None,
                },
                &mut rejected,
            ),
            None
        );
    }

    #[test]
    fn styled_span_helper_preserves_line_breaks_between_segments() {
        let engine = EnglishUebEngine::new();
        let form = super::super::super::token::Typeform::Italic;
        let tokens = [
            EnglishToken::Styled('a', form),
            EnglishToken::LineBreak,
            EnglishToken::Styled('b', form),
        ];
        let mut out = Vec::new();

        engine
            .encode_styled_span(
                0,
                tokens.len(),
                form,
                StyledContext {
                    tokens: &tokens,
                    suppress_caps: false,
                    foreign_scope: None,
                },
                &mut out,
            )
            .unwrap();

        assert!(out.contains(&255));
    }

    #[test]
    fn styled_word_surface_encodes_plain_multiletter_run() {
        let engine = EnglishUebEngine::new();
        let form = super::super::super::token::Typeform::Italic;
        let tokens = [
            EnglishToken::Styled('r', form),
            EnglishToken::Styled('a', form),
            EnglishToken::Styled('d', form),
            EnglishToken::Styled('a', form),
            EnglishToken::Styled('r', form),
        ];

        let encoded = engine.encode(&tokens, false).unwrap();

        assert!(encoded.starts_with(&super::super::super::rule_9::word_indicator(form)));
    }

    #[test]
    fn divided_word_helper_covers_caps_mixed_and_invalid_breaks() {
        let engine = EnglishUebEngine::new();

        let mut invalid = Vec::new();
        assert_eq!(
            engine.encode_divided_word(&['c', 'a', 't'], 0, false, &mut invalid),
            None
        );
        assert_eq!(
            engine.encode_divided_word(&['c', 'a', 't'], 3, false, &mut invalid),
            None
        );

        let mut lower = Vec::new();
        engine
            .encode_divided_word(&['c', 'a', 't', 's'], 2, false, &mut lower)
            .unwrap();
        assert!(lower.contains(&255));

        let mut title = Vec::new();
        engine
            .encode_divided_word(&['C', 'a', 't', 's'], 2, false, &mut title)
            .unwrap();
        assert!(title.starts_with(&[CAPITAL]));

        let mut caps = Vec::new();
        engine
            .encode_divided_word(&['C', 'A', 'T', 'S'], 2, false, &mut caps)
            .unwrap();
        assert!(caps.starts_with(&[CAPITAL, CAPITAL]));
        assert!(caps.contains(&255));

        let mut mixed = Vec::new();
        engine
            .encode_divided_word(&['M', 'c', 'D', 'o', 'g'], 2, false, &mut mixed)
            .unwrap();
        assert!(mixed.contains(&255));
        assert!(mixed.iter().filter(|cell| **cell == CAPITAL).count() >= 2);
    }

    #[test]
    fn styled_word_foreign_detects_non_accented_foreign_letter() {
        // §13: a dot-below foreign letter (`ọ`, U+1ECD) is foreign but not a §4.2
        // accent, so it is detected as foreign vocabulary / a foreign signal.
        assert!(styled_word_is_foreign(&['\u{1ECD}']));
        assert!(styled_word_has_foreign_signal(&['\u{1ECD}']));
        // A recorded English word is not foreign.
        assert!(!styled_word_is_foreign(&['c', 'a', 't']));
    }

    #[test]
    fn encode_divided_word_suppressed_caps_path() {
        // §10.13: a line-divided word encoded inside a §8.4 caps passage
        // (suppress_caps) skips the per-word capital indicator.
        let mut out = Vec::new();
        assert!(
            EnglishUebEngine::new()
                .encode_divided_word(&['r', 'e', 'a', 'd', 'i', 'n', 'g'], 4, true, &mut out)
                .is_some()
        );
    }

    #[test]
    fn encode_styled_word_handles_single_symbol() {
        use super::super::super::token::Typeform;
        // A one-character styled word that is not a letter encodes as its symbol.
        let tokens = [EnglishToken::Styled('&', Typeform::Italic)];
        let ctx = StyledContext {
            tokens: &tokens,
            suppress_caps: false,
            foreign_scope: None,
        };
        let mut out = Vec::new();
        assert!(
            EnglishUebEngine::new()
                .encode_styled_word(&['&'], 0, 1, ctx, &mut out)
                .is_some()
        );
        assert!(!out.is_empty());
    }

    #[test]
    fn encode_styled_span_rejects_non_styled_interior_token() {
        use super::super::super::token::Typeform;
        // A styled span whose interior carries a bare Space (not styled / symbol /
        // line break) is not a well-formed multi-segment styled word.
        let tokens = [
            EnglishToken::Styled('x', Typeform::Italic),
            EnglishToken::Space,
            EnglishToken::Styled('y', Typeform::Italic),
        ];
        let ctx = StyledContext {
            tokens: &tokens,
            suppress_caps: false,
            foreign_scope: None,
        };
        let mut out = Vec::new();
        assert!(
            EnglishUebEngine::new()
                .encode_styled_span(0, 3, Typeform::Italic, ctx, &mut out)
                .is_none()
        );
    }

    #[test]
    fn encode_styled_word_encodes_all_digit_word_as_number() {
        use super::super::super::token::Typeform;
        // §6/§9: a styled word made only of digits encodes as a number.
        let tokens = [
            EnglishToken::Styled('1', Typeform::Italic),
            EnglishToken::Styled('2', Typeform::Italic),
        ];
        let ctx = StyledContext {
            tokens: &tokens,
            suppress_caps: false,
            foreign_scope: None,
        };
        let mut out = Vec::new();
        assert!(
            EnglishUebEngine::new()
                .encode_styled_word(&['1', '2'], 0, 2, ctx, &mut out)
                .is_some()
        );
        assert!(!out.is_empty());
    }

    #[test]
    fn styled_titlecase_place_phrase_encodes_uncontracted_word() {
        assert!(enc("\u{1D43A}\u{1D45F}\u{1D452}\u{1D44E}\u{1D461} \u{1D439}\u{1D44E}\u{1D459}\u{1D459}\u{1D460} from Montana").is_some());
    }
}
