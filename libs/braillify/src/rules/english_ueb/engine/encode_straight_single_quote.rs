macro_rules! encode_straight_single_quote_arm {
    ($tokens:ident, $out:ident, $prev_was_number:ident, $numeric_mode:ident, $i:ident) => {
		                {
		                    if matches!($i.checked_sub(1).and_then(|p| $tokens.get(p)), Some(EnglishToken::Number(_))) {
		                        // §3.15.1: a straight single quote after a number is the
		                        // foot mark/apostrophe cell, not an opening quote.
		                        $out.push(decode_unicode('⠄'));
		                        $prev_was_number = false;
		                        $numeric_mode = false;
		                        continue;
		                    }
		                    if straight_single_quote_exchanged($tokens, $i) {
	                        let role = straight_single_quote_role($tokens, $i);
	                        // §7.6: this exchanged-quote branch is only reached with an
	                        // Open or Close role (straight_single_quote_exchanged returns
	                        // false for an Apostrophe role), so the former Apostrophe arm
	                        // was unreachable.
	                        $out.push(if matches!(role, SingleQuote::Open) {
	                            QUOTE_OPEN
	                        } else {
	                            QUOTE_CLOSE
	                        });
	                    } else {
	                        match straight_single_quote_role($tokens, $i) {
	                            SingleQuote::Open => $out.extend([decode_unicode('⠠'), decode_unicode('⠦')]),
	                            SingleQuote::Close => $out.extend([decode_unicode('⠠'), decode_unicode('⠴')]),
	                            SingleQuote::Apostrophe => $out.push(decode_unicode('⠄')),
	                        }
	                    }
	                    $prev_was_number = false;
	                    $numeric_mode = false;
	                }
    };
}
