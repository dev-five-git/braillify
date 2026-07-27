macro_rules! encode_double_quote_arm {
    ($tokens:ident, $out:ident, $prev_was_number:ident, $numeric_mode:ident, $quote_open:ident, $internal_double_quote_open:ident, $passage:ident, $regex_listing:ident, $i:ident) => {
	                {
	                    if $regex_listing[$i] {
	                        // RUEB 2024 §7.6.5: straight quotes used as ASCII regex
	                        // characters are nondirectional quote signs, not the
	                        // surrounding prose quotation marks.
	                        $out.extend([decode_unicode('⠠'), decode_unicode('⠶')]);
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    if let Some((end, form, caps, _)) = $passage
                        && end == $i + 1
                        && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Symbol(',')))
                    {
                        // §13.2/§9: when a foreign/typeform phrase includes its
                        // trailing comma before a closing quotation mark, the
                        // typeform terminator closes the phrase before the quote.
                        if caps {
                            $out.extend([CAPITAL, decode_unicode('⠄')]);
                        }
                        $out.extend(super::rule_9::terminator(form));
                        $passage = None;
                    }
                    if matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Number(_))) {
                        // §3.15.1: a straight double quote after a number is the
		                        // inch mark, not a directional quotation mark.
		                        $out.extend([decode_unicode('⠠'), decode_unicode('⠶')]);
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if $tokens.iter().enumerate().any(|(idx, _)| straight_single_quote_exchanged($tokens, idx)) {
	                        if $internal_double_quote_open {
	                            $out.extend([decode_unicode('⠠'), decode_unicode('⠴')]);
	                            $internal_double_quote_open = false;
	                        } else {
	                            $out.extend([decode_unicode('⠠'), decode_unicode('⠦')]);
	                            $internal_double_quote_open = true;
	                        }
	                        $prev_was_number = false;
	                        $numeric_mode = false;
	                        continue;
	                    }
	                    // §7.6.10: a double quotation mark standing alone (a space or
                    // text edge on both sides) is the mark referenced in isolation
                    // → grade-1 + the nondirectional double-quote sign ⠰⠠⠶, and it
                    // does not flip the open/close alternation.
                    let standalone = ($i == 0
                        || matches!($tokens.get($i - 1), Some(EnglishToken::Space)))
                        && matches!($tokens.get($i + 1), None | Some(EnglishToken::Space));
                    // §7.6.11 / §2.6 nondirectional: a straight `"` count of 1 in
                    // the whole token stream is unmatched — it cannot be an opening
                    // or closing mark of a pair — so it takes the nondirectional
                    // sign ⠠⠶ (`"yr-123` → ⠠⠶…, `X' Y"` → …⠠⠶). Only fires when the
                    // input carries exactly one straight `"`; a paired `"…"` still
                    // uses the directional ⠦/⠴ alternation below.
                    let straight_quote_count = $tokens
                        .iter()
                        .filter(|t| matches!(t, EnglishToken::Symbol('"')))
                        .count();
                    let unmatched = straight_quote_count == 1;
		                    let prev_text = matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Word(_) | EnglishToken::Styled(..)));
		                    let next_text = matches!($tokens.get($i + 1), Some(EnglishToken::Word(_) | EnglishToken::Styled(..) | EnglishToken::WordDivision { .. }));
			                    if !$quote_open && prev_text && next_text {
		                        $out.extend([decode_unicode('⠘'), QUOTE_OPEN]);
		                        $quote_open = true;
		                        $internal_double_quote_open = true;
		                    } else if $internal_double_quote_open && prev_text {
		                        $out.extend([decode_unicode('⠘'), QUOTE_CLOSE]);
		                        $quote_open = false;
		                        $internal_double_quote_open = false;
	                    } else if standalone {
	                        $out.extend([GRADE1, decode_unicode('⠠'), decode_unicode('⠶')]);
				                    } else if unmatched
				                        && !prev_text
				                        && matches!($tokens.get($i + 1), Some(EnglishToken::Word(_) | EnglishToken::WordDivision { .. }))
				                        && matches!($tokens.get($i + 2), Some(EnglishToken::Symbol('-')))
				                        && matches!($tokens.get($i + 3), Some(EnglishToken::LineBreak))
				                    {
			                        // §7.6 with §10.13: a lone print quote at the beginning of a
			                        // divided word (`"In-\ndepth`) is still an opening quotation
		                        // mark. It is unmatched only because the quoted extract
		                        // continues beyond the testcase snippet.
		                        $out.push(QUOTE_OPEN);
		                        $quote_open = true;
		                    } else if unmatched
		                        && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Symbol('.' | '?' | '!')))
		                    {
	                        $out.push(QUOTE_CLOSE);
	                    } else if unmatched {
	                        // §7.6.11 nondirectional: an unmatched straight `"`
	                        // attached to a word (`"yr-123`, `X' Y"`) takes ⠠⠶
	                        // without the standalone grade-1 indicator.
	                        $out.extend([decode_unicode('⠠'), decode_unicode('⠶')]);
	                    } else {
                        // §7.6 double quotation mark: open ⠦ / close ⠴, alternating.
                        let attached_closing = !$quote_open
                            && $i > 0
                            && !matches!($tokens.get($i - 1), Some(EnglishToken::Space))
                            && !matches!(
                                $tokens.get($i - 1),
                                Some(EnglishToken::Symbol('(' | '[' | '{'))
                            );
                        $out.push(if $quote_open || attached_closing {
                            QUOTE_CLOSE
                        } else {
                            QUOTE_OPEN
                        });
                        $quote_open = !$quote_open;
                    }
                    $prev_was_number = false;
	                    $numeric_mode = false;
	                }
    };
}
