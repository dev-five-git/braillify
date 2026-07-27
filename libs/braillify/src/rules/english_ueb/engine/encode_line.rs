macro_rules! encode_line_arm {
    ($tokens:ident, $out:ident, $prev_was_number:ident, $numeric_mode:ident, $line_mode_active:ident, $skip_flattened_line_indent:ident, $i:ident, $c:ident) => {
                {
                    $skip_flattened_line_indent = false;
                    // §16.2 horizontal line mode: a run of two or more box-drawing
                    // characters opens with the indicator `⠐⠒` (whose `⠒` is the
                    // first solid segment, so a leading `─` folds into it); each
                    // further char maps to its segment/corner/crossing cell. A lone
                    // box char never reaches here (the guard requires a neighbour),
                    // so a mathematical `≡`/`─` keeps its legacy meaning.
                    let prev_is_line = $i.checked_sub(1).is_some_and(|p| {
                        matches!(&$tokens[p], EnglishToken::Symbol(s) if super::rule_16::is_line_char(*s) && (!super::rule_16::is_spatial_segment(*s) || !super::rule_16::is_spatial_segment(*$c)))
                    });
			                    if prev_is_line || $line_mode_active {
			                        // §16.2.4 distinctive markers (e.g. `▭`) take a multi-cell form
			                        // (`⠯⠭⠭⠭⠽`) inside a line; plain segments/corners take one cell.
			                        let second_short_shaft_cell = *$c == super::rule_16::SIMPLE_SEGMENT
			                            && $i.checked_sub(2).is_none_or(|p| {
			                                !matches!($tokens.get(p), Some(EnglishToken::Symbol(s)) if super::rule_16::is_line_char(*s))
			                            })
			                            && matches!($tokens.get($i + 1), Some(EnglishToken::Symbol('┼' | '→')));
			                        let final_wide_box_segment = *$c == super::rule_16::SIMPLE_SEGMENT
			                            && matches!($tokens.get($i + 1), Some(EnglishToken::Symbol('┐' | '┘')))
			                            && {
			                                let mut run = 1usize;
			                                let mut p = $i;
			                                while p > 0
			                                    && matches!($tokens.get(p - 1), Some(EnglishToken::Symbol(s)) if *s == super::rule_16::SIMPLE_SEGMENT)
			                                {
			                                    run += 1;
			                                    p -= 1;
			                                }
			                                run >= 6
			                            };
			                        if second_short_shaft_cell || final_wide_box_segment {
					                        } else if let Some(cells) = super::rule_16::line_marker_cells(*$c) {
					                            $out.extend(cells);
			                        } else {
			                            $out.push(super::rule_16::line_segment(*$c)?);
			                        }
			                    } else {
			                        if *$c == '\u{251C}' {
			                            $out.push(decode_unicode('⠸'));
			                            $out.push(decode_unicode('⠐'));
			                        } else {
			                            $out.push(decode_unicode('⠐'));
			                            if !matches!(*$c, '\u{250C}' | '\u{2514}') {
			                                $out.push(decode_unicode('⠒'));
			                            }
			                        }
			                        if *$c != super::rule_16::SIMPLE_SEGMENT
			                            && (*$c != '\u{2550}' || horizontal_run_reaches_arrow($tokens, $i))
			                            && !matches!(*$c, '\u{250C}' | '\u{2514}' | '\u{251C}')
			                        {
	                            if let Some(cells) = super::rule_16::line_marker_cells(*$c) {
	                                $out.extend(cells);
                            } else {
                                $out.push(super::rule_16::line_segment(*$c)?);
                            }
                        }
                    }
                    $line_mode_active = true;
                    // §16.2.5: a horizontal line interrupted by text mid-line takes
                    // the line mode terminator `⠄` before the text (a following space
                    // ends the line naturally, needing none). The next box run
                    // re-opens with its own `⠐⠒` indicator (§16.4.2).
                    if matches!($tokens.get($i + 1), Some(EnglishToken::Word(_))) {
                        $out.push(decode_unicode('⠄'));
                    }
                    $prev_was_number = false;
                    $numeric_mode = false;
                }
    };
}
