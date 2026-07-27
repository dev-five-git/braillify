macro_rules! encode_space_arm {
    ($tokens:ident, $out:ident, $prev_was_number:ident, $numeric_mode:ident, $skip_to:ident, $line_mode_active:ident, $preserve_spatial_newlines:ident, $flatten_line_layout:ident, $spatial_grade1_passage:ident, $poem_linear_context:ident, $collapse_prose_double_space:ident, $skip_flattened_line_indent:ident, $numeric_separator_count:ident, $i:ident) => {
                {
                    if $poem_linear_context && $out.is_empty() {
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if $flatten_line_layout && $out.is_empty() {
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if $skip_flattened_line_indent {
                        $prev_was_number = false;
                        $numeric_mode = false;
                        continue;
                    }
                    if matches!(
                        (
                            $i.checked_sub(1).and_then(|p| $tokens.get(p)),
                            $tokens.get($i + 1)
                        ),
                        (Some(EnglishToken::Symbol('_')), Some(EnglishToken::Symbol('_')))
                    ) {
                        $prev_was_number = false;
                        $numeric_mode = false;
                        $line_mode_active = false;
                        continue;
                    }
                    if is_numeric_space($tokens, $i) {
                        $numeric_separator_count += 1;
                        $skip_to = encode_following_number_as_numeric_space(
                            $tokens,
                            $i,
                            &mut $out,
                            $numeric_separator_count == 6,
                        )?;
                        $prev_was_number = true;
                        $numeric_mode = true;
                        $line_mode_active = false;
                        continue;
                    }
                    // §15.2.1 scansion notation (`. - . - / . . - -`): a space
                    // between two scansion marks (`.`, `-`, `/`) collapses in
                    // braille so the metrical pattern reads as one unbroken run
                    // of `⠲⠤⠸⠌…`. Detected by both flanking tokens being scansion
                    // symbols — a broader (letter-containing) prose context is
                    // left alone.
                    let is_scan = |t: Option<&EnglishToken>| {
                        matches!(t, Some(EnglishToken::Symbol('.' | '-' | '/')))
                    };
	                    if is_scan($i.checked_sub(1).and_then(|p| $tokens.get(p)))
	                        && is_scan($tokens.get($i + 1))
	                    {
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if bibliography_entry_context($tokens)
	                        && matches!($tokens.get($i + 1), Some(EnglishToken::Space))
	                        && matches!(
	                            $i.checked_sub(1).and_then(|p| $tokens.get(p)),
	                            Some(EnglishToken::Symbol('.' | ':') | EnglishToken::Styled('.', _))
	                        )
	                    {
	                        $out.push(SPACE);
	                        $skip_to = $i + 2;
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    // §12.4/§7 prose: collapse a double space between prose
                    // words (`not:  For`) to one cell. A wider gap or column
                    // context (`has_wide_space_run`) is preserved. Skip the
                    // collapse inside a Korean context — Korean tests use the
                    // legacy path but land here for Latin-embedded inputs like
                    // `1in는 2.54cm이다.`, where the token stream contains no
                    // multi-space runs and this branch would be a no-op anyway.
	                    if $collapse_prose_double_space
	                        && matches!($tokens.get($i + 1), Some(EnglishToken::Space))
	                        && styled_prose_double_space($tokens, $i)
	                    {
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if $preserve_spatial_newlines {
                        let mut end = $i;
                        while matches!($tokens.get(end), Some(EnglishToken::Space)) {
                            end += 1;
                        }
                        if matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Symbol('│')))
                            && matches!($tokens.get(end), Some(EnglishToken::Symbol('│')))
                            && end - $i >= 10
                        {
                            $out.extend(std::iter::repeat_n(SPACE, end - $i - 1));
                            $skip_to = end;
                            $prev_was_number = false;
                            $numeric_mode = false;
                            $line_mode_active = false;
                            continue;
                        }
                        if matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Symbol('┐' | '┘')))
                            && matches!($tokens.get(end), Some(EnglishToken::Symbol('┌' | '└')))
                            && end - $i >= 5
                        {
                            $out.extend(std::iter::repeat_n(SPACE, end - $i + 1));
                            $skip_to = end;
                            $prev_was_number = false;
                            $numeric_mode = false;
                            $line_mode_active = false;
                            continue;
                        }
                        if $spatial_grade1_passage
                            && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Symbol('╲' | '╱')))
                            && matches!($tokens.get(end), Some(EnglishToken::Symbol('│')))
                        {
                            $out.extend(std::iter::repeat_n(SPACE, end - $i - 1));
                            $skip_to = end;
                            $prev_was_number = false;
                            $numeric_mode = false;
                            $line_mode_active = false;
                            continue;
                        }
                    }
                    if let Some((end, dots)) = wide_table_gap_before_word($tokens, $i) {
                        $out.push(SPACE);
                        $out.extend(std::iter::repeat_n(decode_unicode('⠐'), dots));
                        $out.extend([SPACE, SPACE]);
                        $skip_to = end;
                        $prev_was_number = false;
                        $numeric_mode = false;
                        $line_mode_active = false;
                        continue;
                    }
                    if let Some(end) = styled_column_gap($tokens, $i) {
                        $out.push(SPACE);
                        $skip_to = end;
                        $prev_was_number = false;
                        $numeric_mode = false;
                        $line_mode_active = false;
                        continue;
                    }
                    if let Some((end, dots)) = wide_table_gap_before_number($tokens, $i) {
                        // §16.5.1: a wide blank run between a row label and a numeric
                        // column is guide-dot space. Keep a blank cell before and
                        // after the dot-5 run so the columns remain visually aligned.
                        // For a 3+ digit number (dots=0), the gap collapses to a single
                        // blank cell — the wide number already reaches the column edge.
                        $out.push(SPACE);
                        if dots > 0 {
                            $out.extend(std::iter::repeat_n(decode_unicode('⠐'), dots));
                            $out.extend(std::iter::repeat_n(SPACE, (end - $i).saturating_sub(5).max(2)));
                        }
                        $skip_to = end;
                        $prev_was_number = false;
                        $numeric_mode = false;
                        $line_mode_active = false;
                        continue;
                    }
                    if $preserve_spatial_newlines
                        && $line_mode_active
                        && matches!(
                            $i.checked_sub(1).and_then(|p| $tokens.get(p)),
                            Some(EnglishToken::Symbol('┼'))
                        )
                    {
                        let mut end = $i;
                        while matches!($tokens.get(end), Some(EnglishToken::Space)) {
                            end += 1;
                        }
                        if matches!($tokens.get(end), Some(EnglishToken::Symbol(c)) if super::rule_16::is_line_char(*c)) {
                            $out.push(SPACE);
                            $skip_to = end;
                            $prev_was_number = false;
                            $numeric_mode = false;
                            $line_mode_active = false;
                            continue;
                        }
                    }
                    $out.push(SPACE);
                    $prev_was_number = false;
                    $numeric_mode = false;
                    $numeric_separator_count = 0;
                    $line_mode_active = false;
                }
    };
}
