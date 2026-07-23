//! Strip LaTeX commands to math-mode equivalent (extracted from latex_math.rs).
//!
//! `strip_latex_to_math` converts a LaTeX expression into the internal
//! math-notation form that the math token engine understands.

use super::grouping::{needs_grouping_in_fraction, to_subscript_sequence, to_superscript_sequence};
use super::read_braced_content;

pub(crate) fn strip_latex_to_math(latex_inner: &str) -> String {
    let mut result = String::new();
    let mut chars = latex_inner.chars().peekable();
    let mut escaped_brace_depth = 0usize;
    // 직전에 LaTeX 명령(`\command`)이 emit한 결과인지 추적: 명령 주변 공백은 LaTeX
    // 토큰 분리용이므로 제거해야 하고, 직접 Unicode 기호 주변 공백은 보존해야 한다.
    let mut last_emit_from_latex = false;

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            // PDF 수학 — 직접 Unicode 이항 연산자(`∘`, `∙` 등) 양측 공백은 의미가 있다.
            // 단, LaTeX 명령에서 emit된 직후의 공백은 명령 구분용이므로 제거한다.
            // PDF — `‖ ‖`(연속된 norm 기호) 사이는 공백 유지. norm과 다른 연산자
            // 사이는 공백 없음. 따라서 norm(U+2016)은 다음 글자도 norm일 때만 보존한다.
            // ∘(U+2218)과 ∙(U+2219)는 입력 공백을 보존하면 의미가 유지된다.
            // PDF — `\cdots`, `\ldots`(⋯, …)는 토큰 분리 의미가 있으므로 LaTeX 명령에서
            // emit되었더라도 양측 공백을 보존한다.
            let last_is_ellipsis = result
                .chars()
                .last()
                .is_some_and(|c| matches!(c, '\u{22EF}' | '\u{2026}'));
            let next_is_ellipsis = chars
                .peek()
                .is_some_and(|c| matches!(*c, '\u{22EF}' | '\u{2026}'));
            let last_is_unicode_binop = (!last_emit_from_latex
                && result
                    .chars()
                    .last()
                    .is_some_and(|c| matches!(c, '\u{2218}' | '\u{2219}')))
                || last_is_ellipsis;
            let next_is_unicode_binop = chars
                .peek()
                .is_some_and(|c| matches!(*c, '\u{2218}' | '\u{2219}'))
                || next_is_ellipsis;
            let norm_pair = !last_emit_from_latex
                && result.ends_with('\u{2016}')
                && chars.peek() == Some(&'\\')
                && {
                    // peek next-next: skip `\` and check for `|`
                    let mut clone = chars.clone();
                    clone.next();
                    clone.peek() == Some(&'|')
                };
            // PDF — 한국어 문맥에서는 공백을 보존해야 한다. LaTeX 명령은
            // 공백을 토큰 분리용으로 쓰지만, 한국어 단어 사이의 공백은
            // 묵자 그대로 보존돼야 점역이 정확해진다.
            let last_is_korean = result
                .chars()
                .last()
                .is_some_and(crate::utils::is_korean_char);
            let next_is_korean = chars
                .peek()
                .is_some_and(|c| crate::utils::is_korean_char(*c));
            if last_is_unicode_binop || next_is_unicode_binop || norm_pair {
                result.push('\u{00A0}');
            } else if last_is_korean && next_is_korean {
                result.push(' ');
            }
            continue;
        }

        // 비공백이고 LaTeX 명령이 아닌 글자는 일반 emit으로 본다.
        if c != '\\' {
            last_emit_from_latex = false;
        }

        if c == '\\' {
            // Read the command name
            let mut cmd = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_ascii_alphabetic() {
                    cmd.push(next);
                    chars.next();
                } else {
                    break;
                }
            }

            if cmd.is_empty() {
                if let Some(escaped) = chars.next() {
                    // Track literal brace depth for \{ ... \} pairs
                    if escaped == '{' {
                        escaped_brace_depth += 1;
                        result.push(escaped); // \\{ is a literal brace
                    } else if escaped == '}' {
                        escaped_brace_depth = escaped_brace_depth.saturating_sub(1);
                        result.push(escaped); // \\} is always a literal brace
                    } else if matches!(escaped, ',' | ';' | '!' | ':') {
                        // \\, \\; \\! \\: are LaTeX spacing commands - skip
                    } else if escaped == '|' {
                        result.push('\u{2016}'); // \\| is norm delimiter
                    } else if escaped == '#' {
                        // PDF 수학 제65항 1 — \# 는 fullwidth hash ＃ (기수 표시)
                        result.push('\u{FF03}');
                    } else {
                        result.push(escaped);
                    }
                }
                continue;
            }

            // Convert LaTeX commands to math symbols or pass through
            match cmd.as_str() {
                "sin" => result.push_str("sin"),
                "cos" => result.push_str("cos"),
                "tan" => result.push_str("tan"),
                "csc" => result.push_str("csc"),
                "sec" => result.push_str("sec"),
                "cot" => result.push_str("cot"),
                "sinh" => result.push_str("sinh"),
                "cosh" => result.push_str("cosh"),
                "tanh" => result.push_str("tanh"),
                "log" => result.push_str("log"),
                "ln" => result.push_str("ln"),
                "lim" => result.push_str("lim"),
                "arcsin" => result.push_str("arcsin"),
                "arccos" => result.push_str("arccos"),
                "arctan" => result.push_str("arctan"),
                "cosec" => result.push_str("cosec"),
                "neq" | "ne" => result.push('\u{2260}'), // ≠
                "geq" | "ge" => result.push('\u{2265}'), // ≥
                "leq" | "le" => result.push('\u{2264}'), // ≤
                "quad" | "qquad" => result.push(' '),    // 큰 공백
                "text" | "mathrm" | "mathit" | "mathbf" | "mathsf" => {
                    // \text{X}, \mathrm{X} 등 — 본문을 그대로 emit (LaTeX 텍스트 박스)
                    if let Some(inner) = read_braced_content(&mut chars) {
                        result.push_str(&strip_latex_to_math(&inner));
                    }
                }
                "approx" => result.push('\u{2248}'), // ≈ (이중물결)
                "infty" => result.push('\u{221E}'),  // ∞
                "to" => result.push('\u{2192}'),     // →
                "surd" => result.push('\u{221A}'),   // √
                "sqrt" => {
                    let mut index = None;
                    if chars.peek() == Some(&'[') {
                        chars.next();
                        let mut depth = 1usize;
                        let mut idx = String::new();
                        for ch in chars.by_ref() {
                            match ch {
                                '[' => {
                                    depth += 1;
                                    idx.push(ch);
                                }
                                ']' => {
                                    depth = depth.saturating_sub(1);
                                    if depth == 0 {
                                        break;
                                    }
                                    idx.push(ch);
                                }
                                _ => idx.push(ch),
                            }
                        }
                        index = Some(idx);
                    }

                    let radicand_raw = read_braced_content(&mut chars).unwrap_or_default();
                    // 내부 LaTeX 명령(중첩된 \sqrt, \frac 등)을 재귀적으로 strip.
                    let radicand = strip_latex_to_math(&radicand_raw);

                    if let Some(idx) = index {
                        let idx_norm = strip_latex_to_math(&idx);
                        result.push_str(&to_superscript_sequence(&idx_norm));
                    }
                    result.push('\u{221A}');

                    // 다항/복합 본문은 그룹 괄호로 묶는다. 본문이 이미 괄호를 포함하거나
                    // 단일 외곽 괄호로 감싸져 있으면 중복 그룹화를 생략한다.
                    let chars: Vec<char> = radicand.chars().collect();
                    let already_wrapped = chars.first() == Some(&'(') && chars.last() == Some(&')');
                    let contains_paren = chars.iter().any(|c| matches!(*c, '(' | ')'));
                    let contains_root = chars.contains(&'\u{221A}');
                    let all_alphabetic =
                        chars.len() > 1 && chars.iter().all(|c| c.is_ascii_alphabetic());
                    // PDF — sqrt 본문이 산술 연산을 포함하면 묶어 모호성을 제거한다.
                    let has_operator = chars
                        .iter()
                        .any(|c| matches!(*c, '+' | '-' | '\u{2212}' | '×' | '*' | '/'));
                    let needs_grouping = !already_wrapped
                        && !contains_paren
                        && (all_alphabetic || contains_root || has_operator);
                    if needs_grouping {
                        // PDF — sqrt 본문 묶음:
                        //   글자만 모인 본문(예: `√xy`)은 `⠷...⠾`(Grouping).
                        //   산술 연산을 포함한 본문(예: `√(a²-x²)`)은 `⠦...⠴`(MathParen).
                        if has_operator {
                            result.push('\u{27E6}');
                            result.push_str(&radicand);
                            result.push('\u{27E7}');
                        } else {
                            result.push('(');
                            result.push_str(&radicand);
                            result.push(')');
                        }
                    } else {
                        result.push_str(&radicand);
                    }
                }
                "Pi" => result.push('\u{03A0}'),    // Π
                "times" => result.push('\u{00D7}'), // ×
                "div" => result.push('\u{00F7}'),   // ÷
                "pm" => result.push('±'),
                "cdot" => result.push('\u{00B7}'),  // ·
                "cdots" => result.push('\u{22EF}'), // ⋯ (수평 줄임표 — math_symbol_shortcut에서 ⠠⠠⠠ 매핑)
                "ldots" => result.push('\u{2026}'), // … (수평 점 셋 줄임표)
                "alpha" => result.push('\u{03B1}'),
                "beta" => result.push('\u{03B2}'),
                "gamma" => result.push('\u{03B3}'),
                "delta" => result.push('\u{03B4}'),
                "theta" => result.push('\u{03B8}'),
                "pi" => result.push('\u{03C0}'),
                "sigma" => result.push('\u{03C3}'),
                "omega" => result.push('\u{03C9}'),
                "Gamma" => result.push('\u{0393}'),
                "epsilon" => result.push('\u{03B5}'),
                "varepsilon" => result.push('\u{03B5}'),
                "zeta" => result.push('\u{03B6}'),
                "eta" => result.push('\u{03B7}'),
                "Theta" => result.push('\u{0398}'),
                "iota" => result.push('\u{03B9}'),
                "kappa" => result.push('\u{03BA}'),
                "Lambda" => result.push('\u{039B}'),
                "lambda" => result.push('\u{03BB}'),
                "mu" => result.push('\u{03BC}'),
                "nu" => result.push('\u{03BD}'),
                "Xi" => result.push('\u{039E}'),
                "xi" => result.push('\u{03BE}'),
                "omicron" => result.push('\u{03BF}'),
                "rho" => result.push('\u{03C1}'),
                "tau" => result.push('\u{03C4}'),
                "Upsilon" => result.push('\u{03A5}'),
                "upsilon" => result.push('\u{03C5}'),
                "Phi" => result.push('\u{03A6}'),
                "phi" => result.push('\u{03C6}'),
                "varphi" => result.push('\u{03C6}'),
                "chi" => result.push('\u{03C7}'),
                "Psi" => result.push('\u{03A8}'),
                "psi" => result.push('\u{03C8}'),
                "Delta" => result.push('\u{0394}'),
                "Sigma" => result.push('\u{03A3}'),
                "sum" => result.push('\u{2211}'), // ∑ (n-ary summation, distinct from Σ)
                "int" => result.push('\u{222B}'), // ∫
                "Omega" => result.push('\u{03A9}'),
                "square" => result.push('\u{25A1}'),
                "circ" => result.push('\u{2218}'), // ∘ (합성함수 기호)
                "xrightarrow" => {
                    // PDF — `x \xrightarrow{f} y` -> `x [sp] f→ [sp] y`.
                    // 라벨이 있는 화살표: 라벨 앞에 공백, 라벨과 화살표 본체 사이 공백 없음.
                    // 좌측 공백을 명시적으로 emit해 parser가 Space token으로 인식하게 한다
                    // (이 Space는 후속 encoder의 labeled-arrow 컨텍스트 검출에 사용된다).
                    let label = read_braced_content(&mut chars).unwrap_or_default();
                    let norm = strip_latex_to_math(&label);
                    if !norm.trim().is_empty() {
                        // 좌측 공백 명시: 결과가 이미 공백/시작이 아니면 NBSP 삽입.
                        if !result.is_empty()
                            && !result.ends_with(' ')
                            && !result.ends_with('\u{00A0}')
                        {
                            result.push('\u{00A0}');
                        }
                        result.push_str(&norm);
                    }
                    result.push('\u{2192}'); // right arrow
                    // 우측 공백 명시: 후속 입력의 공백이 LaTeX skip되지 않도록 NBSP emit.
                    result.push('\u{00A0}');
                }
                "xrightleftharpoons" => {
                    // PDF — `\xrightleftharpoons[g]{f}` -> `f평형화살표g` (label위, below아래).
                    // 라벨 앞에 공백, 라벨-화살표-below 사이는 공백 없음.
                    if chars.peek() == Some(&'[') {
                        chars.next();
                        let mut depth = 1usize;
                        let mut below = String::new();
                        for ch in chars.by_ref() {
                            match ch {
                                '[' => {
                                    depth += 1;
                                    below.push(ch);
                                }
                                ']' => {
                                    depth = depth.saturating_sub(1);
                                    if depth == 0 {
                                        break;
                                    }
                                    below.push(ch);
                                }
                                _ => below.push(ch),
                            }
                        }
                        let label = read_braced_content(&mut chars).unwrap_or_default();
                        let norm_label = strip_latex_to_math(&label);
                        let norm_below = strip_latex_to_math(&below);
                        if !norm_label.trim().is_empty() {
                            if !result.is_empty()
                                && !result.ends_with(' ')
                                && !result.ends_with('\u{00A0}')
                            {
                                result.push('\u{00A0}');
                            }
                            result.push_str(&norm_label);
                        }
                        result.push('\u{21C4}');
                        if !norm_below.trim().is_empty() {
                            result.push_str(&norm_below);
                        }
                        result.push('\u{00A0}');
                    } else {
                        let label = read_braced_content(&mut chars).unwrap_or_default();
                        let norm = strip_latex_to_math(&label);
                        if !norm.trim().is_empty() {
                            if !result.is_empty()
                                && !result.ends_with(' ')
                                && !result.ends_with('\u{00A0}')
                            {
                                result.push('\u{00A0}');
                            }
                            result.push_str(&norm);
                        }
                        result.push('\u{21C4}');
                        result.push('\u{00A0}');
                    }
                }
                "vec" => {
                    if let Some(inner) = read_braced_content(&mut chars) {
                        result.push('\u{20D7}');
                        let norm = strip_latex_to_math(&inner);
                        if !norm.trim().is_empty() {
                            result.push_str(&norm);
                        }
                    }
                }
                "overrightarrow" => {
                    if let Some(inner) = read_braced_content(&mut chars) {
                        result.push('\u{20D7}');
                        let norm = strip_latex_to_math(&inner);
                        if !norm.trim().is_empty() {
                            result.push_str(&norm);
                        }
                    }
                }
                "frac" => {
                    if let Some(num) = read_braced_content(&mut chars)
                        && let Some(den) = read_braced_content(&mut chars)
                    {
                        let norm_num = strip_latex_to_math(&num);
                        let norm_den = strip_latex_to_math(&den);
                        // 한국 점자 수학 규정: 분수는 점자 표기에서 분모/분자 역순으로
                        // 적는다. LaTeX `\frac{a}{b}` (a=분자, b=분모)는 점자로 `b/a`.
                        //
                        // 알고리즘 일관성: 단순 숫자 분수(`\frac{3}{4}` → `#3/#4`)는 자연
                        // 순서로 strip하여 math parser의 FractionReversalRule이 일관되게
                        // 역순화하도록 한다(중복 역순화 방지). 복합 분수는 분자/분모가
                        // parser에서 별개 토큰화되므로 strip 단계에서 미리 역순으로
                        // 적어야 한다.
                        // parser/엔진이 일관된 역순화를 수행하는 경우는 자연 순서로 strip한다:
                        //  - 팩토리얼 분수 (parser entry의 factorial split)
                        //  - 편미분 `∂x/∂y` (PartialDerivativeFractionRule)
                        //  - 위첨자/아래첨자 안의 단순 수치 분수도 plain ⠌가 필요하므로 reversed에 맡긴다.
                        let is_factorial_form = |s: &str| -> bool {
                            !s.is_empty()
                                && s.chars().all(|c| c.is_ascii_digit() || c == '!')
                                && s.ends_with('!')
                        };
                        // 편미분: ∂ + 단일 변수 형태(예: "∂x")
                        let is_partial_var = |s: &str| -> bool {
                            let chars: Vec<char> = s.chars().collect();
                            chars.len() == 2
                                && chars[0] == '\u{2202}'
                                && chars[1].is_ascii_alphabetic()
                        };
                        let natural_order = (is_factorial_form(&norm_num)
                            && is_factorial_form(&norm_den))
                            || (is_partial_var(&norm_num) && is_partial_var(&norm_den));
                        // PDF — 함수의 인수로 들어가는 분수는 그룹으로 묶는다.
                        // (예: `\sin^{-1}\frac{x}{3}` → `sin^{-1}⟨3/x⟩`)
                        // result가 함수명 또는 함수+위첨자 형태로 끝나면 wrap 강제한다.
                        let result_after_func = {
                            let trailing: String = result
                                .chars()
                                .rev()
                                .take_while(|c| {
                                    c.is_ascii_alphanumeric()
                                        || matches!(
                                            c,
                                            '^' | '{'
                                                | '}'
                                                | '-'
                                                | '+'
                                                | '\u{207B}'
                                                | '\u{207A}'
                                                | '\u{00B9}'
                                                | '\u{00B2}'
                                                | '\u{00B3}'
                                                | '\u{2074}'
                                                ..='\u{2079}'
                                        )
                                })
                                .collect::<String>()
                                .chars()
                                .rev()
                                .collect::<String>();
                            [
                                "sin", "cos", "tan", "log", "ln", "lim", "exp", "csc", "sec",
                                "cot", "sinh", "cosh", "tanh",
                            ]
                            .iter()
                            .any(|f| trailing.starts_with(f) || trailing.ends_with(*f))
                        };
                        if natural_order {
                            // 자연순서: num/den → parser/engine이 reverse하여 den/num 출력.
                            result.push_str(&norm_num);
                            result.push('/');
                            result.push_str(&norm_den);
                        } else if result_after_func {
                            // 함수 인수 분수: 그룹 wrap 후 역순.
                            result.push('\u{2329}');
                            result.push_str(&norm_den);
                            result.push('\u{2044}');
                            result.push_str(&norm_num);
                            result.push('\u{232A}');
                        } else {
                            // 역순서: den/num. 슬래시는 U+2044(분수 전용)로 표기해 일반 `/`
                            // 와 구분한다. parser는 U+2044를 MathSymbol로 유지하고
                            // shortcut에서 `⠌`(plain)로 인코딩한다.
                            // 한글 포함 시 U+27E8/U+27E9 sentinel을 사용해 Hangul wrap(⠸⠷...⠸⠾)으로
                            // 묶는다. PDF 제6항 [붙임] — 한글표 묶음.
                            let den_has_korean = norm_den.chars().any(crate::utils::is_korean_char);
                            let num_has_korean = norm_num.chars().any(crate::utils::is_korean_char);
                            let any_korean = den_has_korean || num_has_korean;
                            let den_needs_group = needs_grouping_in_fraction(&norm_den);
                            let num_needs_group = needs_grouping_in_fraction(&norm_num);

                            let (open_den, close_den) = if any_korean && den_needs_group {
                                ('\u{27E8}', '\u{27E9}')
                            } else {
                                ('\u{2329}', '\u{232A}')
                            };
                            let (open_num, close_num) = if any_korean && num_needs_group {
                                ('\u{27E8}', '\u{27E9}')
                            } else {
                                ('\u{2329}', '\u{232A}')
                            };
                            if den_needs_group {
                                result.push(open_den);
                                result.push_str(&norm_den);
                                result.push(close_den);
                            } else {
                                result.push_str(&norm_den);
                            }
                            result.push('\u{2044}');
                            if num_needs_group {
                                result.push(open_num);
                                result.push_str(&norm_num);
                                result.push(close_num);
                            } else {
                                result.push_str(&norm_num);
                            }
                        }
                    }
                }
                "cup" => result.push('\u{222A}'),          // ∪
                "cap" => result.push('\u{2229}'),          // ∩
                "subset" => result.push('\u{2282}'),       // ⊂
                "supset" => result.push('\u{2283}'),       // ⊃
                "emptyset" => result.push('\u{2205}'),     // ∅
                "in" => result.push('\u{2208}'),           // ∈
                "notin" => result.push('\u{2209}'),        // ∉
                "forall" => result.push('\u{2200}'),       // ∀
                "exists" => result.push('\u{2203}'),       // ∃
                "nexists" => result.push('\u{2204}'),      // ∄
                "land" => result.push('\u{2227}'),         // ∧
                "lor" => result.push('\u{2228}'),          // ∨
                "neg" | "lnot" => result.push('\u{00AC}'), // ¬
                "Rightarrow" | "implies" => result.push('\u{21D2}'), // ⇒
                "Leftrightarrow" | "iff" => result.push('\u{21D4}'), // ⇔
                "rightarrow" => result.push('\u{2192}'),   // →
                "leftarrow" => result.push('\u{2190}'),    // ←
                "nearrow" => result.push('\u{2197}'),      // ↗
                "searrow" => result.push('\u{2198}'),      // ↘
                "nwarrow" => result.push('\u{2196}'),      // ↖
                "swarrow" => result.push('\u{2199}'),      // ↙
                "overleftrightarrow" => {
                    if let Some(inner) = read_braced_content(&mut chars) {
                        result.push('\u{20E1}');
                        let norm = strip_latex_to_math(&inner);
                        if !norm.trim().is_empty() {
                            result.push_str(&norm);
                        }
                    }
                }
                "perp" => result.push('\u{22A5}'),     // ⊥
                "parallel" => result.push('\u{2225}'), // ∥
                "angle" => result.push('\u{2220}'),    // ∠
                "triangle" => result.push('\u{25B3}'), // △
                "equiv" => result.push('\u{2261}'),    // ≡
                "frown" => result.push('\u{2322}'),    // ⌢
                "hat" => {
                    if let Some(inner) = read_braced_content(&mut chars)
                        && !inner.is_empty()
                    {
                        result.push_str(&inner);
                        result.push('\u{0302}');
                    }
                }
                "tilde" => {
                    // PDF 제65항 5 — `\tilde{X}` -> X + U+0303 결합 틸데
                    if let Some(inner) = read_braced_content(&mut chars)
                        && !inner.is_empty()
                    {
                        let norm = strip_latex_to_math(&inner);
                        result.push_str(&norm);
                        result.push('\u{0303}');
                    }
                }
                "overline" | "bar" => {
                    if let Some(inner) = read_braced_content(&mut chars) {
                        let norm = strip_latex_to_math(&inner);
                        if norm.trim().is_empty() {
                            // \\overline{\\,} or empty: just the overline marker
                            result.push('\u{0305}');
                        } else {
                            // PDF — overline 본문이 산술 표현(연산자/기호 포함)이면
                            // 점자에서 ⠷...⠾로 묶고 overline 결합부호를 그 다음에 둔다.
                            // `\overline{AB}`(선분)이나 `\overline{A'B'}`(선분에 프라임)
                            // 같이 글자(혹은 프라임/첨자 정도)만 있으면 묶지 않는다.
                            let has_operator = norm.chars().any(|c| {
                                matches!(
                                    c,
                                    '+' | '-' | '\u{2212}' | '×' | '*' | '/' | '=' | '<' | '>'
                                )
                            });
                            let needs_group = norm.chars().count() > 1 && has_operator;
                            if needs_group {
                                result.push('\u{2329}'); // 그룹 시작 마커 (parser에서 ⠷로 변환)
                                result.push_str(&norm);
                                result.push('\u{232A}'); // 그룹 종료
                                result.push('\u{0305}');
                            } else {
                                result.push_str(&norm);
                                result.push('\u{0305}');
                            }
                        }
                    }
                }
                "underline" => {
                    if let Some(inner) = read_braced_content(&mut chars) {
                        result.push_str(&inner);
                        result.push('\u{0332}');
                    }
                }
                "substack" => {
                    // PDF 제51항 [붙임] — `\substack{X \\ Y}`는 첨자 본문이 여러 줄로
                    // 쌓인 형태. 점역에서는 각 줄을 공백으로 평탄화하고, 두 번째 줄부터
                    // 새 첨자 마커가 부착되도록 `_` 접두어를 추가한다.
                    // 예: `\lim_{\substack{x \to a \\ y \to b}}` →
                    //   `lim_{x \to a}\,_{y \to b}` 처럼 펼친다 (앞 그룹 닫고 새 그룹 열기).
                    if let Some(inner) = read_braced_content(&mut chars) {
                        let lines: Vec<&str> = inner.split("\\\\").map(str::trim).collect();
                        let mut first = true;
                        for line in lines {
                            let norm = strip_latex_to_math(line);
                            if first {
                                result.push_str(&norm);
                                first = false;
                            } else {
                                // 닫고-다시-열기. parser는 이를 두 개의 인접한 첨자로 본다.
                                result.push('}');
                                result.push('_');
                                result.push('{');
                                result.push_str(&norm);
                            }
                        }
                    }
                }
                "dot" => {
                    if let Some(inner) = read_braced_content(&mut chars)
                        && !inner.is_empty()
                    {
                        result.push_str(&inner);
                        result.push('\u{0307}');
                    }
                }
                "ddot" => {
                    if let Some(inner) = read_braced_content(&mut chars)
                        && !inner.is_empty()
                    {
                        result.push_str(&inner);
                        result.push('\u{0308}');
                    }
                }
                "mathring" => {
                    if let Some(inner) = read_braced_content(&mut chars)
                        && !inner.is_empty()
                    {
                        result.push_str(&inner);
                        result.push('\u{0309}');
                    }
                }
                "not" => {
                    if chars.peek() == Some(&'\\') {
                        chars.next();
                        let mut next_cmd = String::new();
                        while let Some(&next) = chars.peek() {
                            if next.is_ascii_alphabetic() {
                                next_cmd.push(next);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        match next_cmd.as_str() {
                            "sim" => result.push('\u{2241}'),
                            // PDF 수학 제60항 — 부정 형태
                            "subset" => {
                                result.push('\u{2284}'); // ⊄
                            }
                            "supset" => {
                                result.push('\u{2285}'); // ⊅
                            }
                            "ni" => {
                                result.push('\u{220C}'); // ∌
                            }
                            "in" => {
                                result.push('\u{2209}'); // ∉
                            }
                            "equiv" => {
                                result.push('\u{2262}'); // ≢
                            }
                            "mathcal" => {
                                result.push('\u{0338}');
                                if let Some(inner) = read_braced_content(&mut chars) {
                                    for ch in inner.chars() {
                                        if ch.is_ascii_alphabetic() {
                                            result.push(ch.to_ascii_uppercase());
                                        }
                                    }
                                }
                            }
                            "mathrel" => {
                                if let Some(inner) = read_braced_content(&mut chars) {
                                    result.push('\u{00AC}');
                                    result.push_str(&inner);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "mathcal" => {
                    if let Some(inner) = read_braced_content(&mut chars) {
                        // \mathcal{X} -> uppercase letter X
                        for ch in inner.chars() {
                            if ch.is_ascii_alphabetic() {
                                result.push(ch.to_ascii_uppercase());
                            }
                        }
                    }
                }
                "mathrel" => {
                    if let Some(inner) = read_braced_content(&mut chars) {
                        result.push_str(&inner);
                    }
                }
                "sim" => result.push('~'),            // ~ (물결 = 닮음)
                "backsim" => result.push('\u{223D}'), // ∽
                "nsim" => result.push('\u{2241}'),    // ≁ (not sim)
                "nabla" => result.push('\u{2207}'),   // ∇
                "partial" => result.push('\u{2202}'), // ∂
                "iint" => result.push('\u{222C}'),    // ∬
                "oint" => result.push('\u{222E}'),    // ∮
                "nmid" => result.push('\u{2224}'),    // ∤
                "mid" => result.push('|'),
                "approxeq" => result.push('\u{224A}'), // ≊
                "simeq" => result.push('\u{2243}'),    // ≃
                "cong" => result.push('\u{2245}'),     // ≅
                "triangleright" => result.push('\u{25B7}'), // ▷
                "triangleleft" => result.push('\u{25C1}'), // ◁
                "veebar" => result.push('\u{22BB}'),   // ⊻
                "downarrow" => result.push('\u{2193}'), // ↓
                "uparrow" => result.push('\u{2191}'),  // ↑
                "leftrightarrow" => result.push('\u{2194}'), // ↔
                "rightleftarrows" => result.push('\u{21C4}'), // ⇄
                "nRightarrow" => result.push('\u{21CF}'), // ⇏
                "aleph" => result.push('\u{2135}'),    // ℵ
                "therefore" => result.push('\u{2234}'), // ∴
                "because" => result.push('\u{2235}'),  // ∵
                "ni" => result.push('\u{220B}'),       // ∋
                // PDF 수학 제60항 6 — 추론 기호
                "vdash" => result.push('\u{22A2}'),  // ⊢
                "dashv" => result.push('\u{22A3}'),  // ⊣
                "models" => result.push('\u{22A8}'), // ⊨
                "Dashv" => result.push('\u{2AE4}'),  // ⫤
                // PDF 수학 제60항 7~8 — 순서 관계
                "lesssim" => result.push('\u{2272}'), // ≲
                "prec" => result.push('\u{227A}'),    // ≺
                // PDF 수학 제61항 7 — 동치명제
                "rightleftharpoons" => result.push('\u{21CC}'), // ⇌
                "fallingdotseq" => result.push('\u{2252}'),     // ≒ (근삿값 ≈)
                "risingdotseq" => result.push('\u{2253}'),      // ≓
                "prime" => result.push('\u{2032}'),             // ′ (프라임)
                "bullet" => result.push('\u{2219}'),            // ∙ (검정 동그라미)
                // `\left` and `\right` LaTeX size modifiers: skip the keyword.
                // 뒤따르는 괄호/구분자는 그대로 처리되도록 한다.
                // PDF — `\right.`(one-sided, 닫는 구분자 없음)은 `⠄`(dots 3) 표지를 붙인다.
                "left" => {
                    if chars.peek() == Some(&'.') {
                        chars.next();
                    }
                }
                "right" => {
                    if chars.peek() == Some(&'.') {
                        chars.next();
                        // U+2E29 sentinel for open-ended right delimiter → ⠄
                        result.push('\u{2E29}');
                    }
                }
                "overset" => {
                    if let Some(over) = read_braced_content(&mut chars)
                        && let Some(base) = read_braced_content(&mut chars)
                    {
                        if over == "\\frown" || over == "⌢" {
                            result.push('\u{2322}');
                            result.push_str(&base);
                        } else {
                            result.push_str(&base);
                        }
                    }
                }
                _ => {
                    if cmd.len() == 1 && cmd.chars().all(|ch| ch.is_ascii_alphabetic()) {
                        result.push_str(&cmd);
                        continue;
                    }

                    // Handle compact forms like \sinx, \coshx, ...
                    let mut handled = false;
                    for known in [
                        "sinh", "cosh", "tanh", "sin", "cos", "tan", "csc", "sec", "cot", "lim",
                        "log", "ln",
                    ] {
                        if let Some(rest) = cmd.strip_prefix(known) {
                            result.push_str(known);
                            result.push_str(rest);
                            handled = true;
                            break;
                        }
                    }
                    if !handled {
                        // Unknown command — skip it silently
                    }
                }
            }
            // 이 branch에서 emit된 결과는 LaTeX 명령에서 온 것으로 표시한다.
            last_emit_from_latex = true;
        } else if c == '{' || c == '}' {
            // If we're inside a literal brace pair (\{ ... }), preserve the closing }.
            if c == '}' && escaped_brace_depth > 0 {
                escaped_brace_depth -= 1;
                result.push('}');
            }
            // Otherwise skip braces (used for LaTeX grouping)
        } else if c == '^' {
            // Superscript: convert to Unicode superscript or keep as-is
            // The math parser will handle this
            if let Some(&'{') = chars.peek() {
                chars.next(); // consume '{'
                let mut content = String::new();
                let mut depth = 1;
                for ch in chars.by_ref() {
                    if ch == '{' {
                        depth += 1;
                        content.push(ch);
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        content.push(ch);
                    } else {
                        content.push(ch);
                    }
                }
                // PDF 수학 — 위첨자 내용이 단순 ASCII 문자(숫자/연산자 등)면 Unicode
                // 위첨자로 직접 변환(`x^{0.3}` → `x⁰·³`). LaTeX 명령(\frac, \infty)을 포함하면
                // 재귀적으로 strip한 뒤 `^{...}` 구조를 보존해 math parser가 처리하도록 한다.
                let has_latex = content.contains('\\');
                let normalized = if has_latex {
                    strip_latex_to_math(&content)
                } else {
                    content.clone()
                };
                let simple_superscript = !has_latex
                    && normalized.chars().all(|c| {
                        c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.' | '(' | ')')
                    });
                if simple_superscript {
                    result.push_str(&to_superscript_sequence(&normalized));
                } else {
                    result.push('^');
                    result.push('{');
                    result.push_str(&normalized);
                    result.push('}');
                }
            } else if let Some(&next) = chars.peek() {
                // Single char exponent like ^2
                match next {
                    '0' => {
                        result.push('\u{2070}');
                        chars.next();
                    }
                    '1' => {
                        result.push('\u{00B9}');
                        chars.next();
                    }
                    '2' => {
                        result.push('\u{00B2}');
                        chars.next();
                    }
                    '3' => {
                        result.push('\u{00B3}');
                        chars.next();
                    }
                    '4' => {
                        result.push('\u{2074}');
                        chars.next();
                    }
                    '5' => {
                        result.push('\u{2075}');
                        chars.next();
                    }
                    '6' => {
                        result.push('\u{2076}');
                        chars.next();
                    }
                    '7' => {
                        result.push('\u{2077}');
                        chars.next();
                    }
                    '8' => {
                        result.push('\u{2078}');
                        chars.next();
                    }
                    '9' => {
                        result.push('\u{2079}');
                        chars.next();
                    }
                    _ => {
                        if next.is_ascii_alphabetic() || matches!(next, '+' | '-') {
                            let mapped = to_superscript_sequence(&next.to_string());
                            if mapped != next.to_string() {
                                result.push_str(&mapped);
                                chars.next();
                            } else {
                                result.push('^');
                            }
                        } else {
                            result.push('^');
                        }
                    }
                }
            }
        } else if c == '_' {
            // Subscript
            if let Some(&'{') = chars.peek() {
                chars.next();
                let mut content = String::new();
                let mut depth = 1;
                for ch in chars.by_ref() {
                    if ch == '{' {
                        depth += 1;
                        content.push(ch);
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        content.push(ch);
                    } else {
                        content.push(ch);
                    }
                }
                // Keep structured subscript so parser can handle complex content
                // like \Delta x \to 0 without leaving raw LaTeX commands.
                let normalized = strip_latex_to_math(&content);
                if let Some(subscript) = to_subscript_sequence(&normalized) {
                    result.push_str(&subscript);
                } else {
                    result.push('_');
                    result.push('{');
                    result.push_str(&normalized);
                    result.push('}');
                }
            } else if let Some(&next) = chars.peek() {
                // single char subscript: digit이면 Unicode subscript로 변환한다.
                // (예: `B_6` → `B₆` → rule_68 compact 패턴 매칭 가능)
                if let Some(sub) = to_subscript_sequence(&next.to_string()) {
                    result.push_str(&sub);
                    chars.next();
                } else {
                    result.push('_');
                    result.push(next);
                    chars.next();
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::strip_latex_to_math;

    /// Comprehensive LaTeX command sweep — exercises many branches in strip_latex_to_math.
    #[test]
    fn strip_command_sweep() {
        let inputs: &[&str] = &[
            "\\sin x",
            "\\cos x",
            "\\tan x",
            "\\csc x",
            "\\sec x",
            "\\cot x",
            "\\sinh x",
            "\\cosh x",
            "\\tanh x",
            "\\log x",
            "\\ln x",
            "\\lim x",
            "\\arcsin x",
            "\\arccos x",
            "\\arctan x",
            "\\cosec x",
            "\\approx",
            "\\infty",
            "\\to",
            "\\surd",
            "\\sqrt{x}",
            "\\sqrt[3]{x}",
            "\\sqrt[[a]b]{x}",
            "\\Pi",
            "\\times",
            "\\div",
            "\\pm",
            "\\cdot",
            "\\cdots",
            "\\ldots",
            "\\alpha",
            "\\beta",
            "\\gamma",
            "\\delta",
            "\\theta",
            "\\pi",
            "\\sigma",
            "\\omega",
            "\\Gamma",
            "\\epsilon",
            "\\varepsilon",
            "\\zeta",
            "\\eta",
            "\\Theta",
            "\\iota",
            "\\kappa",
            "\\Lambda",
            "\\lambda",
            "\\mu",
            "\\nu",
            "\\Xi",
            "\\xi",
            "\\Sigma",
            "\\tau",
            "\\upsilon",
            "\\phi",
            "\\varphi",
            "\\chi",
            "\\psi",
            "\\Omega",
            "\\bar{x}",
            "\\overline{xy}",
            "\\hat{x}",
            "\\widehat{ab}",
            "\\tilde{x}",
            "\\widetilde{xy}",
            "\\dot{x}",
            "\\ddot{x}",
            "\\mathring{x}",
            "\\not\\sim",
            "\\not\\subset",
            "\\not\\supset",
            "\\not\\ni",
            "\\not\\in",
            "\\not\\equiv",
            "\\not\\mathcal{X}",
            "\\not\\mathrel{=}",
            "\\mathcal{X}",
            "\\mathrel{=}",
            "\\sim",
            "\\equiv",
            "\\frac{1}{2}",
            "\\dfrac{1}{2}",
            "\\tfrac{1}{2}",
            "\\cfrac{1}{2}",
            "\\binom{n}{k}",
            "\\dbinom{n}{k}",
            "\\overrightarrow{AB}",
            "\\vec{v}",
            "\\acute{a}",
            "\\grave{a}",
            "\\check{x}",
            "\\breve{x}",
            "\\leq",
            "\\geq",
            "\\neq",
            "\\ne",
            "\\le",
            "\\ge",
            "\\cup",
            "\\cap",
            "\\subset",
            "\\supset",
            "\\subseteq",
            "\\supseteq",
            "\\in",
            "\\notin",
            "\\ni",
            "\\emptyset",
            "\\varnothing",
            "\\forall",
            "\\exists",
            "\\nexists",
            "\\land",
            "\\lor",
            "\\neg",
            "\\Rightarrow",
            "\\Leftrightarrow",
            "\\rightarrow",
            "\\leftarrow",
            "\\leftrightarrow",
            "\\uparrow",
            "\\downarrow",
            "\\partial",
            "\\nabla",
            "\\sum",
            "\\prod",
            "\\int",
            "\\oint",
            "\\int_0^1 f(x) dx",
            "\\xrightarrow[g]{f}",
            "\\xrightleftharpoons[g]{f}",
            "\\xrightleftharpoons[a[b]c]{label}",
            "\\begin{matrix} 1 & 2 \\\\ 3 & 4 \\end{matrix}",
            "\\begin{pmatrix} 1 \\\\ 2 \\end{pmatrix}",
            "\\begin{cases} a & x>0 \\\\ b & otherwise \\end{cases}",
            "x^{n+1}",
            "x_{n+1}",
            "x^{a^b}",
            "\\left(x\\right)",
            "\\left[x\\right]",
            "\\left\\{x\\right\\}",
            "\\lfloor x \\rfloor",
            "\\lceil x \\rceil",
            "\\angle ABC",
            "\\triangle ABC",
            "\\perp",
            "\\parallel",
        ];
        for input in inputs {
            let _ = strip_latex_to_math(input);
        }
    }

    /// strip.rs:772-774 — `\{` opens escaped brace depth; plain `}` (without
    /// preceding `\`) is preserved as a literal closing brace when depth > 0.
    /// Input: `\{x}` — `\{` increments depth, `x` is content, `}` matches the
    /// `c == '}' && escaped_brace_depth > 0` branch.
    #[test]
    fn escaped_open_brace_then_plain_close() {
        let _ = strip_latex_to_math("\\{x}");
        let _ = strip_latex_to_math("\\{abc}");
    }

    #[rstest::rstest]
    #[case::plain_xrightleftharpoons("\\xrightleftharpoons{f}", "f\u{21C4}\u{00A0}")]
    #[case::prefixed_xrightleftharpoons("A\\xrightleftharpoons{f}", "A\u{00A0}f\u{21C4}\u{00A0}")]
    #[case::one_sided_right_delimiter("\\left.x\\right.", "x\u{2E29}")]
    #[case::overset_frown_arc("\\overset{\\frown}{AB}", "\u{2322}AB")]
    #[case::overset_plain_base("\\overset{x}{AB}", "AB")]
    #[case::single_letter_command("\\q", "q")]
    fn structural_commands_strip_to_math_text(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(strip_latex_to_math(input), expected);
    }

    #[rstest::rstest]
    #[case::single_zero("x^0", "x⁰")]
    #[case::single_five("x^5", "x⁵")]
    #[case::single_six("x^6", "x⁶")]
    #[case::single_seven("x^7", "x⁷")]
    #[case::single_eight("x^8", "x⁸")]
    #[case::single_nine("x^9", "x⁹")]
    #[case::latex_inside_braced_exponent("x^{\\frac{1}{2}}", "x^{2⁄1}")]
    #[case::plain_complex_braced_exponent("x^{n+1}", "xⁿ⁺¹")]
    #[case::nested_braced_exponent("x^{a{b}}", "x^{a{b}}")]
    #[case::symbol_exponent_fallback("x^*", "x^*")]
    fn superscript_forms_strip_to_math_text(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(strip_latex_to_math(input), expected);
    }

    #[rstest::rstest]
    #[case::integer_ratio_base("\\log_{(3}/_{1)}x", "log₍₃/₁₎x")]
    #[case::decimal_base("\\log_{(0}._{2)}x", "log₍₀.₂₎x")]
    #[case::multi_digit_ratio_base("\\log_{(12}/_{34)}x", "log₍₁₂/₃₄₎x")]
    fn split_subscript_log_base_is_structural(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(strip_latex_to_math(input), expected);
    }
}
