//! Coverage-extension integration tests.
//!
//! Each `#[case]` pins a single input through `encode` + `encode_to_unicode`
//! and snapshots the result (unicode braille + byte vector) via `insta`.
//! Snapshots live in `tests/snapshots/coverage_extra__snapshot_encode__*.snap`
//! and are checked into the repo, so any future regression in either
//! the braille output for a covered input, or the `Ok`/`Err` shape of
//! `encode`, shows up as a snapshot diff.
//!
//! The list is grouped by the source file each case primarily exercises so
//! `cargo tarpaulin` drift is easy to attribute.

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
// strip.rs — LaTeX → math notation
// =====================================================================
#[case::strip_super_digit_0("strip_super_digit_0", "$x^0$")]
#[case::strip_super_digit_1("strip_super_digit_1", "$x^1$")]
#[case::strip_super_digit_2("strip_super_digit_2", "$x^2$")]
#[case::strip_super_digit_3("strip_super_digit_3", "$x^3$")]
#[case::strip_super_digit_4("strip_super_digit_4", "$x^4$")]
#[case::strip_super_digit_5("strip_super_digit_5", "$x^5$")]
#[case::strip_super_digit_6("strip_super_digit_6", "$x^6$")]
#[case::strip_super_digit_7("strip_super_digit_7", "$x^7$")]
#[case::strip_super_digit_8("strip_super_digit_8", "$x^8$")]
#[case::strip_super_digit_9("strip_super_digit_9", "$x^9$")]
#[case::strip_super_at("strip_super_at", "$x^@$")]
#[case::strip_super_question("strip_super_question", "$x^?$")]
#[case::strip_super_z("strip_super_z", "$x^z$")]
#[case::strip_super_w("strip_super_w", "$y^w$")]
#[case::strip_overset_fg("strip_overset_fg", "$\\overset{f}{g}$")]
#[case::strip_overset_a_bc("strip_overset_a_bc", "$\\overset{a}{bc}$")]
#[case::strip_unknown_a("strip_unknown_a", "$\\a$")]
#[case::strip_unknown_q("strip_unknown_q", "$\\q$")]
#[case::strip_unknown_z_plus("strip_unknown_z_plus", "$\\z + 1$")]
#[case::strip_escaped_braces("strip_escaped_braces", "$\\{x\\}$")]
#[case::strip_escaped_braces_sum("strip_escaped_braces_sum", "$\\{a + b\\}$")]
#[case::strip_xrlh_label("strip_xrlh_label", "$\\xrightleftharpoons{f}$")]
#[case::strip_xrlh_label_ab("strip_xrlh_label_ab", "$A \\xrightleftharpoons{f} B$")]
#[case::strip_xrlh_empty("strip_xrlh_empty", "$\\xrightleftharpoons{}$")]
#[case::strip_not_mathcal("strip_not_mathcal", "$\\not\\mathcal{X}$")]
#[case::strip_not_mathcal_ab("strip_not_mathcal_ab", "$\\not\\mathcal{ab}$")]
#[case::strip_not_mathrel("strip_not_mathrel", "$\\not\\mathrel{P}$")]
#[case::strip_not_unknown("strip_not_unknown", "$\\not\\foobar$")]
#[case::strip_left_dot("strip_left_dot", "$\\left. x \\right.$")]
// =====================================================================
// latex_math.rs — `$X$Korean` particle context
// =====================================================================
#[case::dollar_letter_neun("dollar_letter_neun", "$x$는")]
#[case::dollar_letter_ida("dollar_letter_ida", "$y$이다")]
#[case::dollar_letter_eui("dollar_letter_eui", "$z$의")]
#[case::dollar_letter_reul("dollar_letter_reul", "$a$를")]
#[case::dollar_upper_neun("dollar_upper_neun", "$X$는")]
#[case::dollar_two_letter_neun("dollar_two_letter_neun", "$xy$는")]
#[case::dollar_letter_chain("dollar_letter_chain", "$a$가 $b$보다")]
#[case::dollar_alpha_neun("dollar_alpha_neun", "$\\alpha$는")]
#[case::dollar_omega_ga("dollar_omega_ga", "$\\omega$가")]
#[case::dollar_pi_eui("dollar_pi_eui", "$\\pi$의")]
#[case::dollar_theta_neun("dollar_theta_neun", "$\\theta$는")]
#[case::korean_prefix_n_hang("korean_prefix_n_hang", "제$n$항")]
#[case::korean_prefix_n_hang_kkaji("korean_prefix_n_hang_kkaji", "제$n$항까지")]
#[case::korean_prefix_x_jeol("korean_prefix_x_jeol", "제$x$절")]
#[case::korean_prefix_f_eui("korean_prefix_f_eui", "함수$f$의")]
// =====================================================================
// spacing.rs — comma-separated letter list, Korean prose
// =====================================================================
#[case::comma_list_abc("comma_list_abc", "점 $a, b, c$ 입니다")]
#[case::comma_list_xy("comma_list_xy", "변수 $x, y$ 와 상수")]
#[case::comma_list_upper_abc("comma_list_upper_abc", "$A, B, C$ 의 합")]
#[case::comma_list_ab_neun("comma_list_ab_neun", "수 $a, b$ 는")]
#[case::korean_space_latex_space("korean_space_latex_space", "한국 $x$ 수식")]
#[case::korean_space_latex_def("korean_space_latex_def", "점자 $a$ 정의")]
// =====================================================================
// merge_rule.rs / strip.rs — multi-token $...$ merge
// =====================================================================
#[case::merge_x_plus_1("merge_x_plus_1", "$x + 1$")]
#[case::merge_abc_sum("merge_abc_sum", "$a + b + c$")]
#[case::merge_x_eq_yz("merge_x_eq_yz", "$x = y + z$")]
#[case::merge_frac_sum("merge_frac_sum", "$\\frac{a + b}{c}$")]
#[case::merge_sum_eq("merge_sum_eq", "$\\sum i = 1$")]
#[case::merge_unterminated("merge_unterminated", "$x + 1 missing close")]
#[case::merge_unterminated_short("merge_unterminated_short", "$a + b")]
// =====================================================================
// symbol_rule.rs — math symbol dispatch
// =====================================================================
#[case::forall_x_p("forall_x_p", "∀x p(x)")]
#[case::exists_y_q("exists_y_q", "∃y q(y)")]
#[case::forall_x_f("forall_x_f", "∀x f(x)")]
#[case::exists_z_g("exists_z_g", "∃z g(z)")]
#[case::forall_upper("forall_upper", "∀X P(X)")]
#[case::exists_upper("exists_upper", "∃Y Q(Y)")]
#[case::forall_x_xy("forall_x_xy", "∀x x+y")]
#[case::exists_y_eq0("exists_y_eq0", "∃y y=0")]
#[case::sigma_eq_bound("sigma_eq_bound", "∑(i=1,n)i")]
#[case::sigma_eq_bound_k("sigma_eq_bound_k", "∑(k=0,N)k")]
#[case::sigma_comma_bound("sigma_comma_bound", "∑(i,n)i")]
#[case::sigma_numeric_bound("sigma_numeric_bound", "∑(1,5)x")]
#[case::sigma_paren_only("sigma_paren_only", "∑(x)")]
#[case::sigma_paren_sum("sigma_paren_sum", "∑(i+j)")]
#[case::pi_pair_1_10("pi_pair_1_10", "∏(1,10)")]
#[case::pi_pair_2_5("pi_pair_2_5", "∏(2,5)")]
#[case::pi_pair_0_100("pi_pair_0_100", "∏(0,100)")]
#[case::pi_pair_with_x("pi_pair_with_x", "x∏(1,n)")]
#[case::middle_dot_eq("middle_dot_eq", "a·b=c")]
#[case::middle_dot_plus("middle_dot_plus", "x·y+z")]
#[case::middle_dot_chain("middle_dot_chain", "p·q=r·s")]
#[case::therefore_x_eq_1("therefore_x_eq_1", "∴x=1")]
#[case::because_x_pos("because_x_pos", "∵x>0")]
#[case::ab_therefore("ab_therefore", "a∴b")]
#[case::because_spaced("because_spaced", "∵ p=q")]
#[case::therefore_lead("therefore_lead", "x = 1 ∴ y = 2")]
#[case::because_lead("because_lead", "p > 0 ∵ q > 0")]
#[case::eq_ab("eq_ab", "a=b")]
#[case::lt_ab("lt_ab", "a<b")]
#[case::gt_ab("gt_ab", "a>b")]
#[case::le_ab("le_ab", "a≤b")]
#[case::ge_ab("ge_ab", "a≥b")]
#[case::ne_ab("ne_ab", "a≠b")]
#[case::proportion("proportion", "a:b::c:d")]
#[case::double_arrow_imp("double_arrow_imp", "p⇒q")]
#[case::double_arrow_iff("double_arrow_iff", "p⇔q")]
#[case::right_arrow_ray("right_arrow_ray", "A→B")]
#[case::arrow_lr("arrow_lr", "x↔y")]
#[case::arrow_left("arrow_left", "x←y")]
#[case::greek_alpha("greek_alpha", "α")]
#[case::greek_pi("greek_pi", "π")]
#[case::custom_binop_ring("custom_binop_ring", "a∘b")]
#[case::custom_binop_bullet("custom_binop_bullet", "a∙b")]
#[case::prime_single("prime_single", "a'")]
#[case::prime_double("prime_double", "x''")]
#[case::approx_xy("approx_xy", "x≈y")]
#[case::abs_x("abs_x", "|x|")]
#[case::abs_sum("abs_sum", "|a+b|")]
#[case::divisibility_a_b("divisibility_a_b", "a|b")]
#[case::not_divides("not_divides", "a∤b")]
#[case::norm_x("norm_x", "‖x‖")]
#[case::norm_sum("norm_sum", "‖a+b‖")]
#[case::approx_equal("approx_equal", "a≅b")]
#[case::dot_congruence("dot_congruence", "a≐b")]
#[case::asymptotic("asymptotic", "a≃b")]
#[case::congruence("congruence", "a≡b")]
#[case::triangle_abc("triangle_abc", "△ABC")]
#[case::square_shape("square_shape", "□")]
#[case::arc_ab("arc_ab", "⌢AB")]
#[case::angle_abc("angle_abc", "∠ABC")]
#[case::triangle_only("triangle_only", "△")]
#[case::circle_shape("circle_shape", "○")]
#[case::perpendicular("perpendicular", "a⊥b")]
#[case::similarity("similarity", "a∼b")]
#[case::parallel("parallel", "a∥b")]
#[case::delta_x("delta_x", "Δx")]
#[case::partial_f("partial_f", "∂f")]
#[case::nabla_f("nabla_f", "∇f")]
#[case::integral_f("integral_f", "∫f")]
#[case::double_integral("double_integral", "∬f")]
#[case::contour_integral("contour_integral", "∮f")]
#[case::combining_dot_a("combining_dot_a", "a\u{0307}")]
#[case::combining_dot_x_plus_y("combining_dot_x_plus_y", "x\u{0307}+y")]
#[case::combining_dot_upper("combining_dot_upper", "A\u{0307}")]
#[case::norm_eq_y("norm_eq_y", "y=‖x‖")]
#[case::fullwidth_hash_a("fullwidth_hash_a", "\u{FF03}(A)")]
#[case::fullwidth_hash_x("fullwidth_hash_x", "\u{FF03}(X)")]
#[case::fullwidth_hash_space_a("fullwidth_hash_space_a", "\u{FF03} A")]
#[case::fullwidth_hash_double_space("fullwidth_hash_double_space", "\u{FF03}  A")]
#[case::fullwidth_hash_paren_space("fullwidth_hash_paren_space", "\u{FF03} ( A )")]
#[case::negation_ab("negation_ab", "A¬B")]
#[case::negation_xy("negation_xy", "X¬Y")]
#[case::negation_pq("negation_pq", "p¬Q")]
#[case::negation_ab_space("negation_ab_space", "A ¬ B")]
#[case::negation_pq_space("negation_pq_space", "p ¬ Q")]
#[case::arrow_label_xrightarrow("arrow_label_xrightarrow", "$A \\xrightarrow{f} B$")]
#[case::arrow_label_xleftarrow("arrow_label_xleftarrow", "$X \\xleftarrow{g} Y$")]
// =====================================================================
// rule_12.rs — UpperVariable / matrix / sequences
// =====================================================================
#[case::upper_numeric_pair_a("upper_numeric_pair_a", "A(2,5)")]
#[case::upper_numeric_pair_b("upper_numeric_pair_b", "B(1,10)")]
#[case::matrix_pair_ab_cd(
    "matrix_pair_ab_cd",
    "$\\begin{pmatrix} AB & CD \\\\ EF & GH \\end{pmatrix}$"
)]
#[case::multi_upper_abc("multi_upper_abc", "ABC")]
#[case::multi_upper_primes("multi_upper_primes", "A'B'C'")]
#[case::multi_upper_abcd("multi_upper_abcd", "ABCD")]
#[case::a_or_not_b("a_or_not_b", "A∨¬B")]
#[case::x_or_not_y("x_or_not_y", "X∨¬Y")]
#[case::predicate_p_x("predicate_p_x", "P(x)")]
#[case::predicate_f_y("predicate_f_y", "F(y)")]
#[case::predicate_g_z1("predicate_g_z1", "G(z+1)")]
#[case::predicate_h_ab("predicate_h_ab", "H(a, b)")]
#[case::predicate_t_n("predicate_t_n", "T(n)")]
#[case::overline_a("overline_a", "A\u{0305}")]
#[case::overline_b_macron("overline_b_macron", "B\u{0304}")]
#[case::overline_ab("overline_ab", "AB\u{0305}")]
#[case::overline_x_eq_0("overline_x_eq_0", "X\u{0305} = 0")]
// =====================================================================
// rule_18/19 — super/sub script edge cases
// =====================================================================
#[case::super_sum_ab("super_sum_ab", "$x^{a+b}$")]
#[case::super_neg_1("super_neg_1", "$x^{-1}$")]
#[case::sub_i_plus_1("sub_i_plus_1", "$x_{i+1}$")]
#[case::sub_n_minus_1("sub_n_minus_1", "$x_{n-1}$")]
#[case::nested_super("nested_super", "$a^{b^c}$")]
#[case::nested_sub("nested_sub", "$a_{b_c}$")]
#[case::super_frac_form("super_frac_form", "$x^{a/b}$")]
#[case::sub_frac_form("sub_frac_form", "$x_{a/b}$")]
#[case::super_sqrt("super_sqrt", "$x^{\\sqrt{2}}$")]
#[case::sum_index_pair("sum_index_pair", "$\\sum_{i=0}^{n}$")]
#[case::integral_0_inf("integral_0_inf", "$\\int_0^\\infty$")]
#[case::transpose("transpose", "$A^T$")]
#[case::matrix_ij("matrix_ij", "$A_{ij}$")]
// =====================================================================
// parser edge cases
// =====================================================================
#[case::empty_frac("empty_frac", "$\\frac{}{}$")]
#[case::empty_super("empty_super", "$x^{}$")]
#[case::empty_sub("empty_sub", "$x_{}$")]
#[case::empty_sqrt("empty_sqrt", "$\\sqrt{}$")]
#[case::empty_paren("empty_paren", "$()$")]
#[case::empty_braces("empty_braces", "$\\{\\}$")]
#[case::function_noarg("function_noarg", "$f()$")]
#[case::log_empty_sub("log_empty_sub", "$\\log_{}$")]
#[case::sub_then_super("sub_then_super", "$x_1^2$")]
#[case::super_then_sub("super_then_sub", "$x^2_1$")]
#[case::nested_frac("nested_frac", "$\\frac{1}{\\frac{2}{3}}$")]
#[case::paren_exp("paren_exp", "$(x)^2$")]
#[case::paren_sub("paren_sub", "$(a+b)_n$")]
#[case::abs_squared("abs_squared", "$|x|^2$")]
#[case::sin_squared("sin_squared", "$\\sin^2 x$")]
// =====================================================================
// matrix.rs — environment variants
// =====================================================================
#[case::matrix_empty("matrix_empty", "$\\begin{matrix} \\end{matrix}$")]
#[case::pmatrix_single("pmatrix_single", "$\\begin{pmatrix} a \\end{pmatrix}$")]
#[case::vmatrix_single_row("vmatrix_single_row", "$\\begin{vmatrix} a & b & c \\end{vmatrix}$")]
#[case::matrix_single_col("matrix_single_col", "$\\begin{matrix} 1 \\\\ 2 \\\\ 3 \\end{matrix}$")]
#[case::array_pipes("array_pipes", "$\\begin{array}{|c|c|} a & b \\\\ c & d \\end{array}$")]
#[case::array_c_pipe_c(
    "array_c_pipe_c",
    "$\\begin{array}{c|c} 1 & 2 \\\\ 3 & 4 \\end{array}$"
)]
#[case::array_three_pipe_cols(
    "array_three_pipe_cols",
    "$\\begin{array}{|c|c|c|} 1 & 2 & 3 \\end{array}$"
)]
#[case::array_rcl(
    "array_rcl",
    "$\\begin{array}{rcl} a & = & b \\\\ c & = & d \\end{array}$"
)]
#[case::cases_two("cases_two", "$\\begin{cases} a \\\\ b \\end{cases}$")]
#[case::cases_three("cases_three", "$\\begin{cases} a \\\\ b \\\\ c \\end{cases}$")]
#[case::matrix_nested_frac(
    "matrix_nested_frac",
    "$\\begin{matrix} \\frac{1}{2} & x \\\\ y & \\sqrt{2} \\end{matrix}$"
)]
#[case::matrix_pmatrix_braces(
    "matrix_pmatrix_braces",
    "$\\begin{pmatrix} \\{a\\} & b \\end{pmatrix}$"
)]
#[case::bmatrix_negative(
    "bmatrix_negative",
    "$\\begin{bmatrix} -1 & 0 \\\\ 0 & -1 \\end{bmatrix}$"
)]
// =====================================================================
// apply.rs — colon-math, set-builder, multi-letter Korean ident
// =====================================================================
#[case::colon_math_lt("colon_math_lt", "a < b:")]
#[case::colon_math_gt("colon_math_gt", "a > b:")]
#[case::colon_math_eq("colon_math_eq", "a = b:")]
#[case::colon_math_ne("colon_math_ne", "a ≠ b:")]
#[case::colon_math_le("colon_math_le", "a ≤ b:")]
#[case::colon_math_ge("colon_math_ge", "a ≥ b:")]
#[case::colon_math_lesssim("colon_math_lesssim", "a ≲ b:")]
#[case::colon_math_gtrsim("colon_math_gtrsim", "a ≳ b:")]
#[case::colon_math_prec("colon_math_prec", "a ≺ b:")]
#[case::colon_math_succ("colon_math_succ", "a ≻ b:")]
#[case::colon_math_in("colon_math_in", "a ∈ b:")]
#[case::colon_math_notin("colon_math_notin", "a ∉ b:")]
#[case::colon_math_xor("colon_math_xor", "p ⊻ q:")]
#[case::set_builder_basic("set_builder_basic", "{x | x > 0}")]
#[case::set_builder_real("set_builder_real", "{x | x ∈ R}")]
#[case::set_builder_korean("set_builder_korean", "{n | n 은 정수}")]
#[case::set_builder_eq("set_builder_eq", "{a | a + b = c}")]
#[case::set_builder_sq("set_builder_sq", "{x | x^2 = 4}")]
#[case::set_builder_range("set_builder_range", "{x | 0 < x < 1}")]
#[case::multi_letter_ab_lower("multi_letter_ab_lower", "ab의 값을 구하라")]
#[case::multi_letter_ab_upper("multi_letter_ab_upper", "AB의 값은 5이다")]
#[case::multi_letter_xy_product("multi_letter_xy_product", "xy의 곱은 0")]
#[case::multi_letter_abc_upper("multi_letter_abc_upper", "ABC의 값을 계산")]
#[case::multi_letter_pqr_product("multi_letter_pqr_product", "pqr의 곱을 구하시오")]
#[case::multi_letter_abcd_compare("multi_letter_abcd_compare", "AB와 CD의 값을 비교")]
#[case::greek_list_alpha_beta("greek_list_alpha_beta", "각 α, β에 대하여")]
#[case::greek_list_pi_sigma("greek_list_pi_sigma", "값 π, σ는 양수")]
#[case::greek_list_theta_phi("greek_list_theta_phi", "변수 θ, φ가 직각")]
#[case::greek_list_alpha_beta_sum("greek_list_alpha_beta_sum", "각도 α, β의 합")]
#[case::ellipsis_subscript_a("ellipsis_subscript_a", "a₁, a₂, ..., aₙ")]
#[case::ellipsis_subscript_x("ellipsis_subscript_x", "x₁, x₂, ..., xₙ 의 합")]
#[case::ellipsis_numbers("ellipsis_numbers", "1, 2, 3, ..., 10")]
#[case::ellipsis_letters("ellipsis_letters", "a + b + ... + z")]
#[case::ellipsis_ldots("ellipsis_ldots", "$f(x_1, x_2, \\ldots, x_n)$")]
#[case::therefore_korean("therefore_korean", "조건 ∴ 결론")]
#[case::because_korean("because_korean", "전제 ∵ 근거")]
#[case::dollar_neg2_korean("dollar_neg2_korean", "$-2$는 음수")]
#[case::dollar_decimal_korean("dollar_decimal_korean", "$0.3010$이다")]
#[case::dollar_frac_korean("dollar_frac_korean", "$\\frac{1}{2}$의 역수")]
#[case::dollar_sum_korean("dollar_sum_korean", "$x+1$은 양수")]
#[case::dollar_commalist_korean("dollar_commalist_korean", "$a, b, c$에 대하여")]
#[case::dollar_sin_korean("dollar_sin_korean", "$\\sin x$가 0")]
#[case::korean_math_value("korean_math_value", "수식 f(x)+g(x) 의 값")]
#[case::korean_math_squared("korean_math_squared", "변수 a^2 와 b^2")]
#[case::korean_math_factor("korean_math_factor", "함수 (x+1)(x-1) 분해")]
#[case::korean_math_matrix("korean_math_matrix", "행렬 [a;b] 의 곱")]
#[case::korean_math_divides("korean_math_divides", "조건 x|y 정의")]
#[case::korean_math_triangle("korean_math_triangle", "삼각형 △ABC 의 둘레")]
#[case::korean_math_circle("korean_math_circle", "원 ⊙O 의 반지름")]
#[case::korean_math_angle("korean_math_angle", "각 ∠A 의 크기")]
// =====================================================================
// Unicode super/sub codepoint sweep (parser table)
// =====================================================================
#[case::usup_0("usup_0", "a⁰")]
#[case::usup_1("usup_1", "a¹")]
#[case::usup_2("usup_2", "a²")]
#[case::usup_3("usup_3", "a³")]
#[case::usup_4("usup_4", "a⁴")]
#[case::usup_5("usup_5", "a⁵")]
#[case::usup_6("usup_6", "a⁶")]
#[case::usup_7("usup_7", "a⁷")]
#[case::usup_8("usup_8", "a⁸")]
#[case::usup_9("usup_9", "a⁹")]
#[case::usup_plus("usup_plus", "a⁺")]
#[case::usup_minus("usup_minus", "a⁻")]
#[case::usup_n("usup_n", "aⁿ")]
#[case::usup_k("usup_k", "aᵏ")]
#[case::usup_m("usup_m", "aᵐ")]
#[case::usup_x("usup_x", "aˣ")]
#[case::usup_paren("usup_paren", "a⁽ᵇ⁾")]
#[case::usub_0("usub_0", "a₀")]
#[case::usub_1("usub_1", "a₁")]
#[case::usub_2("usub_2", "a₂")]
#[case::usub_3("usub_3", "a₃")]
#[case::usub_4("usub_4", "a₄")]
#[case::usub_5("usub_5", "a₅")]
#[case::usub_6("usub_6", "a₆")]
#[case::usub_7("usub_7", "a₇")]
#[case::usub_8("usub_8", "a₈")]
#[case::usub_9("usub_9", "a₉")]
#[case::usub_plus("usub_plus", "a₊")]
#[case::usub_minus("usub_minus", "a₋")]
#[case::usub_a("usub_a", "aₐ")]
#[case::usub_e("usub_e", "aₑ")]
#[case::usub_o("usub_o", "aₒ")]
#[case::usub_x("usub_x", "aₓ")]
#[case::usub_h("usub_h", "aₕ")]
#[case::usub_k("usub_k", "aₖ")]
#[case::usub_l("usub_l", "aₗ")]
#[case::usub_m("usub_m", "aₘ")]
#[case::usub_n("usub_n", "aₙ")]
#[case::usub_p("usub_p", "aₚ")]
#[case::usub_s("usub_s", "aₛ")]
#[case::usub_t("usub_t", "aₜ")]
#[case::usub_i("usub_i", "aᵢ")]
#[case::usub_r("usub_r", "aᵣ")]
#[case::usub_u("usub_u", "aᵤ")]
#[case::usub_v("usub_v", "aᵥ")]
#[case::usub_frac("usub_frac", "x₁/₂")]
#[case::usub_decimal("usub_decimal", "x₀.₅")]
// =====================================================================
// rule_18 — number^digit with middle dot / slash
// =====================================================================
#[case::sci_2_10_2("sci_2_10_2", "2·10²")]
#[case::sci_3_10_neg5("sci_3_10_neg5", "3·10⁻⁵")]
#[case::frac_1_10_2("frac_1_10_2", "1/10²")]
#[case::frac_5_2_3("frac_5_2_3", "5/2³")]
// =====================================================================
// grouping.rs — multi-char super/sub mapping
// =====================================================================
#[case::sub_aeo("sub_aeo", "$x_{aeo}$")]
#[case::sub_hkl("sub_hkl", "$x_{hkl}$")]
#[case::sub_mnp("sub_mnp", "$x_{mnp}$")]
#[case::sub_st("sub_st", "$x_{st}$")]
#[case::sub_iruv("sub_iruv", "$x_{iruv}$")]
#[case::sub_0123("sub_0123", "$x_{0123}$")]
#[case::sub_4567("sub_4567", "$x_{4567}$")]
#[case::sub_89("sub_89", "$x_{89}$")]
#[case::sub_plus_1("sub_plus_1", "$y_{+1}$")]
#[case::sub_minus_1("sub_minus_1", "$y_{-1}$")]
#[case::sub_paren_a("sub_paren_a", "$z_{(a)}$")]
#[case::sub_unmapped("sub_unmapped", "$f_{xyz}$")]
#[case::sup_nkm("sup_nkm", "$x^{nkm}$")]
#[case::sup_4567("sup_4567", "$x^{4567}$")]
#[case::sup_89("sup_89", "$x^{89}$")]
#[case::sup_6("sup_6", "$x^{6}$")]
#[case::sup_ab_div_cd("sup_ab_div_cd", "$x^{ab/cd}$")]
#[case::sup_paren_a("sup_paren_a", "$x^{(a)}$")]
#[case::sup_dot("sup_dot", "$x^{a.b}$")]
#[case::sup_0123("sup_0123", "$x^{0123}$")]
#[case::frac_outer_paren("frac_outer_paren", "$\\frac{(x+1)}{(y-2)}$")]
#[case::frac_single_a("frac_single_a", "$\\frac{(a)}{b}$")]
#[case::frac_adjacent_parens("frac_adjacent_parens", "$\\frac{(a+b)(c+d)}{e}$")]
#[case::frac_differential("frac_differential", "$\\frac{dx}{dy}$")]
#[case::frac_partial_diff("frac_partial_diff", "$\\frac{d^2 z}{dx dy}$")]
// =====================================================================
// math expression detection edge cases
// =====================================================================
#[case::detect_long_arith("detect_long_arith", "1+2+3+4+5")]
#[case::detect_nested_parens("detect_nested_parens", "(((a)))")]
#[case::detect_set_builder("detect_set_builder", "{x | x > 0}")]
#[case::detect_multi_constraints("detect_multi_constraints", "x ≥ 0, y ≤ 1")]
#[case::detect_mixed_ops("detect_mixed_ops", "a*b/c+d-e")]
#[case::detect_chained_super("detect_chained_super", "a^b^c")]
#[case::detect_scientific("detect_scientific", "1.5e-3")]
#[case::detect_decimal("detect_decimal", "0.123")]
#[case::detect_thousands("detect_thousands", "1,000")]
#[case::detect_ne_empty("detect_ne_empty", "x ≠ ∅")]
#[case::detect_subset_chain("detect_subset_chain", "a ⊂ b ⊂ c")]
#[case::detect_set_empty("detect_set_empty", "{∅}")]
// =====================================================================
// parser diverse inputs
// =====================================================================
#[case::parse_alpha_beta("parse_alpha_beta", "α + β")]
#[case::parse_decimal_pair("parse_decimal_pair", "1.0 + 2.5")]
#[case::parse_leading_zero("parse_leading_zero", "0.001")]
#[case::parse_thousands("parse_thousands", "10,000")]
#[case::parse_bare_decimal("parse_bare_decimal", ".5")]
#[case::parse_chained_slash("parse_chained_slash", "1/2/3")]
#[case::parse_alt_super_sub("parse_alt_super_sub", "a^b_c^d_e")]
#[case::parse_multiarg_function("parse_multiarg_function", "f(x, y, z)")]
#[case::parse_lim_arrow("parse_lim_arrow", "lim a→b")]
#[case::parse_floor("parse_floor", "⌊x⌋")]
#[case::parse_ceiling("parse_ceiling", "⌈x⌉")]
fn snapshot_encode(#[case] name: &str, #[case] input: &str) {
    let rendered = render(input);
    insta::assert_snapshot!(name, rendered);
}
