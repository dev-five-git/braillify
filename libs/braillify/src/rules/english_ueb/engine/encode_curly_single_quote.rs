macro_rules! encode_curly_single_quote_arm {
    ($tokens:ident, $out:ident, $prev_was_number:ident, $numeric_mode:ident, $sq_roles:ident, $i:ident) => {
	                {
                    // §7.6 curly single quotation mark vs apostrophe, resolved by
                    // the matched-pair analysis in `single_quote_roles`: an opening
                    // mark → ⠠⠦, a closing mark → ⠠⠴, an apostrophe → ⠄. The straight
                    // `'` is ambiguous and stays an apostrophe on the default path.
                    match $sq_roles[$i] {
                        SingleQuote::Open => {
                            // §7.6.10: a detached opening single quote (a space sits
                            // between it and the text it bounds) takes a grade-1
                            // indicator ⠰ so the ⠠⠦ is not misread.
                            if matches!($tokens.get($i + 1), Some(EnglishToken::Space)) {
                                $out.push(GRADE1);
                            }
                            $out.extend([decode_unicode('⠠'), decode_unicode('⠦')]);
                        }
                        SingleQuote::Close => {
                            // §7.6.10 / §2.6.5: a detached closing single quote takes
                            // the grade-1 indicator when its LEFT side is anchoring
                            // (space/edge) and its RIGHT side ultimately reaches a
                            // §2.6.1 boundary via §2.6.3 transparent symbols. The
                            // stripping loop lets a bracket + edge (`... ')` ) still
                            // trigger the indicator, matching §7.6.10's example
                            // `(‘To be or not ... ’)` → `... ⠰⠠⠴⠐⠜`.
                            let space_before =
                                $i > 0 && matches!($tokens.get($i - 1), Some(EnglishToken::Space));
                            let mut r = $i;
                            while r + 1 < $tokens.len()
                                && matches!(
                                    $tokens.get(r + 1),
                                    Some(EnglishToken::Symbol(
                                        ')' | ']' | '}' | '.' | ',' | ':' | ';'
                                    ))
                                )
                            {
                                r += 1;
                            }
                            let reaches_boundary = matches!(
                                $tokens.get(r + 1),
                                None | Some(EnglishToken::Space | EnglishToken::LineBreak)
                            );
                            if space_before
                                && reaches_boundary
                                && !matches!($tokens.get($i + 1), Some(EnglishToken::Symbol('.')))
                            {
                                $out.push(GRADE1);
                            }
                            $out.extend([decode_unicode('⠠'), decode_unicode('⠴')]);
                        }
	                        SingleQuote::Apostrophe => $out.push(decode_unicode('⠄')),
                    }
                    $prev_was_number = false;
	                    $numeric_mode = false;
	                }
    };
}
