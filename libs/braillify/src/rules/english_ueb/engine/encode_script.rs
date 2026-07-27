macro_rules! encode_script_arm {
    ($tokens:ident, $explicit_english:ident, $out:ident, $prev_was_number:ident, $numeric_mode:ident, $skip_to:ident, $grade1_passage:ident, $i:ident, $c:ident) => {
		                {
                    // §3.24 super/subscript: a digit run following a base takes the
                    // level indicator (`⠔`/`⠢`). The grade-1 indicator `⠰` is added
                    // for a letter base (`B₁₂`, `clarion¹`) but not after a number,
                    // whose numeric mode already covers it (`1682.³`). A *leading*
                    // script (no base, e.g. `¹ clarion` or combinatorics `₇𝑃₂`) or a
                    // non-digit script (`ᵐ`, `⁺`) fails the whole UEB attempt so the
                    // legacy/math path (제18/19항) keeps ownership.
		                    let kind = script_kind(*$c)?;
		                    let base_is_number = match $i.checked_sub(1).map(|p| &$tokens[p]) {
		                        Some(EnglishToken::Word(_)) => $i
		                            .checked_sub(2)
		                            .is_some_and(|p| matches!($tokens.get(p), Some(EnglishToken::Number(_)))),
		                        Some(EnglishToken::Number(_)) => true,
                        // A base reached across a single period (`1682.³`, `knowledge.³`).
                        Some(EnglishToken::Symbol('.')) => {
                            match $i.checked_sub(2).map(|p| &$tokens[p]) {
                                Some(EnglishToken::Word(_)) => false,
                                Some(EnglishToken::Number(_)) => true,
                                _ => return None,
                            }
                        }
		                        None => false,
		                        _ => return None,
		                    };
		                    let mut digits = Vec::new();
		                    let mut letters = Vec::new();
	                        if let Some((_, first)) = super::rule_3_24::script_digit(*$c) {
	                            digits.push(first);
	                        } else if let Some((_, letter)) = script_letter(*$c) {
	                            letters.push(letter);
	                        }
			                    let mut j = $i + 1;
			                    while let Some(EnglishToken::Symbol(sc)) = $tokens.get(j) {
		                        if !super::rule_3_24::is_script_char(*sc) {
		                            break;
		                        }
		                        match (super::rule_3_24::script_digit(*sc), script_letter(*sc)) {
		                            (Some((k, d)), _) if k == kind && letters.is_empty() => digits.push(d),
		                            (None, Some((k, letter))) if k == kind && digits.is_empty() => letters.push(letter),
	                            // a mixed-kind or non-digit script char is unsupported.
	                            _ => return None,
			                        }
			                        j += 1;
			                    }
			                    if !$explicit_english
			                        && !letters.is_empty()
			                        && matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Word(w)) if w.len() == 1 && w[0].is_ascii_lowercase())
			                    {
			                        // §3.24 prose/science notation needs a word/unit base (e.g.
			                        // `massₛᵤₙ`). A lowercase single-letter base (`aₙ`) is
			                        // an ordinary math variable by default; explicit English
			                        // contexts may still force the UEB level indicator.
			                        return None;
			                    }
			                    if !base_is_number && !in_grade1_passage($i, $grade1_passage) {
			                        $out.push(GRADE1);
			                    }
		                    if letters.len() >= 2 {
		                        $out.push(GRADE1);
		                    }
		                    $out.push(kind.indicator());
		                    if !digits.is_empty() {
		                        $out.push(decode_unicode('⠼'));
		                        for d in &digits {
		                            $out.push(super::rule_6::digit_cell(*d)?);
		                        }
		                    } else if letters.len() >= 2 {
		                        $out.push(decode_unicode('⠣'));
		                        for letter in &letters {
		                            $out.push(crate::english::encode_english(*letter).ok()?);
		                        }
		                        $out.push(decode_unicode('⠜'));
		                    } else {
		                        for letter in &letters {
		                            $out.push(crate::english::encode_english(*letter).ok()?);
		                        }
		                    }
                    $skip_to = j;
                    $prev_was_number = false;
                    $numeric_mode = false;
                }
    };
}
