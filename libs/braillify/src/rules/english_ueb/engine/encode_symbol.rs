fn spatial_row_has_earlier_vertical(tokens: &[EnglishToken], i: usize) -> bool {
    let row_start = tokens[..i]
        .iter()
        .rposition(|token| matches!(token, EnglishToken::LineBreak))
        .map_or(0, |position| position + 1);
    tokens[row_start..i]
        .iter()
        .any(|token| matches!(token, EnglishToken::Symbol('│')))
}

struct SymbolPassageContext<'a> {
    tokens: &'a [EnglishToken],
    index: usize,
    foreign_code: bool,
    spanish_foreign: bool,
    foreign_passage: bool,
}

fn guillemet_styled_passage(
    context: SymbolPassageContext<'_>,
    out: &mut Vec<u8>,
) -> Option<Option<ActiveTypeformPassage>> {
    if !matches!(context.tokens.get(context.index), Some(EnglishToken::Symbol('«'))) {
        return Some(None);
    }
    let Some(form) = context
        .tokens
        .get(context.index + 1)
        .and_then(token_typeform)
    else {
        return Some(None);
    };
    let (words, end) = styled_passage_extent(context.tokens, context.index + 1, form);
    if words < 3 {
        return Some(None);
    }
    out.extend(super::rule_9::passage_indicator(form));
    out.extend(super::rule_7::encode_punctuation('«')?);
    let detected_scope = styled_passage_foreign_scope(
        context.tokens,
        context.index + 1,
        end,
        form,
        context.foreign_code,
        context.spanish_foreign,
    );
    let scope = match detected_scope {
        Some(scope) => Some(scope),
        None if context.foreign_passage => Some((
            super::rule_13::AccentCode::Foreign,
            context.spanish_foreign,
        )),
        None => None,
    };
    Some(Some((end, form, false, scope)))
}

fn ueb_inverted_punctuation_cells(c: char) -> Vec<u8> {
    vec![
        decode_unicode('⠘'),
        decode_unicode('⠰'),
        decode_unicode(if c == '¡' { '⠖' } else { '⠦' }),
    ]
}

macro_rules! encode_symbol_arm {
    ($engine:expr, $tokens:ident, $out:ident, $prev_was_number:ident, $numeric_mode:ident, $skip_to:ident, $line_mode_active:ident, $passage:ident, $cap_term:ident, $in_passage:ident, $url_listing:ident, $regex_listing:ident, $foreign_code:ident, $spanish_foreign:ident, $foreign_passage:ident, $early_english:ident, $preserve_spatial_newlines:ident, $skip_flattened_line_indent:ident, $numeric_separator_count:ident, $i:ident, $c:ident) => {
			                {
			                    $skip_flattened_line_indent = false;
			                    if $passage.is_none()
			                        && let Some(active_passage) = guillemet_styled_passage(
			                            SymbolPassageContext {
			                                tokens: $tokens,
			                                index: $i,
			                                foreign_code: $foreign_code,
			                                spanish_foreign: $spanish_foreign,
			                                foreign_passage: $foreign_passage,
			                            },
			                            &mut $out,
			                        )?
			                    {
			                        $passage = Some(active_passage);
			                        $prev_was_number = false;
			                        $numeric_mode = false;
			                        continue;
			                    }
			                    if *$c == '~'
			                        && matches!($tokens.get($i + 1), Some(EnglishToken::Space))
			                        && matches!($tokens.get($i + 2), Some(EnglishToken::Styled(_, super::token::Typeform::Bold)))
			                    {
			                        let mut first_end = $i + 2;
			                        let mut first = Vec::new();
			                        while let Some(EnglishToken::Styled($c, super::token::Typeform::Bold)) =
			                            $tokens.get(first_end)
			                        {
			                            first.push(*$c);
			                            first_end += 1;
			                        }
			                        if matches!($tokens.get(first_end), Some(EnglishToken::Space))
			                            && matches!($tokens.get(first_end + 1), Some(EnglishToken::Styled(_, super::token::Typeform::Bold)))
			                        {
			                            let mut second_end = first_end + 1;
			                            let mut second = Vec::new();
			                            while let Some(EnglishToken::Styled($c, super::token::Typeform::Bold)) =
			                                $tokens.get(second_end)
			                            {
			                                second.push(*$c);
			                                second_end += 1;
			                            }
			                            if !first.is_empty() && !second.is_empty() {
			                                // §3.25: a swung dash can stand for the repeated
			                                // dictionary headword inside a styled phrase.
			                                $out.extend(super::rule_9::passage_indicator(
			                                    super::token::Typeform::Bold,
			                                ));
			                                $out.extend(super::rule_3::encode_symbol('~')?);
			                                $out.push(SPACE);
			                                $engine.encode_word(
			                                    &first,
			                                    WordContext {
			                                        standing_alone: true,
			                                        upper_usable: true,
			                                        shortform_usable: true,
			                                        allow_longer_shortforms: true,
			                                        lower_usable: true,
			                                        suppress_caps: true,
			                                        word_initial: true,
			                                        restricted_prefix_boundary: true,
			                                        digit_adjacent: false,
			                                    },
			                                    &mut $out,
			                                )?;
			                                $out.push(SPACE);
			                                $engine.encode_word(
			                                    &second,
			                                    WordContext {
			                                        standing_alone: true,
			                                        upper_usable: true,
			                                        shortform_usable: true,
			                                        allow_longer_shortforms: true,
			                                        lower_usable: true,
			                                        suppress_caps: true,
			                                        word_initial: true,
			                                        restricted_prefix_boundary: true,
			                                        digit_adjacent: false,
			                                    },
			                                    &mut $out,
			                                )?;
			                                $out.extend(super::rule_9::terminator(
			                                    super::token::Typeform::Bold,
			                                ));
			                                $skip_to = second_end;
			                                $prev_was_number = false;
			                                $numeric_mode = false;
			                                continue;
			                            }
			                        }
			                    }
		                    if let Some((cells, end)) = braille_mention_at($tokens, $i) {
		                    $out.extend(cells);
		                        $skip_to = end;
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if *$c == '^' {
		                        $out.extend([decode_unicode('⠈'), decode_unicode('⠢')]);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if matches!(*$c, '\u{266D}' | '\u{266F}' | '\u{266E}')
		                        && matches!($tokens.get($i + 1), Some(EnglishToken::Symbol('(')))
		                    {
		                        // §3.18 with §3.24: when a musical accidental is printed
		                        // as a modifier on the preceding symbol before a grouped
		                        // argument (`X♭(Y)`), write it as a superscript item.
		                        $out.extend([GRADE1, super::rule_3_24::ScriptKind::Superscript.indicator()]);
		                        $out.extend(super::rule_3::encode_symbol(*$c)?);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
			                    if $line_mode_active && !matches!(*$c, '→' | '↓') {
	                        $out.push(decode_unicode('⠄'));
	                        $line_mode_active = false;
	                    }
		                    if *$c == '_' {
	                        $out.extend([decode_unicode('⠨'), decode_unicode('⠤')]);
	                        let mut j = $i + 1;
	                        while matches!($tokens.get(j), Some(EnglishToken::Symbol('_'))) {
	                            j += 1;
	                        }
	                        $skip_to = j;
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
		                    if *$c == '-'
		                        && matches!($tokens.get($i + 1), Some(EnglishToken::Symbol('-')))
		                    {
                        let mut j = $i;
                        while matches!($tokens.get(j), Some(EnglishToken::Symbol('-'))) {
                            j += 1;
                        }
                        // §7.2.6: double hyphen used as a dash substitute (in typing
                        // or email). Two adjacent hyphens between complete words
                        // become one dash `⠠⠤`. A fragment word (`rec--ve`) keeps
                        // literal hyphens. Threshold ≥3 letters keeps common short
                        // words (`set`, `bat`, `she`) but excludes typical
                        // truncations (`ve`, `re`, `en`, `un`).
                        let dash_substitute = j == $i + 2
                            && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Word(w)) if w.len() >= 3)
                            && matches!($tokens.get(j), Some(EnglishToken::Word(w)) if w.len() >= 3);
                        if dash_substitute {
                            $out.extend([decode_unicode('⠠'), decode_unicode('⠤')]);
                        } else {
                            $out.extend(std::iter::repeat_n(decode_unicode('⠤'), j - $i));
                        }
                        $skip_to = j;
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
	                    }
                    if matches!(*$c, '–' | '—' | '―') {
	                        let repeated = matches!($tokens.get($i + 1), Some(EnglishToken::Symbol(next)) if *next == *$c);
	                        if repeated {
	                            $out.extend([decode_unicode('⠐'), decode_unicode('⠠'), decode_unicode('⠤')]);
	                            $skip_to = $i + 2;
	                        } else {
	                            if capital_omitted_letter_dash($tokens, $i) {
	                                $out.push(decode_unicode('⠐'));
	                            }
	                            if matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Word(_)))
	                                && matches!($tokens.get($i + 1), Some(EnglishToken::Space))
	                                && $out.last().copied() != Some(SPACE)
	                            {
	                                $out.push(SPACE);
	                            }
                            let adjacent_line_break = matches!(
                                $i.checked_sub(1).and_then(|p| $tokens.get(p)),
                                Some(EnglishToken::LineBreak)
                            ) || matches!($tokens.get($i + 1), Some(EnglishToken::LineBreak));
                            let midword_dash = matches!(
                                $i.checked_sub(1).and_then(|p| $tokens.get(p)),
                                Some(EnglishToken::Word(_))
                            ) && matches!($tokens.get($i + 1), Some(EnglishToken::Word(_)));
                            let has_short_and_long_dash = $tokens
                                .iter()
                                .any(|t| matches!(t, EnglishToken::Symbol('–')))
                                && $tokens.iter().any(|t| matches!(t, EnglishToken::Symbol('—')));
                            // §2.6.1: em-dash at the very start of an input with no
                            // preceding token is the "long dash" (`⠐⠠⠤`), used to signal
                            // omitted leading text (`—st`  ⠐⠠⠤⠎⠞).
                            let leading_em_dash = matches!(*$c, '—' | '―')
                                && $i == 0
                                && !matches!($tokens.get($i + 1), Some(EnglishToken::Symbol('¡' | '¿')));
                            if matches!(*$c, '—' | '―')
                                && (leading_em_dash
                                    || (has_short_and_long_dash
                                        && !adjacent_line_break
                                        && !midword_dash))
                            {
                                $out.push(decode_unicode('⠐'));
                            }
                            $out.extend([decode_unicode('⠠'), decode_unicode('⠤')]);
                        }
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if *$c == '-' && ends_spelled_letter_run_before_word($tokens, $i) {
	                        $out.push(decode_unicode('⠤'));
	                        $out.extend([GRADE1, decode_unicode('⠄')]);
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if *$c == '.' && matches!($tokens.get($i + 1), Some(EnglishToken::Symbol('.'))) {
	                        let mut j = $i;
	                        let mut dots = 0usize;
	                        while matches!($tokens.get(j), Some(EnglishToken::Symbol('.'))) {
	                            dots += 1;
	                            j += 1;
	                        }
                        if dots >= 3 {
                            if matches!($tokens.get(j), Some(EnglishToken::Space))
                                && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Word(_)))
                                && $out.last().copied() != Some(SPACE)
                            {
	                                $out.push(SPACE);
	                            }
	                            for _ in 0..dots {
	                                $out.push(decode_unicode('⠲'));
	                            }
	                            $skip_to = j;
	                            $prev_was_number = false;
	                            $numeric_mode = false;
	                            continue;
	                        }
	                    }
		                    if *$c == '.' && matches!($tokens.get($i + 1), Some(EnglishToken::Space)) {
                        let mut k = $i;
                        let mut dots = 0usize;
                        while matches!($tokens.get(k), Some(EnglishToken::Symbol('.'))) {
                            dots += 1;
                            k += 1;
                            if matches!($tokens.get(k), Some(EnglishToken::Space)) {
                                k += 1;
                            } else {
                                break;
                            }
                        }
                        while matches!($tokens.get(k), Some(EnglishToken::Space)) {
                            k += 1;
                        }
                        if dots >= 3 && matches!($tokens.get(k), Some(EnglishToken::Number(_))) {
                            // §16.5.1: guide dots MUST be flanked by "at least one
                            // blank cell before and after the sequence" — so emit a
                            // trailing space before the following Number regardless
                            // of the source dot count.
                            let cells = if dots >= 15 { 15 } else { 2 };
                            for _ in 0..cells {
                                $out.push(decode_unicode('⠐'));
                            }
                            $out.push(SPACE);
                            $skip_to = k;
                            $prev_was_number = false;
                            $numeric_mode = false;
                            continue;
                        }
                    }
                    if $early_english && *$c == '&' {
                        $out.extend([decode_unicode('⠈'), decode_unicode('⠯')]);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
		                    if let Some(cells) = match *$c {
		                        '©' | '®' | '™' | '□' | '✏' | '☞' | '✓' | '‰' => {
		                            super::rule_3::encode_symbol(*$c)
		                        }
		                        _ => None,
		                    } {
		                        $out.extend(cells);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if *$c == '-'
		                        && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Word(w)) if w.len() == 1 && w[0].eq_ignore_ascii_case(&'x'))
		                        && matches!($tokens.get($i + 1), Some(EnglishToken::Word(w)) if w.iter().collect::<String>().eq_ignore_ascii_case("it"))
		                    {
		                        // §2.6: a standalone wordsign-letter before a dash keeps the
		                        // letter reading (`x-it`) and the dash is the two-cell dash.
		                        $out.extend([decode_unicode('⠠'), decode_unicode('⠤')]);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if *$c == '×' {
	                        $out.extend([decode_unicode('⠐'), decode_unicode('⠦')]);
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
                    if let Some(cells) = greek_letter_cells_with_caps(*$c, $in_passage[$i]) {
                        $out.extend(cells);
                        if $cap_term[$i] {
                            $out.extend([CAPITAL, decode_unicode('⠄')]);
                        }
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
	                    }
	                    if matches!(*$c, '−' | '=') {
	                        $out.extend(if *$c == '=' {
	                            [decode_unicode('⠐'), decode_unicode('⠶')]
	                        } else {
	                            [decode_unicode('⠐'), decode_unicode('⠤')]
	                        });
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if *$c == '$'
                        && matches!($tokens.get($i + 1), Some(EnglishToken::Space))
                        && let Some(EnglishToken::Number(digits)) = $tokens.get($i + 2)
                    {
                        $out.extend([
                            decode_unicode('⠈'),
                            decode_unicode('⠎'),
                            super::rule_6::NUMERIC_INDICATOR,
                            SPACE,
                        ]);
                        for d in digits {
                            $out.push(super::rule_6::digit_cell(*d)?);
                        }
                        $skip_to = $i + 3;
                        $prev_was_number = true;
                        $numeric_mode = true;
                        continue;
                    }
                    if matches!(*$c, '′' | '″')
                        && $i.checked_sub(1).is_some_and(|p| {
                            matches!($tokens.get(p), Some(EnglishToken::Number(_)))
                                || matches!(
                                    $tokens.get(p),
                                    Some(EnglishToken::Word(w)) if w.len() == 1 && w[0].is_uppercase()
                                )
                        })
                    {
                        $out.push(decode_unicode('⠶'));
                        if *$c == '″' {
                            $out.push(decode_unicode('⠶'));
                        }
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if $spanish_foreign && *$c == '?' {
                        $out.push(decode_unicode('⠢'));
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
	                    if *$c == '↓'
	                        && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::LineBreak))
	                    {
	                        $out.extend([GRADE1, decode_unicode('⠳'), decode_unicode('⠩')]);
                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
                    if $preserve_spatial_newlines && *$c == '│' {
                        if let Some(EnglishToken::Symbol(next @ ('╲' | '╱'))) = $tokens.get($i + 1) {
                            let mut k = $i + 2;
                            let mut later_vertical = false;
                            while !matches!($tokens.get(k), None | Some(EnglishToken::LineBreak)) {
                                if matches!($tokens.get(k), Some(EnglishToken::Symbol('│'))) {
                                    later_vertical = true;
                                    break;
                                }
                                k += 1;
                            }
                            let earlier_vertical = spatial_row_has_earlier_vertical($tokens, $i);
                            if later_vertical || earlier_vertical {
                                $out.push(decode_unicode('⠸'));
                                $prev_was_number = false;
                                $numeric_mode = false;
                                continue;
                            }
                            $out.push(decode_unicode(if *next == '╲' { '⠣' } else { '⠜' }));
                            $prev_was_number = false;
                            $numeric_mode = false;
                            continue;
                        }
                        if matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Symbol('╲' | '╱'))) {
                            let later_diagonal = $tokens.get($i + 1).is_some_and(|next| {
                                matches!(next, EnglishToken::Symbol('╲' | '╱'))
                            });
                            if !later_diagonal {
                                $prev_was_number = false;
                                $numeric_mode = false;
                                continue;
                            }
                        }
                    }
	                    if $preserve_spatial_newlines
	                        && let Some(cells) = super::rule_16::line_arrow(*$c)
	                    {
                        $out.extend(cells);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    // §15.2.2: two adjacent primes `′′` denote a double-prime (bold
                    // prime) stress mark — one `⠘⠨⠃` cell pair, not two consecutive
                    // `⠘⠨⠆` cells. The single-`′` (⠘⠨⠆) case falls through to the
                    // rule_15::encode_symbol chain below.
                    if *$c == '′' && matches!($tokens.get($i + 1), Some(EnglishToken::Symbol('′'))) {
                        $out.extend([
                            decode_unicode('⠘'),
                            decode_unicode('⠨'),
                            decode_unicode('⠃'),
                        ]);
                        $skip_to = $i + 2;
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    let tone_level_context = $tokens.iter().any(|t| matches!(t, EnglishToken::Symbol('↑')))
                        && $tokens.iter().any(|t| matches!(t, EnglishToken::Symbol('↓')));
                    if tone_level_context
                        && matches!(*$c, '↑' | '↓')
                        && matches!($tokens.get($i + 1), Some(EnglishToken::Word(_)))
                    {
                        // §15.3.2: when tone is shown by level change, the arrow is a
                        // separate tone mark before a word, followed by a bullet under
                        // that word in braille.
                        $out.extend(super::rule_15::encode_symbol(*$c)?);
                        $out.extend([SPACE, decode_unicode('⠸'), decode_unicode('⠲')]);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if tone_level_context && matches!(*$c, '↑' | '↓') {
                        $out.extend(super::rule_15::encode_symbol(*$c)?);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if matches!(*$c, '←')
                        && let Some(cells) = super::rule_3::encode_symbol(*$c)
                    {
                        $out.extend(cells);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if matches!(*$c, '¡' | '¿')
                        && $passage.is_none()
                        && let Some(EnglishToken::Styled(_, form)) = $tokens.get($i + 1)
                    {
                        let (words, mut end) = styled_passage_extent($tokens, $i + 1, *form);
			                                let title_end = bibliography_styled_number_title_end($tokens, end, words);
			                            if words >= 3 || title_end.is_some() {
			                                if let Some(new_end) = title_end {
			                                    end = new_end;
			                                }
			                                $out.extend(super::rule_9::passage_indicator(*form));
                            let caps = styled_passage_all_caps($tokens, $i + 1, end, *form);
                            if caps {
                                $out.extend([CAPITAL, CAPITAL, CAPITAL]);
                            }
                            let scope = styled_passage_foreign_scope(
                                $tokens,
                                $i + 1,
                                end,
                                *form,
                                $foreign_code,
                                $spanish_foreign,
                            );
                            if matches!($tokens.get(end - 1), Some(EnglishToken::Symbol('.')))
                                && !styled_passage_introduced_by_colon($tokens, $i + 1)
                                && matches!(
                                    scope,
                                    Some((super::rule_13::AccentCode::Ueb, _))
                                )
                            {
                                end -= 1;
                            }
                            $passage = Some((end, *form, caps, scope));
                        }
                    }
                    if $foreign_passage
                        && *$c == '('
                        && let Some(EnglishToken::Word(word)) = $tokens.get($i + 1)
                        && matches!($tokens.get($i + 2), Some(EnglishToken::Symbol(')')))
                    {
                        let lower_word: String = word.iter().flat_map(|$c| $c.to_lowercase()).collect();
                        if super::pronunciation::cmudict::is_recorded_word(&lower_word) {
                            // §13.7.3 with §14.3.1: an English gloss inside a foreign
                            // passage is an embedded UEB word.  Open a non-UEB word
                            // indicator so the following parenthesised gloss is read in
                            // UEB (`(immediately)` → `⠘⠷⠐⠣⠊⠍⠍⠇⠽⠐⠜`).
                            $out.extend([decode_unicode('⠘'), decode_unicode('⠷')]);
                            $out.extend(super::rule_7::encode_punctuation('(')?);
                            let lower: Vec<char> = word.iter().flat_map(|$c| $c.to_lowercase()).collect();
                            $engine.encode_word(
                                &lower,
                                WordContext {
                                    standing_alone: true,
                                    upper_usable: true,
                                    shortform_usable: true,
                                    allow_longer_shortforms: true,
                                    lower_usable: true,
                                    suppress_caps: true,
                                    word_initial: true,
                                    restricted_prefix_boundary: true,
                                    digit_adjacent: false,
                                },
                                &mut $out,
                            )?;
                            $out.extend(super::rule_7::encode_punctuation(')')?);
                            $skip_to = $i + 3;
                            $prev_was_number = false;
                            $numeric_mode = false;
                            continue;
                        }
                    }
                    if $foreign_passage
                        && $passage.is_none()
                        && *$c == '('
                        && matches!($tokens.get($i + 1), Some(EnglishToken::Styled(..)))
                    {
                        // §13.6.4: in a foreign-code passage, French parentheses use
                        // the foreign-code grouping signs, not UEB round brackets.
                        $out.extend([decode_unicode('⠶'), decode_unicode('⠒')]);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if $foreign_passage
                        && $passage.is_none()
                        && *$c == ')'
                        && $i.checked_sub(1)
                            .and_then(|p| $tokens.get(p))
                            .is_some_and(|token| matches!(token, EnglishToken::Word(_) | EnglishToken::Styled(..)))
                        && parenthesized_foreign_style_before($tokens, $i)
                    {
                        $out.push(decode_unicode('⠶'));
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    // §7.1.3: a lower-cell punctuation mark whose cell collides with
                    // a lower contraction takes a grade-1 indicator ⠰ where that
                    // contraction could be read instead (a standing-alone `?`, a
                    // word-internal `:`, a word-initial `.`).
	                    if $regex_listing[$i]
	                        && *$c == '?'
	                        && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Symbol('"')))
	                        && matches!($tokens.get($i + 1), Some(EnglishToken::Symbol('+')))
	                    {
	                        $out.push(GRADE1);
	                    }
	                    if punctuation_grade1($tokens, $i, *$c) {
	                        $out.push(GRADE1);
	                    }
                    let shape_terminator = matches!(
                        $tokens.get($i + 1),
                        Some(
                            EnglishToken::Word(_)
                                | EnglishToken::Number(_)
                                | EnglishToken::Styled(..)
                                | EnglishToken::Technical(_)
                        )
                    );
                    // §13.5.1: an inverted Spanish exclamation/question mark
                    // adjacent to typography-marked (bold/italic) foreign
                    // vocabulary takes the UEB two-cell sign (`⠘⠰⠖` / `⠘⠰⠦`),
                    // not the foreign-code single cell. Triggered when the
                    // document contains ≤2 styled foreign-accent vocabulary
                    // words (§13.5.1 occasional foreign material, §13.7.2
                    // typography identifies foreign) and this `¡`/`¿` sits
                    // adjacent to that styled word. 3+ styled words trigger a
                    // §14.3.2 passage where `¿`/`¡` become foreign-code cells
                    // inside the passage indicator instead.
                    let ueb_inverted_punctuation = matches!(*$c, '¡' | '¿')
                        && ((styled_word_count($tokens) <= 2
                            && document_any_styled_phrase_has_foreign_letter($tokens)
                            && document_all_styled_phrases_are_short_vocabulary($tokens))
                            || matches!(
                                $passage,
                                Some((_, _, _, Some((super::rule_13::AccentCode::Ueb, _))))
                            ))
                        && punctuation_adjacent_to_styled($tokens, $i);
                    let cells = if ueb_inverted_punctuation {
                        Some(ueb_inverted_punctuation_cells(*$c))
                    } else if matches!(*$c, '↑' | '↓') {
                        super::rule_3::encode_symbol(*$c)
                    } else {
                        super::rule_15::encode_symbol(*$c)
                            .or_else(|| super::rule_11::encode_symbol(*$c, shape_terminator))
                            .or_else(|| super::rule_16::spatial_symbol(*$c))
                            .or_else(|| super::rule_7::encode_punctuation(*$c))
                            .or_else(|| super::rule_3::encode_symbol(*$c))
                            .or_else(|| super::rule_6::encode_vulgar_fraction(*$c))
                    }?;
		                    $out.extend(cells);
		                    if solidus_linebreak_space_after($tokens, $i) {
		                        $out.push(SPACE);
		                    }
		                    if url_listing_line_continuation_after($tokens, $i, &$url_listing) {
		                        $out.extend([decode_unicode('⠐'), SPACE]);
		                    }
	                    if *$c == ','
	                        && angle_group_comma($tokens, $i)
                        && !matches!($tokens.get($i + 1), Some(EnglishToken::Space))
                    {
                        $out.push(SPACE);
                    }
                    $prev_was_number = false;
                    if $numeric_mode
                        && *$c == ','
                        && matches!($tokens.get($i + 1), Some(EnglishToken::Number(_)))
                    {
                        $numeric_separator_count += 1;
                        if $numeric_separator_count == 6 {
                            $out.push(decode_unicode('⠐'));
                            $out.push(SPACE);
                        }
                    }
                    // §6.3: a `,` or `.` between two numbers is a digit separator —
                    // numeric mode (and thus the single `⠼`) carries across it. Any
                    // other symbol, or a `,`/`.` not flanked by digits, ends it.
                    $numeric_mode = $numeric_mode
                        && ((matches!($c, ',' | '.')
                            && matches!(
                                $tokens.get($i + 1),
                                Some(EnglishToken::Number(_))
                                    | Some(EnglishToken::Symbol('⎵'))
                            ))
                            || (*$c == '⎵'
                                && matches!(
                                    $tokens.get($i + 1),
                                    Some(EnglishToken::Number(_))
                                        | Some(EnglishToken::Symbol('⎵'))
                                )));
                    if !$numeric_mode {
                        $numeric_separator_count = 0;
                    }
                }
    };
}

#[cfg(test)]
mod tests {
    use super::test_support::enc;
    use super::*;

    #[test]
    fn guillemet_passage_uses_document_foreign_scope_fallback() {
        let form = super::super::token::Typeform::Italic;
        let tokens = [
            EnglishToken::Symbol('«'),
            EnglishToken::Styled('l', form),
            EnglishToken::Styled('e', form),
            EnglishToken::Space,
            EnglishToken::Styled('c', form),
            EnglishToken::Styled('h', form),
            EnglishToken::Styled('a', form),
            EnglishToken::Styled('t', form),
            EnglishToken::Space,
            EnglishToken::Styled('e', form),
            EnglishToken::Styled('s', form),
            EnglishToken::Styled('t', form),
        ];
        let mut out = Vec::new();

        let passage = guillemet_styled_passage(
            SymbolPassageContext {
                tokens: &tokens,
                index: 0,
                foreign_code: false,
                spanish_foreign: false,
                foreign_passage: true,
            },
            &mut out,
        )
        .flatten();

        assert!(matches!(
            passage,
            Some((_, _, false, Some((super::super::rule_13::AccentCode::Foreign, false))))
        ));
    }

    #[test]
    fn guillemet_short_styled_run_does_not_start_passage() {
        let form = super::super::token::Typeform::Italic;
        let tokens = [
            EnglishToken::Symbol('«'),
            EnglishToken::Styled('s', form),
            EnglishToken::Styled('a', form),
            EnglishToken::Styled('l', form),
            EnglishToken::Styled('u', form),
            EnglishToken::Styled('t', form),
        ];
        let mut out = Vec::new();

        let passage = guillemet_styled_passage(
            SymbolPassageContext {
                tokens: &tokens,
                index: 0,
                foreign_code: false,
                spanish_foreign: false,
                foreign_passage: true,
            },
            &mut out,
        );

        assert_eq!(passage, Some(None));
    }

    #[test]
    fn guillemet_english_styled_passage_needs_no_foreign_scope() {
        let input = "He wrote «\u{1D45F}\u{1D452}\u{1D451} \u{1D454}\u{1D45F}\u{1D452}\u{1D452}\u{1D45B} \u{1D44F}\u{1D459}\u{1D462}\u{1D452}";
        assert!(enc(input).is_some());

        let tokens = super::super::parser::parse_english(input);
        let mut out = Vec::new();
        let passage = guillemet_styled_passage(
            SymbolPassageContext {
                tokens: &tokens,
                index: 4,
                foreign_code: false,
                spanish_foreign: false,
                foreign_passage: false,
            },
            &mut out,
        );
        assert!(matches!(passage, Some(Some((_, _, false, None)))));
    }

    #[test]
    fn styled_all_caps_passage_closes_before_plain_prose() {
        assert!(enc("\u{1D411}\u{1D404}\u{1D403} \u{1D406}\u{1D411}\u{1D404}\u{1D404}\u{1D40D} \u{1D401}\u{1D40B}\u{1D414}\u{1D404} now").is_some());
    }
}
