macro_rules! encode_word_arm {
    ($engine:expr, $tokens:ident, $explicit_english:ident, $out:ident, $prev_was_number:ident, $numeric_mode:ident, $skip_to:ident, $line_mode_active:ident, $grade1_passage:ident, $cap_start_grade1:ident, $in_passage:ident, $escaped_code:ident, $regex_listing:ident, $spanish_foreign:ident, $foreign_passage:ident, $scansion_stress_context:ident, $early_english:ident, $spatial_grade1_passage:ident, $skip_flattened_line_indent:ident, $i:ident, $chars:ident) => {
		                {
		                    $skip_flattened_line_indent = false;
		                    $line_mode_active = false;
		                    if regex_char_class_word($tokens, $i, $chars, &$regex_listing, &mut $out)? {
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if $escaped_code[$i] {
                        // RUEB 2024 §7.6.7 program snippets: text inside escaped
                        // quotes is code, so quote disambiguation uses two-cell
                        // quote signs and the intervening words are transcribed
                        // letter-for-letter, not contracted (`\“Remember ...\”`).
                        encode_literal_word($chars, &mut $out)?;
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if $spatial_grade1_passage
                        && $chars.len() == 1
                        && matches!($chars[0], 'X' | 'O')
                    {
                        $out.push(crate::english::encode_english($chars[0].to_ascii_lowercase()).ok()?);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if $grade1_passage.is_none()
                        && let Some(span) = grade1_passage_span($tokens, $i)
                    {
				                        $out.extend(std::iter::repeat_n(GRADE1, span.indicator_cells));
				                        $grade1_passage = Some(span);
				                    }
			                    if isolated_shape_circle($tokens, $i, $chars) {
		                        $out.extend([
		                            GRADE1,
		                            decode_unicode('⠫'),
		                            decode_unicode('⠿'),
		                        ]);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
                    if let Some(EnglishToken::Symbol('\u{035e}')) = $tokens.get($i + 1)
                        && let Some(EnglishToken::Word(right)) = $tokens.get($i + 2)
                    {
                        // §4.2.5: a single modifier applying to multiple letters is
                        // written before braille grouping indicators. U+035E is
                        // anchored between two print letters, so only the adjacent
                        // pair is grouped; surrounding letters remain outside.
                        let (&left_last, left_prefix) = $chars.split_last()?;
                        let (&right_first, right_rest) = right.split_first()?;
                        for &c in left_prefix {
                            push_literal_letter(c, &mut $out)?;
                        }
                        let grouped = [left_last, right_first];
                        emit_group_modifier('\u{0304}', &grouped, &mut $out)?;
                        for &c in right_rest {
                            push_literal_letter(c, &mut $out)?;
                        }
                        $skip_to = $i + 3;
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if let Some(EnglishToken::Symbol('\u{0361}' | '\u{0336}')) = $tokens.get($i + 1)
                        && let Some(EnglishToken::Word(right)) = $tokens.get($i + 2)
                    {
	                        // §4.3.1/§4.3.3: joined letters take the ligature indicator
	                        // between the affected letters; the joined pair is a
	                        // contraction boundary, so encode literal letters.
                        emit_ligature_between($chars, right, &mut $out)?;
                        $skip_to = if matches!($tokens.get($i + 3), Some(EnglishToken::Symbol('\u{0336}'))) {
                            $i + 4
                        } else {
                            $i + 3
                        };
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
	                    if let Some(EnglishToken::Symbol(mark)) = $tokens.get($i + 1)
	                        && combining_modifier_cells(*mark).is_some()
	                    {
	                        // §4.2.1: combining mark printed after a letter is placed
	                        // before that letter in braille.
	                        emit_word_with_modifier_on_last($chars, *mark, &mut $out)?;
	                        $skip_to = $i + 2;
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
			                    if $early_english
	                        || (!$scansion_stress_context && $chars.iter().any(|c| {
	                            matches!(
	                                c,
                                'þ' | 'Þ'
                                    | 'ð'
                                    | 'Ð'
                                    | 'ȝ'
                                    | 'Ȝ'
                                    | 'ƿ'
                                    | 'Ƿ'
                                    | 'ǣ'
                                    | 'Ǣ'
                                    | 'ē'
                                    | 'Ē'
                                    | 'ō'
                                    | 'Ō'
                                    | 'ū'
                                    | 'Ū'
                                    | 'ȳ'
                                    | 'Ȳ'
	                            )
	                        }))
                    {
                        let early_word: String = $chars.iter().flat_map(|c| c.to_lowercase()).collect();
                        if let Some(cells) = super::rule_12::middle_english_contract_word(&early_word) {
                            $out.extend(cells);
                        } else {
                            $out.extend(super::rule_12::encode_uncontracted_word($chars)?);
                        }
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    // §12.3 without explicit early-English signal: apply the ME
                    // contracted spelling for words whose form is unambiguously
                    // archaic (never modern English) OR when a following
                    // `(modern-spelling)` gloss marks the word as Middle English.
                    let lower_word_check: String =
                        $chars.iter().flat_map(|c| c.to_lowercase()).collect();
                    let has_modern_gloss = matches!($tokens.get($i + 1), Some(EnglishToken::Space))
                        && matches!($tokens.get($i + 2), Some(EnglishToken::Symbol('(')))
                        && matches!($tokens.get($i + 3), Some(EnglishToken::Word(_)))
                        && $tokens
                            .iter()
                            .skip($i + 4)
                            .take(2)
                            .any(|t| matches!(t, EnglishToken::Symbol(')')));
                    let use_middle_english = super::rule_12::is_archaic_only_spelling(&lower_word_check)
                        || (has_modern_gloss
                            && super::rule_12::middle_english_contract_word(&lower_word_check)
                                .is_some());
                    if use_middle_english
                        && let Some(cells) = super::rule_12::middle_english_contract_word(&lower_word_check)
                    {
                        $out.extend(cells);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
	                    let prev = $i.checked_sub(1).map(|p| &$tokens[p]);
                    let next = $tokens.get($i + 1);
	                    if (matches!(next, Some(EnglishToken::Symbol('=')))
	                        || matches!(prev, Some(EnglishToken::Symbol('='))))
	                        && $chars.iter().any(|c| c.is_uppercase())
	                        && $chars.iter().any(|c| c.is_lowercase())
	                    {
                        encode_literal_word($chars, &mut $out)?;
                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
		                    if $chars.iter().all(|c| greek_letter_cells(*c).is_some()) {
	                        for &c in $chars {
	                            $out.extend(greek_letter_cells_with_caps(c, $in_passage[$i])?);
	                        }
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
					                    let lower_word: String = $chars.iter().flat_map(|c| c.to_lowercase()).collect();
					                    if bibliography_foreign_quote_word($tokens, $i) {
					                        match classify_caps($chars)? {
					                            _ if $in_passage[$i] => {}
					                            Caps::None => {}
					                            Caps::Single => $out.push(CAPITAL),
					                            Caps::Word => $out.extend([CAPITAL, CAPITAL]),
					                        }
					                        $out.extend(super::rule_13::encode_uncontracted_word(
					                            &$chars.iter().flat_map(|c| c.to_lowercase()).collect::<Vec<_>>(),
					                            super::rule_13::AccentCode::Ueb,
					                            false,
					                        )?);
					                        $prev_was_number = false;
					                        $numeric_mode = false;
					                        continue;
					                    }
					                    if bibliography_con_word($chars, $tokens, $i) {
					                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
					                            $out.push(CAPITAL);
					                        }
					                        $out.push(decode_unicode('⠒'));
					                        let tail: Vec<char> = $chars[3..]
					                            .iter()
					                            .flat_map(|c| c.to_lowercase())
					                            .collect();
					                        $out.extend(super::rule_10_9::encode_with_optional_longer_shortforms(
					                            &tail,
					                            &$engine.contractions,
					                            false,
					                            false,
					                            true,
					                        )?);
					                        $prev_was_number = false;
					                        $numeric_mode = false;
					                        continue;
					                    }
					                    if is_pronunciation_or_letter_label_context($tokens, $i) {
					                        // UEB §5.7.1 and §5.8.1 in a §9 example: a
					                        // standing-alone wordsign letter used as a print
					                        // letter label keeps grade 1 before its capital
					                        // indicator (`M is for 𝑀other`).  Pure §5.11
					                        // grade-1 teaching examples have no styled token and
					                        // remain unprefixed.
					                        if $tokens.iter().any(|t| matches!(t, EnglishToken::Styled(..)))
					                            && $chars.len() == 1
					                            && super::rule_5_7::is_wordsign_letter($chars[0])
					                        {
					                            $out.push(GRADE1);
					                        }
					                        $out.extend(encode_letters_literal($chars)?);
					                        $prev_was_number = false;
					                        $numeric_mode = false;
				                        continue;
				                    }
				                    if lower_word == "ins"
		                        && ($prev_was_number
		                            || $numeric_mode
		                            || matches!(prev, Some(EnglishToken::Number(_))))
	                    {
	                        $out.extend([
	                            GRADE1,
	                            decode_unicode('⠊'),
	                            decode_unicode('⠝'),
	                            decode_unicode('⠎'),
	                        ]);
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
                    let follows_numeric = $prev_was_number || $numeric_mode;
                    let ascii_word = $chars.iter().all(|c| c.is_ascii_alphabetic());
                    if follows_numeric && ascii_word {
                        if $numeric_mode
                            && $chars.len() == 2
                            && $chars.iter().all(|c| c.is_ascii_uppercase())
                            && matches!(prev, Some(EnglishToken::Number(_)))
                            && matches!($i.checked_sub(2).and_then(|p| $tokens.get(p)), Some(EnglishToken::Symbol('.')))
                            && matches!($i.checked_sub(3).and_then(|p| $tokens.get(p)), Some(EnglishToken::Number(_)))
                        {
                            // §3.24.1: a subscripted unit letter immediately after a
                            // decimal number remains in numeric grade-1 mode, so only
                            // the level indicator is needed between the capital letters.
                            $out.push(CAPITAL);
                            $out.push(crate::english::encode_english($chars[0].to_ascii_lowercase()).ok()?);
                            $out.push(super::rule_3_24::ScriptKind::Subscript.indicator());
                            $out.push(CAPITAL);
                            $out.push(crate::english::encode_english($chars[1].to_ascii_lowercase()).ok()?);
                        } else if $chars.len() >= 2 && $chars.iter().all(|c| c.is_ascii_uppercase()) {
                            $out.extend([CAPITAL, CAPITAL]);
                            for &c in $chars {
                                $out.push(crate::english::encode_english(c.to_ascii_lowercase()).ok()?);
                            }
		                        } else {
		                            if $chars
		                                .first()
		                                .is_some_and(|c| c.is_ascii_lowercase() && ('a'..='j').contains(c))
		                            {
		                                $out.push(GRADE1);
		                            }
		                            encode_literal_word($chars, &mut $out)?;
		                        }
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
			                    // §13.2.1/§13.6: a whole-sentence foreign passage takes
			                    // the foreign-accent path first, so an accented word
			                    // (`collège`, `Ménard`) uses the §13.6 foreign accent cell
			                    // (`⠮` for è, `⠿` for é) rather than the §4.2 UEB accent
			                    // (`⠘⠡`, `⠘⠌`). This check must precede the modified-letter
			                    // path below because that path emits UEB accents.
					                    if $foreign_passage
                                        && let Some(cells) = super::rule_13::encode_uncontracted_word(
                                            $chars,
                                            super::rule_13::AccentCode::Foreign,
                                            $spanish_foreign,
                                        )
                                    {
					                        $out.extend(cells);
					                        $prev_was_number = false;
					                        $numeric_mode = false;
					                        continue;
					                    }
								                    if classify_caps($chars).is_some()
					                        && $chars.iter().any(|c| super::rule_4::is_modified_letter(*c))
					                    {
		                        if let Some(cells) = encode_pdf_abbreviation($chars) {
		                            $out.extend(cells);
		                        } else {
		                            match classify_caps($chars)? {
		                                _ if $in_passage[$i] => {}
		                                Caps::None => {}
		                                Caps::Single => $out.push(CAPITAL),
		                                Caps::Word => $out.extend([CAPITAL, CAPITAL]),
		                            }
		                            encode_modified_word(
		                                &$engine.contractions,
		                                $chars,
		                                word_initial_boundary(prev),
		                                restricted_prefix_boundary(prev),
		                                &mut $out,
		                            )?;
		                        }
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
			                    if repeated_initial_letter_stammer($chars)
			                        && !in_grade1_passage($i, $grade1_passage)
			                    {
			                        $out.push(GRADE1);
			                        $out.extend(encode_letters_literal($chars)?);
			                        $prev_was_number = false;
			                        $numeric_mode = false;
			                        continue;
			                    }
		                    if after_repeated_stammer_prefix($tokens, $i, &lower_word)
		                        && !in_grade1_passage($i, $grade1_passage)
		                    {
	                        if let Some(first) = $chars.first() {
	                            $out.push(crate::english::encode_english(first.to_ascii_lowercase()).ok()?);
	                        }
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
		                    if abbreviating_letters($tokens, $i, &lower_word) {
		                        $out.extend(encode_letters_literal($chars)?);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if dash_bounded_strong_sequence_literal($tokens, $i, &lower_word) {
		                        $out.extend(encode_letters_literal($chars)?);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if midword_parenthesized_ing($tokens, $i, &lower_word) {
		                        $out.push(decode_unicode('⠬'));
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if measurement_in_abbreviation($tokens, $i, &lower_word) {
		                        $out.extend([decode_unicode('⠊'), decode_unicode('⠝')]);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if lower_word == "in" && number_hyphen_in_abbreviation($tokens, $i) {
		                        $out.push(decode_unicode('⠔'));
		                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
			                    if lower_word == "fr"
		                        && matches!(next, Some(EnglishToken::Symbol('.')))
		                        && matches!($tokens.get($i + 2), Some(EnglishToken::Symbol('.')))
		                        && matches!($tokens.get($i + 3), Some(EnglishToken::Symbol('.')))
		                    {
		                        $out.extend(encode_letters_literal($chars)?);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
                    if lower_word == "mustn"
                        && matches!(next, Some(EnglishToken::Symbol('\'')))
                        && matches!($tokens.get($i + 2), Some(EnglishToken::Word(w)) if w.len() == 1 && w[0].eq_ignore_ascii_case(&'t'))
                    {
		                        $out.extend([decode_unicode('⠍'), decode_unicode('⠌'), decode_unicode('⠝')]);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if let Some(grade1_count) = shortform_confusion_grade1_count(&lower_word, $chars) {
		                        $out.extend(std::iter::repeat_n(GRADE1, grade1_count));
		                        $out.extend(encode_letters_literal($chars)?);
	                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
	                    }
		                    if shortform_abbreviation_literal(&lower_word, $chars) {
		                        $out.extend(encode_letters_literal($chars)?);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
			                    if stammer_fragment_literal($tokens, $i, &lower_word)
			                        && !in_grade1_passage($i, $grade1_passage)
			                    {
		                        $out.extend(encode_letters_literal($chars)?);
		                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
		                    if lower_word == "en" && foreign_en_spells_letters(prev, next) {
                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
                            $out.push(CAPITAL);
                        }
                        $out.extend([decode_unicode('⠑'), decode_unicode('⠝')]);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if lower_word == "wouldn"
                        && matches!(next, Some(EnglishToken::Symbol('\'')))
                        && matches!($tokens.get($i + 2), Some(EnglishToken::Word(w)) if w.len() == 1 && w[0].eq_ignore_ascii_case(&'t'))
                    {
                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
                            $out.push(CAPITAL);
                        }
                        // §10.9 shortform `would` + suffix `n't`: keep `would` as `wd`
                        // and append the suffix letters around the apostrophe.
                        $out.extend([decode_unicode('⠺'), decode_unicode('⠙'), decode_unicode('⠝')]);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
		                    if syllable_alphabetic_wordsign_literal($tokens, $i, &lower_word) {
	                        let lower: Vec<char> = $chars.iter().flat_map(|c| c.to_lowercase()).collect();
	                        $out.extend(super::rule_10_9::encode_with_optional_longer_shortforms(
	                            &lower,
	                            &$engine.contractions,
	                            word_initial_boundary(prev),
	                            false,
	                            true,
	                        )?);
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if lower_word == "in" && spell_line_division_in($tokens, $i, &lower_word) {
                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
                            $out.push(CAPITAL);
                        }
                        $out.extend([decode_unicode('⠊'), decode_unicode('⠝')]);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
	                    if lower_word == "in" && standalone_hyphen_in($tokens, $i) {
	                        $out.extend([decode_unicode('⠊'), decode_unicode('⠝')]);
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
			                    if lower_word == "in" && spell_lower_in_for_preference($tokens, $i) {
			                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
			                            $out.push(CAPITAL);
			                        }
			                        $out.extend([decode_unicode('⠊'), decode_unicode('⠝')]);
	                        $prev_was_number = false;
		                        $numeric_mode = false;
			                        continue;
			                    }
				                    if let Some(cells) = dot_delimited_domain_word_cells($tokens, $i, &lower_word) {
				                        $out.extend(cells);
				                        $prev_was_number = false;
				                        $numeric_mode = false;
				                        continue;
				                    }
		                    if lower_word == "in" && spell_in_for_lower_wordsign_limit($tokens, $i) {
		                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
		                            $out.push(CAPITAL);
		                        }
		                        $out.extend([decode_unicode('⠊'), decode_unicode('⠝')]);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
			                    if let Some(cells) = lower_sequence_before_apostrophe_cells(
			                        $chars,
			                        &$engine.contractions,
			                        prev,
			                        next,
			                        false,
			                    ) {
		                        encode_lower_sequence_word($chars, &cells, &mut $out)?;
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
	                    if lower_word == "where"
                        && matches!(next, Some(EnglishToken::Symbol('\'' | '’')))
                    {
                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
                            $out.push(CAPITAL);
                        }
                        $out.extend([
                            decode_unicode('⠱'),
                            decode_unicode('⠻'),
                            decode_unicode('⠑'),
                        ]);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if lower_word == "enough"
                        && matches!(next, Some(EnglishToken::Symbol('–' | '—')))
                        && matches!($tokens.get($i + 2), Some(EnglishToken::LineBreak))
                        && !matches!(prev, Some(EnglishToken::Symbol('(')))
                    {
                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
                            $out.push(CAPITAL);
                        }
                        let lower: Vec<char> = $chars.iter().flat_map(|c| c.to_lowercase()).collect();
                        $out.extend(super::rule_10_9::encode_with_longer_shortforms(
                            &lower,
                            &$engine.contractions,
                            word_initial_boundary(prev),
                        )?);
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
	                    if lower_word == "enough"
	                        && enough_followed_by_upper_dot_sequence($tokens, $i)
	                    {
	                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
	                            $out.push(CAPITAL);
	                        }
	                        $out.push(decode_unicode('⠢'));
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if lower_word == "enough" && enough_followed_by_sentence_close($tokens, $i) {
	                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
	                            $out.push(CAPITAL);
	                        }
	                        $out.push(decode_unicode('⠢'));
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if lower_word == "enough"
	                        && touches_hyphen_or_line_break(prev, next)
	                        && !lower_contact_after_division_word(next)
	                    {
                        if matches!(classify_caps($chars), Some(Caps::Single | Caps::Word)) {
                            $out.push(CAPITAL);
                        }
                        $out.push(decode_unicode('⠢'));
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }

		                    let standing_alone = (super::standing_alone::is_standing_alone_at($tokens, $i)
		                        || transcriber_note_ends_at($tokens, $i, true)
	                        || closing_transcriber_note_starts_at($tokens, $i + 1)
	                        || closing_transcriber_note_after_transparent_suffix($tokens, $i))
	                        && !continues_across_bracket($tokens, $i);
                    // §6.5: a lowercase letter a–j immediately after a number needs
                    // the grade-1 indicator ⠰ so it is not misread as a digit.
                    let numeric_punctuation_before_word = matches!(prev, Some(EnglishToken::Symbol('.' | ',')))
                        && $i.checked_sub(2)
                            .is_some_and(|p| matches!($tokens.get(p), Some(EnglishToken::Number(_))));
                    let after_number_grade1 = ($prev_was_number || numeric_punctuation_before_word)
                        && $chars
                            .first()
                            .is_some_and(|c| c.is_ascii_lowercase() && ('a'..='j').contains(c));
                    // §5.7.1: a single wordsign-letter standing alone (§2.6) takes a
                    // grade-1 indicator ⠰ so it is not read as the wordsign; §5.8.1
                    // places it before any capital. Full rule in `rule_5_7`.
	                    // §10.12.15: a letter-by-letter spelled run (`w-i-n-d-o-w`) takes
	                    // one grade-1 *passage* ⠰⠰ at its first letter; its members then
	                    // suppress the per-letter grade-1 ⠰.
	                    let spelled_run = spelled_letter_run($tokens, $i);
	                    let initialism_run = hyphenated_initialism_run($tokens, $i);
		                    if !in_grade1_passage($i, $grade1_passage)
		                        && (matches!(spelled_run, Some((start, _)) if start == $i)
		                            || matches!(initialism_run, Some((start, _)) if start == $i))
		                    {
		                        $out.extend([GRADE1, GRADE1]);
		                    }
			                    let letter_grade1 = !$cap_start_grade1
			                        && spelled_run.is_none()
			                        && initialism_run.is_none()
			                        && !in_grade1_passage($i, $grade1_passage)
			                        && (super::rule_5_7::needs_grade1_indicator($tokens, $i, $explicit_english)
	                            || ($chars.len() == 1
	                                && $chars[0].is_uppercase()
	                                && super::rule_5_7::is_wordsign_letter($chars[0])
	                                && matches!(next, Some(EnglishToken::Symbol('!')))));
			                    if after_number_grade1 || letter_grade1 || apostrophe_wrapped_letter($tokens, $i, $chars) {
		                        $out.push(GRADE1);
		                    }
		                    if !$foreign_passage
		                        && document_all_words($tokens).len() >= 3
		                        && $chars.len() >= 5
		                        && $chars.iter().all(|c| c.is_ascii_alphabetic())
		                        && classify_caps($chars).is_some()
		                        && !super::pronunciation::cmudict::is_recorded_word(&lower_word)
		                        && !domain_component_context($tokens, $i)
		                    {
		                        // UEB §13.2.3: anglicised Roman-script loan/proper words
		                        // in English context keep UEB contractions.  CMU does
		                        // not record many such words (`Ferhadija`, `pancetta`,
		                        // `pensione`), so route them through a §13.2.3 mode
		                        // rather than the ordinary English shortform whitelist.
		                        match classify_caps($chars)? {
		                            _ if $in_passage[$i] => {}
		                            Caps::None => {}
		                            Caps::Single => $out.push(CAPITAL),
		                            Caps::Word => $out.extend([CAPITAL, CAPITAL]),
		                        }
		                        let lower_chars: Vec<char> =
		                            $chars.iter().flat_map(|c| c.to_lowercase()).collect();
		                        $out.extend(super::rule_10_9::encode_anglicised_word(
		                            &lower_chars,
		                            &$engine.contractions,
		                            word_initial_boundary(prev),
		                            restricted_prefix_boundary(prev),
		                        )?);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
	                    let shortform_usable =
	                        standing_alone && !matches!(next, Some(EnglishToken::Symbol('@' | '/')));
                    // §10.5 lower wordsigns need a stricter boundary than §10.1/§10.2.
                    let mut lower_usable = standing_alone && lower_wordsign_usable(prev, next);
                    // §10.5.2: "enough's" keeps the wordsign (its interior apostrophe is
                    // "standing alone" per §2.6.4) — an explicit exception to the
                    // lower-dot-contact bar that spells out his'/was'/be'.
                    if !lower_usable
                        && $chars
                            .iter()
                            .map(|c| c.to_ascii_lowercase())
                            .eq("enough".$chars())
                        && matches!(next, Some(EnglishToken::Symbol('\'')))
                        && matches!($tokens.get($i + 2),
                            Some(EnglishToken::Word(w)) if w.len() == 1 && w[0].eq_ignore_ascii_case(&'s'))
                    {
                        lower_usable = true;
                    }
		                    $engine.encode_word(
		                        $chars,
		                        WordContext {
	                            standing_alone,
	                            upper_usable: standing_alone
	                                && !matches!(prev, Some(EnglishToken::Symbol('/')))
	                                && !matches!(next, Some(EnglishToken::Symbol('/'))),
                            shortform_usable: shortform_usable
                                && !in_grade1_passage($i, $grade1_passage),
                            // §10.9.3: longer-word shortforms (`brl` in `Brailletype`,
                            // `af` in `afterwards`) require the WHOLE longer word to be
                            // "standing alone" (§2.6). If the following token is a
                            // non-transparent §2.6.3 symbol (e.g. ®, ™) the word is not
                            // standing alone and the shortform must not be applied.
		                            allow_longer_shortforms: !next_breaks_standing_alone(next)
		                                && !domain_component_context($tokens, $i)
		                                && !solidus_component_context($tokens, $i),
	                            lower_usable: lower_usable && !in_grade1_passage($i, $grade1_passage),
	                            suppress_caps: $in_passage[$i],
                            word_initial: word_initial_boundary(prev),
                            restricted_prefix_boundary: restricted_prefix_boundary(prev),
                            digit_adjacent: matches!(prev, Some(EnglishToken::Number(_)))
                                || matches!(next, Some(EnglishToken::Number(_))),
                        },
                        &mut $out,
                    )?;
                    $prev_was_number = false;
                    $numeric_mode = false;
                }
    };
}
