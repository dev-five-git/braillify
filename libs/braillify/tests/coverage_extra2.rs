//! Second batch of coverage-extension snapshot tests.
//!
//! Targets remaining gaps after Wave 1 dead-code removal + binding tests +
//! `lib_coverage_extra_tests.rs` snapshots. Each case pins a single input
//! through `encode` + `encode_to_unicode` and snapshots the result via insta.
//!
//! Each case derives its input from a PDF article (see `note` in case name).
//! NO expected-byte tables — `insta` captures whatever the encoder produces
//! today, and future regressions diff against the committed snapshot.

use rstest::rstest;

fn render(input: &str) -> String {
    let unicode = match braillify::encode_to_unicode(input) {
        Ok(s) => format!("ok: {s:?}"),
        Err(e) => format!("err: {e}"),
    };
    let bytes = match braillify::encode(input) {
        Ok(b) => format!("ok: {b:?}"),
        Err(e) => format!("err: {e}"),
    };
    format!("input  = {input:?}\nunicode = {unicode}\nbytes   = {bytes}\n")
}

#[rstest]
// =====================================================================
// rule_68 — Korean subscript digits (compact notation) — each ₀..₉ arm
// =====================================================================
#[case::sub_digit_0("sub_digit_0", "A₀")]
#[case::sub_digit_1("sub_digit_1", "B₁")]
#[case::sub_digit_2("sub_digit_2", "C₂")]
#[case::sub_digit_3("sub_digit_3", "D₃")]
#[case::sub_digit_4("sub_digit_4", "E₄")]
#[case::sub_digit_5("sub_digit_5", "F₅")]
#[case::sub_digit_6("sub_digit_6", "G₆")]
#[case::sub_digit_7("sub_digit_7", "H₇")]
#[case::sub_digit_8("sub_digit_8", "I₈")]
#[case::sub_digit_9("sub_digit_9", "J₉")]
#[case::sub_multi("sub_multi", "X₁₂")]
#[case::digit_grade_plus("digit_grade_plus", "1++ 등급")]
#[case::digit_grade_minus("digit_grade_minus", "5-- 등급")]
#[case::digit_grade_mixed("digit_grade_mixed", "7+- 등급")]
// =====================================================================
// rule_47 — log / lim (제46·47항)
// =====================================================================
#[case::log_paren_complex("log_paren_complex", "log(x+1)")]
#[case::log_digit_base("log_digit_base", "log_3 x")]
#[case::log_var_base("log_var_base", "log_a x")]
#[case::log_var_base_complex("log_var_base_complex", "log_a (x+y)")]
#[case::log_var_base_with_uppervar("log_var_base_with_uppervar", "log_a (X/Y)")]
#[case::log_paren_base_arg("log_paren_base_arg", "log_(2) x")]
#[case::log_paren_base_with_div("log_paren_base_with_div", "log_(a/b) x")]
#[case::log_no_base_with_div_arg("log_no_base_with_div_arg", "log a/b")]
#[case::log_no_base_with_div_n_n("log_no_base_with_div_n_n", "log 3/4")]
#[case::log_unmatched_paren("log_unmatched_paren", "log_3 (x")]
#[case::lim_arrow_paren("lim_arrow_paren", "lim_(x→0) f(x)")]
#[case::lim_arrow_subscript("lim_arrow_subscript", "lim_{x→0} f(x)")]
// =====================================================================
// rule_18 — superscript variants
// =====================================================================
#[case::super_int_left("super_int_left", "∫^a x")]
#[case::super_sum_left("super_sum_left", "∑^n i")]
#[case::super_prod_left("super_prod_left", "∏^k j")]
#[case::super_forall_left("super_forall_left", "∀^x p(x)")]
#[case::super_func_sin("super_func_sin", "sin^2 x")]
#[case::super_func_cos("super_func_cos", "cos^3 y")]
#[case::super_after_square_close("super_after_square_close", "[a]_i ^2")]
#[case::super_num_slash_super("super_num_slash_super", "10²/⁵")]
#[case::super_num_middledot_super("super_num_middledot_super", "10²·⁵")]
// =====================================================================
// detect.rs — math expression detection gaps
// =====================================================================
#[case::detect_arcsin("detect_arcsin", "arcsinx")]
#[case::detect_arccos("detect_arccos", "arccosy")]
#[case::detect_arctan("detect_arctan", "arctanz")]
#[case::detect_relation_arb("detect_relation_arb", "aRb")]
#[case::detect_letter_slash_letter("detect_letter_slash_letter", "F/N")]
#[case::detect_signed_minus("detect_signed_minus", "-3x")]
#[case::detect_signed_minus_unicode("detect_signed_minus_unicode", "−3x")]
#[case::detect_bracket_letter_op("detect_bracket_letter_op", "[a+b]")]
#[case::detect_bracket_letter_super("detect_bracket_letter_super", "[a²]")]
#[case::detect_year_suffix_a("detect_year_suffix_a", "1998a")]
#[case::detect_year_suffix_a_comma("detect_year_suffix_a_comma", "1998a,")]
#[case::detect_year_suffix_b_semi("detect_year_suffix_b_semi", "1998b;")]
#[case::detect_year_suffix_c_period("detect_year_suffix_c_period", "1998c.")]
#[case::detect_unit_prefix("detect_unit_prefix", "180cm")]
#[case::detect_unit_kg("detect_unit_kg", "5kg")]
// =====================================================================
// symbol_rule — math symbol dispatch edge cases
// =====================================================================
#[case::sigma_with_paren_eq_only("sigma_with_paren_eq_only", "∑(i=1)")]
#[case::sigma_with_complex_inner("sigma_with_complex_inner", "∑(x*y+1)")]
#[case::pi_with_three_args("pi_with_three_args", "∏(1,2,3)")]
#[case::sigma_paren_unmatched("sigma_paren_unmatched", "∑(i=1")]
#[case::norm_in_middle("norm_in_middle", "a‖b‖c")]
#[case::norm_with_op_prefix("norm_with_op_prefix", "+‖x‖")]
#[case::hash_paren_lowercase("hash_paren_lowercase", "\u{FF03}(a)")]
#[case::negation_lower_lower("negation_lower_lower", "a¬b")]
// =====================================================================
// rule_46 — trigonometric functions (제46항)
// =====================================================================
#[case::trig_sin_paren("trig_sin_paren", "sin(x)")]
#[case::trig_cos_paren("trig_cos_paren", "cos(y)")]
#[case::trig_tan_paren("trig_tan_paren", "tan(z)")]
#[case::trig_sin_complex("trig_sin_complex", "sin(x+y)")]
#[case::trig_sin_fraction("trig_sin_fraction", "sin(x/2)")]
#[case::trig_sin_no_paren("trig_sin_no_paren", "sin x")]
#[case::trig_cot("trig_cot", "cot x")]
#[case::trig_sec("trig_sec", "sec x")]
#[case::trig_csc("trig_csc", "csc x")]
#[case::trig_sinh("trig_sinh", "sinh x")]
#[case::trig_cosh("trig_cosh", "cosh y")]
// =====================================================================
// rule_12 / rule_7 — variable + super/sub
// =====================================================================
#[case::var_super_lower_n("var_super_lower_n", "x^n")]
#[case::var_super_lower_k("var_super_lower_k", "y^k")]
#[case::var_sub_lower_i("var_sub_lower_i", "a_i")]
#[case::var_sub_lower_j("var_sub_lower_j", "b_j")]
#[case::var_prime_super("var_prime_super", "f'(x)")]
#[case::var_prime_double_super("var_prime_double_super", "f''(x)")]
// =====================================================================
// rule_19 — subscript variants
// =====================================================================
#[case::sub_simple_v_v("sub_simple_v_v", "a_n")]
#[case::sub_simple_v_d("sub_simple_v_d", "a_5")]
#[case::sub_simple_d_v("sub_simple_d_v", "5_a")]
#[case::sub_complex("sub_complex", "x_{i+1}")]
#[case::sub_double("sub_double", "x_{i_j}")]
#[case::sub_with_super("sub_with_super", "x_i^2")]
// =====================================================================
// English-dominant Korean wrap (token_rules/english_dominant_korean_wrap.rs)
// =====================================================================
#[case::eng_dom_pure_english("eng_dom_pure_english", "Hello World")]
#[case::eng_dom_with_korean("eng_dom_with_korean", "Hello 안녕")]
#[case::eng_dom_long_english("eng_dom_long_english", "The quick brown fox jumps over the lazy dog")]
#[case::eng_dom_short("eng_dom_short", "Hi")]
#[case::eng_dom_mixed_caps("eng_dom_mixed_caps", "HTML and CSS")]
// =====================================================================
// emphasis_ring (token_rules/emphasis_ring.rs)
// =====================================================================
#[case::emph_ring_single("emph_ring_single", "*안녕*")]
#[case::emph_ring_word("emph_ring_word", "**중요**")]
#[case::emph_ring_korean("emph_ring_korean", "이것은 *강조*된 단어입니다")]
// =====================================================================
// rule_33_citation (token_rules)
// =====================================================================
#[case::citation_year_suffix_a("citation_year_suffix_a", "Smith 1998a")]
#[case::citation_year_suffix_b("citation_year_suffix_b", "Jones 2020b")]
// =====================================================================
// rule_73_appendix_placeholder (token_rules)
// =====================================================================
#[case::appendix_x_3("appendix_x_3", "x___3")]
#[case::appendix_placeholder("appendix_placeholder", "___")]
// =====================================================================
// uppercase_passage
// =====================================================================
#[case::uppercase_long("uppercase_long", "ABCDEFG")]
#[case::uppercase_passage_word("uppercase_passage_word", "HELLO WORLD")]
// =====================================================================
// roman_numeral (token_rules)
// =====================================================================
#[case::roman_lowercase("roman_lowercase", "xviii")]
#[case::roman_uppercase("roman_uppercase", "MMXXIII")]
#[case::roman_mixed("roman_mixed", "VIII과 IX")]
// =====================================================================
// quote_attachment (token_rules)
// =====================================================================
#[case::quote_attached("quote_attached", "그는 \"좋다\"고 말했다")]
#[case::quote_single_attached("quote_single_attached", "그는 '아니다'라고 말했다")]
// =====================================================================
// matrix encoder (token_rules/latex_math/matrix.rs)
// =====================================================================
#[case::matrix_with_text(
    "matrix_with_text",
    "$\\begin{matrix} \\text{a} & \\text{b} \\end{matrix}$"
)]
#[case::vmatrix_large(
    "vmatrix_large",
    "$\\begin{vmatrix} a_{11} & a_{12} & a_{13} \\\\ a_{21} & a_{22} & a_{23} \\\\ a_{31} & a_{32} & a_{33} \\end{vmatrix}$"
)]
#[case::bmatrix_long_row(
    "bmatrix_long_row",
    "$\\begin{bmatrix} 1 & 2 & 3 & 4 & 5 \\end{bmatrix}$"
)]
#[case::pmatrix_with_frac(
    "pmatrix_with_frac",
    "$\\begin{pmatrix} \\frac{1}{2} & 0 \\\\ 0 & \\frac{1}{3} \\end{pmatrix}$"
)]
// =====================================================================
// symbol_rule — specific dispatch arm coverage
// =====================================================================
#[case::neg_var_upper("neg_var_upper", "x¬B")]
#[case::neg_upper_upper("neg_upper_upper", "A¬B")]
#[case::sigma_paren_simple("sigma_paren_simple", "∑(i)")]
#[case::sigma_paren_with_eq_and_comma("sigma_paren_with_eq_and_comma", "∑(i=1,n)x")]
#[case::pi_paren_numbers_three("pi_paren_numbers_three", "∏(1,2)x")]
#[case::pi_paren_number_letter("pi_paren_number_letter", "∏(1,n)x")]
#[case::pi_paren_letter_letter("pi_paren_letter_letter", "∏(i,n)x")]
#[case::forall_followed_by_upper("forall_followed_by_upper", "∀X p(X)")]
#[case::exists_followed_by_upper("exists_followed_by_upper", "∃Y q(Y)")]
#[case::forall_var_then_number("forall_var_then_number", "∀x 5+1")]
#[case::forall_var_then_paren("forall_var_then_paren", "∀x (x>0)")]
// ∀<UpperVar><Variable/UpperVar/Number/OpenParen> with NO space between forces
// the ∀-with-body branch (symbol_rule.rs lines 171-173 — UpperVariable arm).
#[case::forall_upper_then_var_attached("forall_upper_then_var_attached", "∀Xp(x)")]
#[case::exists_upper_then_var_attached("exists_upper_then_var_attached", "∃Yq(y)")]
#[case::forall_upper_then_number("forall_upper_then_number", "∀X5")]
#[case::forall_upper_then_paren("forall_upper_then_paren", "∀X(x>0)")]
#[case::forall_lower_then_var_attached("forall_lower_then_var_attached", "∀xy")]
// =====================================================================
// rule_47 — log error paths via direct tokens (lim/log unmatched)
// =====================================================================
#[case::log_open_paren_no_close("log_open_paren_no_close", "log_2 (x+1")]
#[case::lim_open_paren_no_close("lim_open_paren_no_close", "lim_x (n→0")]
// =====================================================================
// detect.rs — specific patterns
// =====================================================================
#[case::detect_arcsin_uppercase("detect_arcsin_uppercase", "arcsinX")]
#[case::detect_letter_slash_simple("detect_letter_slash_simple", "a/b")]
#[case::detect_bracket_digits("detect_bracket_digits", "[123]")]
#[case::detect_bracket_letter_only("detect_bracket_letter_only", "[abc]")]
#[case::detect_func_call("detect_func_call", "f(x)")]
#[case::detect_func_call_g("detect_func_call_g", "g(y)")]
// =====================================================================
// rule_12 — UpperVariable with simple paren arg / numeric pair
// =====================================================================
#[case::upper_paren_var("upper_paren_var", "P(x)")]
#[case::upper_paren_complex("upper_paren_complex", "F(x+y)")]
#[case::upper_numeric_pair_c("upper_numeric_pair_c", "C(2,5)")]
#[case::upper_xor_negation_upper("upper_xor_negation_upper", "A∨¬B")]
// =====================================================================
// rule_57 / rule_54 — derivative + partial / continued
// =====================================================================
#[case::partial_derivative_form("partial_derivative_form", "∂f/∂x")]
#[case::triple_integral("triple_integral", "∭f")]
#[case::sigma_with_complex_x("sigma_with_complex_x", "∑x²")]
// =====================================================================
// rule_19 — left subscript / sub-fractions
// =====================================================================
#[case::left_subscript_sum("left_subscript_sum", "ₙΠᵣ")]
#[case::sub_letter_letter("sub_letter_letter", "log_a b")]
// =====================================================================
// rule_8 — decimal point edge cases
// =====================================================================
#[case::decimal_with_combining_mark("decimal_with_combining_mark", "0.5\u{0307}")]
#[case::decimal_then_next_dot_number("decimal_then_next_dot_number", "0.5.7")]
#[case::decimal_with_super_following("decimal_with_super_following", "1.5⁻¹")]
// =====================================================================
// encoder.rs (math) — Korean word inside math expression (제63항)
// =====================================================================
#[case::math_korean_word_in_paren("math_korean_word_in_paren", "x=한글일때 y=다른")]
#[case::math_korean_times_korean("math_korean_times_korean", "한국×수학")]
#[case::math_korean_eq_korean("math_korean_eq_korean", "수=학")]
// =====================================================================
// engine.rs — rule dispatch unusual paths
// =====================================================================
#[case::engine_skip_unmatched("engine_skip_unmatched", "\u{0001}")]
#[case::engine_multiple_rules_per_char("engine_multiple_rules_per_char", "ABCabc123")]
// =====================================================================
// emphasis_ring  — bold/italic markers (제32항)
// =====================================================================
#[case::emph_ring_double_star("emph_ring_double_star", "**bold text**")]
#[case::emph_ring_underscore("emph_ring_underscore", "_emphasized_")]
#[case::emph_ring_at_word_boundary("emph_ring_at_word_boundary", "the *quick* brown")]
// =====================================================================
// rule_12 — uppercase numeric pair / matrix
// =====================================================================
#[case::matrix_2x2_uppercase(
    "matrix_2x2_uppercase",
    "$\\begin{pmatrix} AB & CD \\\\ EF & GH \\end{pmatrix}$"
)]
#[case::sequence_2_upper_with_prime("sequence_2_upper_with_prime", "AB'CD")]
#[case::matrix_with_neg(
    "matrix_with_neg",
    "$\\begin{vmatrix} -1 & 2 \\\\ 3 & -4 \\end{vmatrix}$"
)]
// =====================================================================
// English-dominant Korean wrap — long english with embedded korean
// =====================================================================
#[case::eng_dom_korean_in_middle("eng_dom_korean_in_middle", "Hello 안녕 World")]
#[case::eng_dom_sentence_period("eng_dom_sentence_period", "The quick brown fox. 매우 빠르다.")]
#[case::eng_dom_long_with_uppercase(
    "eng_dom_long_with_uppercase",
    "API와 SDK를 사용해서 ABCDE 작업을 한다"
)]
// apply.rs special-pattern: ∆ + `=` + `)+(` triple-condition spacer
// (PDF 수학 — 증분 + 등호 + 다항식 조합)
#[case::delta_eq_polysum("delta_eq_polysum", "∆x=(a)+(b)")]
#[case::delta_eq_polysum_2("delta_eq_polysum_2", "∆y=(p)+(q)+(r)")]
// Same pattern but with a non-Korean leading token so apply.rs `index != 0`
// and `!prev_has_korean` both fire → the +2-space prefix arm runs.
#[case::delta_eq_polysum_with_lead("delta_eq_polysum_with_lead", "hello ∆x=(a)+(b)")]
#[case::delta_eq_polysum_with_lead2("delta_eq_polysum_with_lead2", "y ∆x=(a)+(b)")]
// apply.rs needs_decimal_context_spacing with Space prev (combining marks / ⋯)
#[case::ellipsis_in_kor_with_space("ellipsis_in_kor_with_space", "값 a⋯z 합")]
#[case::combining_mark_in_kor_with_space("combining_mark_in_kor_with_space", "값 x\u{0305} 합")]
// parse.rs `^` at end-of-input (Raw fallback)
#[case::caret_at_end_no_arg("caret_at_end_no_arg", "a^")]
#[case::caret_alone("caret_alone", "^")]
fn coverage_extra2_snapshot(#[case] name: &str, #[case] input: &str) {
    let rendered = render(input);
    insta::with_settings!({snapshot_path => "snapshots2"}, {
        insta::assert_snapshot!(name, rendered);
    });
}
