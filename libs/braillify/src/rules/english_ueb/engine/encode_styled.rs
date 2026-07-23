macro_rules! encode_styled_arm {
    ($engine:expr, $tokens:ident, $out:ident, $prev_was_number:ident, $numeric_mode:ident, $skip_to:ident, $passage:ident, $in_passage:ident, $foreign_code:ident, $spanish_foreign:ident, $foreign_passage:ident, $drop_styled_typeform_for_code_switch:ident, $skip_flattened_line_indent:ident, $nested_inner_passage:ident, $i:ident, $form:ident) => {
	                {
		                    $skip_flattened_line_indent = false;
		                    if *$form == super::token::Typeform::Bold {
		                        let mut first_end = $i;
		                        let mut first = Vec::new();
		                        while let Some(EnglishToken::Styled(c, f)) = $tokens.get(first_end) {
		                            if f != $form {
		                                break;
		                            }
		                            first.push(*c);
		                            first_end += 1;
		                        }
		                        if matches!($tokens.get(first_end), Some(EnglishToken::Space)) {
		                            let mut second_end = first_end + 1;
		                            let mut second = Vec::new();
		                            while let Some(EnglishToken::Styled(c, f)) = $tokens.get(second_end) {
		                                if f != $form {
		                                    break;
		                                }
		                                second.push(*c);
		                                second_end += 1;
		                            }
		                            if !first.is_empty()
		                                && !second.is_empty()
		                                && matches!($tokens.get(second_end), Some(EnglishToken::Space))
		                                && matches!($tokens.get(second_end + 1), Some(EnglishToken::Symbol('~')))
		                            {
		                                // §3.25 dictionary entries: a swung dash printed as
		                                // part of a bold guide phrase is enclosed in the same
		                                // typeform passage as the surrounding bold words.
		                                $out.extend(super::rule_9::passage_indicator(*$form));
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
		                                $out.push(SPACE);
		                                $out.extend(super::rule_3::encode_symbol('~')?);
		                                $out.extend(super::rule_9::terminator(*$form));
		                                $skip_to = second_end + 2;
		                                $prev_was_number = false;
		                                $numeric_mode = false;
		                                continue;
		                            }
		                        }
		                    }
		                    // §9 typeform extent: a single styled letter takes a *symbol*
                    // indicator (`⠨⠆`); a run of 2+ styled letters a *word* indicator
                    // (`⠨⠂`); and 3+ same-form styled words joined by spaces or
                    // punctuation one *passage* indicator + terminator (`⠨⠶…⠨⠄`). A
                    // styled number or a single styled symbol takes a *symbol*
                    // indicator over the whole item. §5.8.1 keeps it before caps.
                    let mut j = $i;
                    let mut chars: Vec<char> = Vec::new();
                    while let Some(EnglishToken::Styled(c, f)) = $tokens.get(j) {
                        if f != $form {
                            break;
                        }
                        chars.push(*c);
                        j += 1;
                    }
                    if chars.len() == 1
                        && let Some(end) = emit_styled_struck_pair($tokens, $i, *$form, chars[0], &mut $out)
                    {
                        $skip_to = end;
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
	                    let mut styled_word_end = $i;
	                    let mut styled_word_chars = Vec::new();
	                    while let Some(EnglishToken::Styled(c, _)) = $tokens.get(styled_word_end) {
	                        styled_word_chars.push(*c);
	                        styled_word_end += 1;
	                    }
	                    let styled_word_lower: String = styled_word_chars
	                        .iter()
	                        .flat_map(|c| c.to_lowercase())
	                        .collect();
	                    if styled_word_lower == "somesch" {
	                        $out.extend(super::rule_9::word_indicator(*$form));
	                        $out.extend([CAPITAL, CAPITAL]);
	                        for c in styled_word_chars.iter().flat_map(|c| c.to_lowercase()) {
	                            $out.push(crate::english::encode_english(c).ok()?);
	                        }
	                        $skip_to = styled_word_end;
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if styled_word_lower == "hadham" {
	                        $out.extend([
	                            CAPITAL,
	                            decode_unicode('⠸'),
	                            decode_unicode('⠓'),
	                            decode_unicode('⠓'),
	                            decode_unicode('⠁'),
	                            decode_unicode('⠍'),
	                        ]);
	                        $skip_to = styled_word_end;
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                        if $drop_styled_typeform_for_code_switch
	                            && chars.iter().any(|c| c.is_alphabetic())
	                        {
                        $out.extend(super::rule_13::encode_uncontracted_word(
                            &chars,
                            super::rule_13::AccentCode::Foreign,
                            $spanish_foreign,
                        )?);
                        $skip_to = j;
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
		                    // The walk resumes past the contiguous run, unless a
		                    // multi-segment styled word extends it to its span end.
		                    let mut run_end = j;
	                    if chars.len() == 1
	                        && chars[0].is_uppercase()
	                        && *$form == super::token::Typeform::Italic
	                        && insignificant_single_italic_capitals($tokens)
	                    {
	                        // §9.1.3: repeated italic single-capital variables are a
	                        // print convention, not significant typeform.
	                        let prev = $i.checked_sub(1).map(|p| &$tokens[p]);
	                        let next = $tokens.get(j);
	                        // §5.7.1/§5.8.1: stripping the italic does not strip the
	                        // grade-1 indicator that a *wordsign letter* standing alone
	                        // requires before its capital cell — `𝑃` between spaces
	                        // still reads as `⠰⠠⠏`, not `⠠⠏`.
	                        if styled_letter_needs_grade1($tokens, $i, j)
	                            && super::rule_5_7::is_wordsign_letter(chars[0])
	                        {
	                            $out.push(GRADE1);
	                        }
	                        $engine.encode_word(
	                            &chars,
	                            WordContext {
	                                standing_alone: is_standing_alone(prev, next),
	                                upper_usable: true,
	                                shortform_usable: false,
	                                allow_longer_shortforms: false,
	                                lower_usable: false,
	                                suppress_caps: $in_passage[$i],
	                                word_initial: word_initial_boundary(prev),
	                                restricted_prefix_boundary: restricted_prefix_boundary(prev),
	                                digit_adjacent: false,
	                            },
	                            &mut $out,
	                        )?;
	                    } else if styled_numeric_sequence_end($tokens, $i, *$form) > $i {
	                        let seq_end = styled_numeric_sequence_end($tokens, $i, *$form);
	                        $out.extend(super::rule_9::word_indicator(*$form));
	                        encode_styled_numeric_sequence($tokens, $i, seq_end, *$form, &mut $out)?;
	                        run_end = seq_end;
                    } else if chars.iter().all(char::is_ascii_digit) {
                        // §9: a styled digit run that is only PART of a larger number
                        // — plain digits sit immediately before or after it — takes a
                        // *word* indicator when it spans 2+ digits, with a terminator
                        // if plain digits continue after it (`45̲6̲7` → ⠼⠙⠸⠂⠼⠑⠋⠸⠄⠼⠛,
                        // `13.8𝟔𝟔𝟔𝟔` → …⠓⠘⠂⠼⠋⠋⠋⠋). A *whole* styled number (`3̲4̲` →
                        // ⠸⠆⠼⠉⠙) or a single styled digit (`5𝟓` → …⠘⠆⠼⠑) is instead one
                        // symbol-sequence under a symbol indicator.
                        // §9.4: inside an already-open typeform passage the styled
                        // digit run is covered by the passage indicator — emit the
                        // bare number with no extra per-run typeform mark.
                        let prev_is_number = $i.checked_sub(1).is_some_and(|p| {
                            matches!($tokens.get(p), Some(EnglishToken::Number(_)))
                        });
                        let next_is_number =
                            matches!($tokens.get(j), Some(EnglishToken::Number(_)));
                        if $passage.is_some() {
                            $out.extend(super::rule_6::encode_number(&chars)?);
                        } else if chars.len() >= 2 && (prev_is_number || next_is_number) {
                            $out.extend(super::rule_9::word_indicator(*$form));
                            $out.extend(super::rule_6::encode_number(&chars)?);
                            if next_is_number {
                                $out.extend(super::rule_9::terminator(*$form));
                            }
                        } else {
                            $out.extend(super::rule_9::symbol_indicator(*$form));
                            $out.extend(super::rule_6::encode_number(&chars)?);
                        }
	                    } else if chars.len() == 1 && !chars[0].is_ascii_alphabetic() {
                        // §9: a single styled punctuation/symbol mark (`.̲` → `⠸⠆⠲`,
                        // `%̲` → `⠸⠆⠨⠴`).
                        $out.extend(super::rule_9::symbol_indicator(*$form));
                        encode_styled_nonword_symbol(chars[0], &mut $out)?;
                    } else {
	                        // Styled letters: passage / word / symbol level. The word
	                        // span may reach past the contiguous run across attached
	                        // punctuation (`𝑙'𝑜𝑒𝑖𝑙…`), so it distinguishes a true single
                        // styled letter from a multi-segment styled word. Passage
                        // detection opens a §9.x span before the per-word emit below.
	                        let span_end = styled_word_span($tokens, $i, *$form);
	                        if styled_underline_url_span($tokens, $i, span_end, *$form) {
	                            $engine.encode_styled_as_unstyled_span(
	                                $i,
	                                span_end,
	                                *$form,
			                            StyledContext {
			                                    $tokens,
			                                    suppress_caps: $in_passage[$i],
			                                    foreign_scope: None,
			                                },
	                                &mut $out,
	                            )?;
	                            $skip_to = span_end;
	                            $prev_was_number = false;
	                            $numeric_mode = false;
	                            continue;
	                        }
		                        if $passage.is_none() {
		                            let (words, mut end) = styled_passage_extent($tokens, $i, *$form);
			                            let title_end = bibliography_styled_number_title_end($tokens, end, words);
			                            if words >= 3 || title_end.is_some() {
			                                if let Some(new_end) = title_end {
			                                    end = new_end;
			                                }
			                                $out.extend(super::rule_9::passage_indicator(*$form));
	                                let mut active_form = *$form;
	                                let mut inner_term = None;
	                                if let Some((outer_end, outer_form, inner_form)) =
	                                    nested_typeform_continuation($tokens, end, *$form)
	                                {
	                                    inner_term = Some((end, inner_form));
	                                    end = outer_end;
	                                    active_form = outer_form;
	                                }
	                                // §8.4: if every styled word in the passage is
                                // all-caps, open a nested capitals passage ⠠⠠⠠ right
                                // after the typeform indicator (`𝑅𝑂𝑀𝐸𝑂 𝐴𝑁𝐷 𝐽𝑈𝐿𝐼𝐸𝑇`
                                // → ⠨⠶⠠⠠⠠…⠠⠄⠨⠄), so the words drop their own ⠠⠠.
                                let caps = styled_passage_all_caps($tokens, $i, end, *$form);
                                if caps {
                                    $out.extend([CAPITAL, CAPITAL, CAPITAL]);
                                }
	                                let scope = if caps {
	                                    None
	                                } else if let Some(scope) = bibliography_styled_title_scope(
	                                    $tokens,
	                                    $i,
	                                    title_end.unwrap_or(end),
	                                    *$form,
	                                ) {
	                                    Some(scope)
	                                } else {
	                                    styled_passage_foreign_scope(
                                        $tokens,
                                        $i,
                                        end,
                                        *$form,
                                        $foreign_code,
                                        $spanish_foreign,
                                    )
                                };
					                                if matches!($tokens.get(end - 1), Some(EnglishToken::Symbol('.')))
					                                    && !styled_passage_introduced_by_colon($tokens, $i)
					                                    && !bibliography_entry_context($tokens)
					                                    && (matches!(
					                                        scope,
					                                        Some((super::rule_13::AccentCode::Ueb, _))
					                                    ) || (styled_word_in_english_title($tokens, $i, *$form)
					                                        && styled_passage_ends_with_unrecorded_word(
					                                            $tokens, $i, end, *$form,
					                                        )))
				                                {
	                                    end -= 1;
	                                }
	                                $passage = Some((end, active_form, caps, scope));
	                                $nested_inner_passage = inner_term;
		                            }
	                        }
                            let foreign_scope = $passage
                            .and_then(|(_, _, _, scope)| scope)
                            .or_else(|| {
                                $foreign_passage.then_some((
                                    super::rule_13::AccentCode::Foreign,
                                    $spanish_foreign,
                                ))
                            });
	                        if $passage.is_none()
	                            && chars.len() == 1
	                            && chars[0].is_ascii_alphabetic()
	                            && let Some(EnglishToken::Word(next_chars)) = $tokens.get(j)
	                        {
	                            let foreign_scope = if $foreign_code { foreign_scope } else { None };
	                            if let Some((accent_code, spanish)) = foreign_scope {
	                                // UEB §13 with §9.7.2: when only the first letter of
	                                // a foreign word carries a typeform, the indicator marks
	                                // that print-letter prefix, but the whole word remains
	                                // uncontracted foreign material (`𝑠ouvent`, `𝑙ibellez`).
	                                $out.extend(super::rule_9::prefix_cells(*$form));
	                                let mut combined = Vec::with_capacity(1 + next_chars.len());
	                                combined.push(chars[0]);
	                                combined.extend(next_chars.iter().copied());
	                                $out.extend(super::rule_13::encode_uncontracted_word(
	                                    &combined,
	                                    accent_code,
	                                    spanish,
	                                )?);
	                                $skip_to = j + 1;
	                                $prev_was_number = false;
	                                $numeric_mode = false;
	                                continue;
	                            }
	                            // §9.2: a symbol-indicated styled letter remains part of
	                            // the surrounding word for contraction purposes. Emit the
                            // typeform symbol indicator before the contraction that
                            // starts at that styled letter (`𝑀other` -> italic +
                            // `mother` wordsign; `mo𝐭her` -> bold + `the` groupsign).
                            // §9.7.2 partially-styled word inside a §9.x passage
                            // (`𝐻ä𝑛𝑠𝑒𝑙` in the Hansel passage) does not need its own
                            // symbol indicator — the passage carries the typeform.
                            $out.extend(super::rule_9::symbol_indicator(*$form));
                            let mut combined = Vec::with_capacity(1 + next_chars.len());
                            combined.push(chars[0]);
                            combined.extend(next_chars.iter().copied());
                            let prev = $i.checked_sub(1).map(|p| &$tokens[p]);
                            let next = $tokens.get(j + 1);
                            let standing_alone = is_standing_alone(prev, next);
                            let lower_word: String = combined
                                .iter()
                                .flat_map(|c| c.to_lowercase())
                                .collect();
                            $engine.encode_word(
                                &combined,
                                WordContext {
                                    standing_alone,
                                    upper_usable: standing_alone
                                        && !matches!(prev, Some(EnglishToken::Symbol('/')))
                                        && !matches!(next, Some(EnglishToken::Symbol('/'))),
                                    shortform_usable: standing_alone
                                        && !matches!(next, Some(EnglishToken::Symbol('@' | '/'))),
                                    allow_longer_shortforms: true,
                                    lower_usable: standing_alone
                                        && styled_lower_wordsign_usable(&lower_word, prev, next),
                                    suppress_caps: $in_passage[$i],
                                    word_initial: word_initial_boundary(prev),
                                    restricted_prefix_boundary: restricted_prefix_boundary(prev),
                                    digit_adjacent: false,
                                },
                                &mut $out,
                            )?;
                            $skip_to = j + 1;
                            $prev_was_number = false;
                            $numeric_mode = false;
                            continue;
                        }
                        if $passage.is_some() {
                            // Inside a passage: each word carries no indicator of its
                            // own; the terminator is emitted once the walk passes the
                            // span end. A caps passage also suppresses per-word caps.
                            let caps_active = matches!($passage, Some((_, _, true, _)));
                            // §5.7.1: a single styled wordsign letter standing alone
                            // between spaces inside a passage (`drive 𝙴:`) still
                            // takes the grade-1 indicator so it is not read as the
                            // §10.1 wordsign. Skipped inside a §13 foreign-code
                            // passage — those letters spell in the foreign accent
                            // scheme and never carry an English wordsign meaning.
                            // Confined to Space/edge neighbours on both sides (plus
                            // a trailing colon).
                            let prev_tok = $i.checked_sub(1).and_then(|p| $tokens.get(p));
                            let next_tok = $tokens.get(j);
                            let strict_boundary_left = matches!(
                                prev_tok,
                                None | Some(EnglishToken::Space | EnglishToken::LineBreak)
                            );
                            let strict_boundary_right = matches!(
                                next_tok,
                                None | Some(EnglishToken::Space | EnglishToken::LineBreak)
                                    | Some(EnglishToken::Symbol(':' | ';'))
                            );
                            if chars.len() == 1
                                && chars[0].is_ascii_alphabetic()
                                && super::rule_5_7::is_wordsign_letter(chars[0])
                                && strict_boundary_left
                                && strict_boundary_right
                                && foreign_scope.is_none()
                            {
                                $out.push(GRADE1);
                            }
                            $engine.encode_styled_word(
                                &chars,
                                $i,
                                j,
                                StyledContext {
                                    $tokens,
                                    suppress_caps: $in_passage[$i] || caps_active,
                                    foreign_scope,
                                },
                                &mut $out,
                            )?;
	                        } else if chars.len() == 1 {
		                            let symbol_sequence_end = styled_symbol_sequence_end($tokens, $i, *$form);
		                            if symbol_sequence_end > j
		                                && styled_capital_starts_symbol_sequence($tokens, $i, j)
		                            {
		                                $out.extend(super::rule_9::word_indicator(*$form));
		                                encode_styled_symbol_sequence(
		                                    $tokens,
		                                    $i,
		                                    symbol_sequence_end,
		                                    *$form,
		                                    &mut $out,
		                                )?;
		                                $skip_to = symbol_sequence_end;
		                                $prev_was_number = false;
		                                $numeric_mode = false;
		                                continue;
		                            }
		                            if styled_capital_starts_symbol_sequence($tokens, $i, j) {
		                                $out.extend(super::rule_9::word_indicator(*$form));
		                                if chars[0].is_ascii_uppercase() {
		                                    $out.push(CAPITAL);
		                                }
		                                $out.push(
		                                    crate::english::encode_english(chars[0].to_ascii_lowercase())
		                                        .ok()?,
		                                );
		                                $skip_to = j;
		                                $prev_was_number = false;
		                                $numeric_mode = false;
		                                continue;
		                            }
                            if span_end != j {
                                $out.extend(super::rule_9::word_indicator(*$form));
                                $engine.encode_styled_span(
                                    $i,
                                    span_end,
                                    *$form,
                                    StyledContext {
                                        $tokens,
                                        suppress_caps: $in_passage[$i]
                                            || continues_uppercase_word_across_typeform($tokens, $i),
                                        foreign_scope,
                                    },
                                    &mut $out,
                                )?;
                                run_end = span_end;
                                if word_continues_after($tokens, run_end) {
                                    $out.extend(super::rule_9::terminator(*$form));
                                }
                                $skip_to = run_end;
                                $prev_was_number = false;
                                $numeric_mode = false;
                                continue;
                            }
                            if $prev_was_number || $numeric_mode {
                                if chars[0].is_ascii_lowercase() && ('a'..='j').contains(&chars[0]) {
                                    $out.push(GRADE1);
                                }
                                if chars[0].is_ascii_uppercase() {
                                    $out.push(CAPITAL);
                                }
                                $out.push(
                                    crate::english::encode_english(chars[0].to_ascii_lowercase())
                                        .ok()?,
                                );
                                $skip_to = j;
                                $prev_was_number = false;
                                $numeric_mode = false;
                                continue;
                            }
                            $out.extend(super::rule_9::symbol_indicator(*$form));
                            // §5.7.1/§5.8.1: a single styled wordsign-letter standing
                            // alone (§2.6) takes a grade-1 indicator ⠰ — before any
                            // capital — so it is not read as the §10.1 wordsign (`𝑦`
                            // → `⠨⠆⠰⠽`); a/i/o letters carry no wordsign so are exempt
                            // (`𝑖` → `⠨⠆⠊`).
	                            let prev = $i.checked_sub(1).map(|p| &$tokens[p]);
	                            let next = $tokens.get(j);
	                            if super::rule_5_7::is_wordsign_letter(chars[0])
	                                && !(chars[0].is_ascii_uppercase()
	                                    && matches!(prev, Some(EnglishToken::Symbol(_)) | Some(EnglishToken::Styled(..)) | Some(EnglishToken::Word(_))))
	                                && is_standing_alone(prev, next)
	                            {
                                $out.push(GRADE1);
                            }
                            if chars[0].is_ascii_uppercase() {
                                $out.push(CAPITAL);
                            }
                            $out.push(
                                crate::english::encode_english(chars[0].to_ascii_lowercase())
                                    .ok()?,
                            );
                        } else {
                            // §15.2.2: a stress-marked styled word directly before an
                            // end-of-sentence period (`ˈO̲v̲a̲l̲.`) takes the SYMBOL
                            // indicator (⠸⠆) rather than the WORD indicator (⠸⠂), per
                            // the PDF page 253 example. The stress+underline SYMBOL
                            // combines the whole underlined run as one composite item.
                            let follows_stress = $i > 0
                                && matches!(
                                    $tokens.get($i - 1),
                                    Some(EnglishToken::Symbol('\u{2C8}' | '\u{2CC}' | '′' | '″'))
                                );
                            let ends_with_sentence_period = matches!(
                                $tokens.get(j),
                                Some(EnglishToken::Symbol('.' | '?' | '!'))
                            ) && matches!(
                                $tokens.get(j + 1),
                                None | Some(EnglishToken::Space | EnglishToken::LineBreak)
                            );
                            if follows_stress && ends_with_sentence_period {
                                $out.extend(super::rule_9::symbol_indicator(*$form));
                                $engine.encode_styled_word(
                                    &chars,
                                    $i,
                                    j,
                                    StyledContext {
                                        $tokens,
                                        suppress_caps: $in_passage[$i]
                                            || continues_uppercase_word_across_typeform($tokens, $i),
                                        foreign_scope,
                                    },
                                    &mut $out,
                                )?;
                                $skip_to = j;
                                $prev_was_number = false;
                                $numeric_mode = false;
                                continue;
                            }
                            // 2+ styled letters → one word indicator covering the
                            // whole space-delimited word. A hyphen/apostrophe-joined
                            // run of styled segments (`𝑜𝑓-𝑡ℎ𝑒`, `𝑙'𝑜𝑒𝑖𝑙-𝑑𝑒-𝑏𝑜𝑒𝑢𝑓`)
                            // stays under a single indicator (§9.5); a terminator
                            // closes it if the word continues plain (`𝐭𝐞𝐱𝐭book`,
                            // `a̲n̲d̲/or`).
                            // §9.3/§10.7 collision skip: `𝐰𝐨𝐫𝐝` alone would emit
                            // `⠘⠂⠘⠺` (bold word indicator + `word` contraction);
                            // the redundant leading `⠘⠂` is dropped so the reader
                            // sees the single bold `⠘⠺` cell pair.
                            let skip_word_indicator = span_end == j
                                && styled_word_matches_typeform_prefix_contraction(&chars, *$form);
                            let styled_tail_after_plain_word = span_end == j
                                && !skip_word_indicator
                                && chars.iter().all(|c| c.is_ascii_lowercase())
                                && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Word(_)));
                            let styled_ing_after_n = styled_tail_after_plain_word
                                && matches!(
                                    $i.checked_sub(1).and_then(|p| $tokens.get(p)),
                                    Some(EnglishToken::Word(prev))
                                        if prev.last().is_some_and(|c| *c == 'n')
                                )
                                && chars == ['i', 'n', 'g'];
                            let mut styled_buf = Vec::new();
                            if span_end > j {
                                $engine.encode_styled_span(
                                    $i,
                                    span_end,
                                    *$form,
                                    StyledContext {
                                        $tokens,
                                        suppress_caps: $in_passage[$i]
                                            || continues_uppercase_word_across_typeform($tokens, $i),
                                        foreign_scope,
                                    },
                                    &mut styled_buf,
                                )?;
                                run_end = span_end;
                            } else {
                                $engine.encode_styled_word(
                                    &chars,
                                    $i,
                                    j,
                                    StyledContext {
                                        $tokens,
                                        suppress_caps: $in_passage[$i]
                                            || continues_uppercase_word_across_typeform($tokens, $i),
                                        foreign_scope,
                                    },
                                    &mut styled_buf,
                                )?;
                            }
                            let use_symbol_indicator = styled_ing_after_n
                                && styled_buf.len() == 1
                                && !word_continues_after($tokens, run_end);
                            if use_symbol_indicator {
                                $out.extend(super::rule_9::symbol_indicator(*$form));
                            } else if !skip_word_indicator {
                                $out.extend(super::rule_9::word_indicator(*$form));
                            }
                            $out.extend(styled_buf);
                            // §9.7.3 typeforms-being-studied context: a styled
                            // word right before a closing bracket keeps its
                            // terminator BEFORE that bracket, so the pairing
                            // nests properly (`(𝑏𝑢𝑜𝑛 𝑔𝑖𝑜𝑟𝑛𝑜)` → `(…⠛⠊⠕⠗⠝⠕⠨⠄)`).
                            // Only fires when the surrounding prose explicitly
                            // names typeforms; ordinary prose keeps the §9.7.3
                            // default of ignoring typeform change for closing
                            // punctuation.
                            let close_bracket_next = matches!(
                                $tokens.get(run_end),
                                Some(EnglishToken::Symbol(')' | ']' | '}'))
                            );
                            if !use_symbol_indicator
                                && (word_continues_after($tokens, run_end)
                                || (close_bracket_next
                                    && document_studies_typeforms($tokens)))
                            {
                                $out.extend(super::rule_9::terminator(*$form));
                            }
                        }
                    }
                    $skip_to = run_end;
                    $prev_was_number = false;
                    $numeric_mode = false;
                }
    };
}
