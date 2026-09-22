use phf::phf_map;

use crate::rules::RuleMeta;
use crate::rules::math::math_token_rule::UNDECLARED_MATH_RULE;
use crate::unicode::decode_unicode;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MathSymbolShortcut {
    pub(crate) cells: &'static [u8],
    pub(crate) fallback_meta: &'static RuleMeta,
}

macro_rules! math_meta {
    ($(($constant:ident, $section:literal, $name:literal, $description:literal)),+ $(,)?) => {
        $(
            pub(crate) static $constant: RuleMeta = RuleMeta {
                section: $section,
                subsection: None,
                name: $name,
                standard_ref: concat!("2024 Korean Braille Standard, 수학 제", $section, "항"),
                description: $description,
            };
        )+
    };
}

math_meta! {
    (META_2, "2", "math_arithmetic_operator", "Arithmetic operators"),
    (META_3, "3", "math_equality_symbol", "Equality symbols"),
    (META_4, "4", "math_comparison_symbol", "Comparison symbols"),
    (META_5, "5", "math_ratio_symbol", "Ratio and proportion symbols"),
    (META_7, "7", "math_fraction_symbol", "Fraction notation"),
    (META_9, "9", "math_repeating_decimal", "Repeating decimal marks"),
    (META_10, "10", "math_arrow_symbol", "Arrow symbols"),
    (META_13, "13", "math_greek_symbol", "Greek letters"),
    (META_15, "15", "math_custom_binary_operator", "Custom binary operators"),
    (META_16, "16", "math_base_subscript", "Base-notation subscripts"),
    (META_17, "17", "math_prime_mark", "Prime marks"),
    (META_18, "18", "math_superscript_symbol", "Superscript symbols"),
    (META_19, "19", "math_subscript_symbol", "Subscript symbols"),
    (META_21, "21", "math_absolute_value", "Absolute-value bars"),
    (META_22, "22", "math_root_symbol", "Root symbols"),
    (META_23, "23", "math_overline_symbol", "Overline and underline marks"),
    (META_24, "24", "math_sequence_brace", "Sequence braces"),
    (META_25, "25", "math_sigma_symbol", "Summation symbols"),
    (META_27, "27", "math_divisibility_symbol", "Divisibility symbols"),
    (META_28, "28", "math_norm_symbol", "Norm symbols"),
    (META_30, "30", "math_dot_congruence", "Dot-congruence symbols"),
    (META_31, "31", "math_asymptotic_equality", "Asymptotic equality"),
    (META_32, "32", "math_congruence_symbol", "Congruence symbols"),
    (META_33, "33", "math_geometric_operator", "Geometric operators"),
    (META_34, "34", "math_relation_symbol", "Relation symbols and their negations"),
    (META_35, "35", "math_segment_symbol", "Segment bar over two points"),
    (META_36, "36", "math_arc_symbol", "Arc symbol"),
    (META_37, "37", "math_line_symbol", "Bidirectional line symbols"),
    (META_38, "38", "math_ray_symbol", "Ray symbols, also used for vectors"),
    (META_39, "39", "math_angle_symbol", "Angle symbol"),
    (META_40, "40", "math_geometric_shape", "Geometric shapes"),
    (META_41, "41", "math_perpendicular_symbol", "Perpendicular symbols"),
    (META_42, "42", "math_similarity_symbol", "Similarity symbols"),
    (META_43, "43", "math_identity_symbol", "Identity symbols"),
    (META_44, "44", "math_parallel_symbol", "Parallel symbols"),
    (META_50, "50", "math_infinity_symbol", "Infinity"),
    (META_53, "53", "math_derivative_product", "Product signs in derivative formulas"),
    (META_54, "54", "math_partial_derivative", "Partial derivatives"),
    (META_55, "55", "math_nabla_symbol", "Nabla"),
    (META_56, "56", "math_integral_symbol", "Indefinite integrals"),
    (META_58, "58", "math_double_integral", "Double integrals"),
    (META_59, "59", "math_contour_integral", "Contour integrals"),
    (META_60, "60", "math_set_symbol", "Set and inference symbols"),
    (META_61, "61", "math_logic_symbol", "Logic symbols"),
    (META_64, "64", "math_hat_symbol", "Hat notation"),
    (META_65, "65", "math_miscellaneous_symbol", "Miscellaneous math symbols"),
}

pub(crate) static META_KOREAN_49: RuleMeta = RuleMeta {
    section: "49",
    subsection: None,
    name: "korean_sentence_punctuation_in_math",
    standard_ref: "2024 Korean Braille Standard, 한글 제49항",
    description: "Question and exclamation marks inside math input",
};
pub(crate) static META_KOREAN_50: RuleMeta = RuleMeta {
    section: "50",
    subsection: None,
    name: "korean_middle_dot_in_math",
    standard_ref: "2024 Korean Braille Standard, 한글 제50항",
    description: "Middle dot inside math input",
};
pub(crate) static META_KOREAN_51: RuleMeta = RuleMeta {
    section: "51",
    subsection: None,
    name: "korean_colon_in_math",
    standard_ref: "2024 Korean Braille Standard, 한글 제51항",
    description: "Colon inside math input",
};
pub(crate) static META_KOREAN_53: RuleMeta = RuleMeta {
    section: "53",
    subsection: None,
    name: "korean_ellipsis_in_math",
    standard_ref: "2024 Korean Braille Standard, 한글 제53항",
    description: "Ellipsis inside math input",
};
pub(crate) static META_KOREAN_59: RuleMeta = RuleMeta {
    section: "59",
    subsection: None,
    name: "korean_semicolon_in_math",
    standard_ref: "2024 Korean Braille Standard, 한글 제59항",
    description: "Semicolon inside math input",
};
pub(crate) static META_KOREAN_64: RuleMeta = RuleMeta {
    section: "64",
    subsection: None,
    name: "korean_enclosed_number_in_math",
    standard_ref: "2024 Korean Braille Standard, 한글 제64항",
    description: "Circled numbers inside math input",
};
pub(crate) static META_KOREAN_69_APPENDIX_2: RuleMeta = RuleMeta {
    section: "69",
    subsection: Some("붙임 2"),
    name: "korean_degree_symbol_in_math",
    standard_ref: "2024 Korean Braille Standard, 한글 제69항 [붙임 2]",
    description: "Degree sign inside math input",
};

pub(crate) static MATH_SYMBOL_VARIANT_METAS: &[&RuleMeta] = &[
    &META_2,
    &META_4,
    &META_5,
    &META_7,
    &META_9,
    &META_10,
    &META_13,
    &META_15,
    &META_16,
    &META_17,
    &META_18,
    &META_19,
    &META_21,
    &META_22,
    &META_23,
    &META_24,
    &META_25,
    &META_27,
    &META_28,
    &META_30,
    &META_31,
    &META_32,
    &META_33,
    &META_34,
    &META_35,
    &META_36,
    &META_37,
    &META_38,
    &META_39,
    &META_40,
    &META_41,
    &META_42,
    &META_43,
    &META_44,
    &META_50,
    &META_53,
    &META_54,
    &META_55,
    &META_56,
    &META_58,
    &META_59,
    &META_60,
    &META_61,
    &META_64,
    &META_65,
    &META_KOREAN_50,
    &META_KOREAN_53,
    &META_KOREAN_64,
    &META_KOREAN_69_APPENDIX_2,
    &UNDECLARED_MATH_RULE,
];

macro_rules! shortcut_map {
    ($($meta:expr => { $($symbol:expr => $cells:expr),+ $(,)? }),+ $(,)?) => {
        phf_map! {
            $($(
                $symbol => MathSymbolShortcut {
                    cells: $cells,
                    fallback_meta: $meta,
                },
            )+)+
        }
    };
}

static SHORTCUT_MAP: phf::Map<char, MathSymbolShortcut> = shortcut_map! {
    &META_KOREAN_64 => {
        '\u{2460}' => &[decode_unicode('⠼'), decode_unicode('⠂')],
        '\u{2461}' => &[decode_unicode('⠼'), decode_unicode('⠆')],
        '\u{2462}' => &[decode_unicode('⠼'), decode_unicode('⠒')],
        '\u{2463}' => &[decode_unicode('⠼'), decode_unicode('⠲')],
        '\u{2464}' => &[decode_unicode('⠼'), decode_unicode('⠢')],
        '\u{2465}' => &[decode_unicode('⠼'), decode_unicode('⠖')],
        '\u{2466}' => &[decode_unicode('⠼'), decode_unicode('⠶')],
        '\u{2467}' => &[decode_unicode('⠼'), decode_unicode('⠦')],
        '\u{2468}' => &[decode_unicode('⠼'), decode_unicode('⠔')],
        '\u{2469}' => &[decode_unicode('⠼'), decode_unicode('⠴')],
    },
    &META_2 => {
        '+' => &[decode_unicode('⠢')],
        '\u{2212}' => &[decode_unicode('⠔')],
        '\u{00D7}' => &[decode_unicode('⠡')],
        '\u{2A09}' => &[decode_unicode('⠡')],
        '\u{00F7}' => &[decode_unicode('⠌'), decode_unicode('⠌')],
        '\u{00B1}' => &[decode_unicode('⠢'), decode_unicode('⠔')],
    },
    &META_7 => {
        '/' => &[decode_unicode('⠸'), decode_unicode('⠌')],
        '\u{2500}' => &[decode_unicode('⠌')],
    },
    &META_3 => {
        '=' => &[decode_unicode('⠒'), decode_unicode('⠒')],
        '\u{2260}' => &[decode_unicode('⠨'), decode_unicode('⠒'), decode_unicode('⠒')],
        '\u{2252}' => &[decode_unicode('⠐'), decode_unicode('⠒'), decode_unicode('⠒')],
        '\u{2248}' => &[decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠈'), decode_unicode('⠔')],
    },
    &META_4 => {
        '>' => &[decode_unicode('⠢'), decode_unicode('⠢')],
        '<' => &[decode_unicode('⠔'), decode_unicode('⠔')],
        '\u{2265}' => &[decode_unicode('⠲'), decode_unicode('⠲')],
        '\u{2267}' => &[decode_unicode('⠲'), decode_unicode('⠲')],
        '\u{2264}' => &[decode_unicode('⠖'), decode_unicode('⠖')],
        '\u{2266}' => &[decode_unicode('⠖'), decode_unicode('⠖')],
        '\u{226E}' => &[decode_unicode('⠨'), decode_unicode('⠔'), decode_unicode('⠔')],
        '\u{226F}' => &[decode_unicode('⠨'), decode_unicode('⠢'), decode_unicode('⠢')],
        '\u{2270}' => &[decode_unicode('⠨'), decode_unicode('⠖'), decode_unicode('⠖')],
        '\u{2271}' => &[decode_unicode('⠨'), decode_unicode('⠲'), decode_unicode('⠲')],
    },
    &META_5 => {
        '\u{2236}' => &[decode_unicode('⠐'), decode_unicode('⠂')],
    },
    &META_38 => {
        '\u{2192}' => &[decode_unicode('⠒'), decode_unicode('⠕')],
        '\u{27F6}' => &[decode_unicode('⠒'), decode_unicode('⠕')],
        '\u{20D7}' => &[decode_unicode('⠒'), decode_unicode('⠕')],
    },
    &META_37 => {
        '\u{2194}' => &[decode_unicode('⠪'), decode_unicode('⠒'), decode_unicode('⠕')],
        '\u{20E1}' => &[decode_unicode('⠪'), decode_unicode('⠒'), decode_unicode('⠕')],
    },
    &META_10 => {
        '\u{2190}' => &[decode_unicode('⠪'), decode_unicode('⠒')],
        '\u{2191}' => &[decode_unicode('⠰'), decode_unicode('⠒'), decode_unicode('⠕')],
        '\u{2193}' => &[decode_unicode('⠘'), decode_unicode('⠒'), decode_unicode('⠕')],
        '\u{21D2}' => &[decode_unicode('⠒'), decode_unicode('⠒'), decode_unicode('⠕')],
        '\u{21D4}' => &[decode_unicode('⠪'), decode_unicode('⠒'), decode_unicode('⠒'), decode_unicode('⠕')],
        '\u{2196}' => &[decode_unicode('⠪'), decode_unicode('⠢')],
        '\u{2197}' => &[decode_unicode('⠔'), decode_unicode('⠕')],
        '\u{2198}' => &[decode_unicode('⠢'), decode_unicode('⠕')],
        '\u{2199}' => &[decode_unicode('⠪'), decode_unicode('⠔')],
    },
    &META_61 => {
        '\u{21C4}' => &[decode_unicode('⠪'), decode_unicode('⠶'), decode_unicode('⠕')],
        '\u{21CC}' => &[decode_unicode('⠪'), decode_unicode('⠶'), decode_unicode('⠕')],
        '\u{00AC}' => &[decode_unicode('⠈'), decode_unicode('⠔')],
        '\u{2200}' => &[decode_unicode('⠨'), decode_unicode('⠄')],
        '\u{2203}' => &[decode_unicode('⠨'), decode_unicode('⠢')],
        '\u{2204}' => &[decode_unicode('⠨'), decode_unicode('⠨'), decode_unicode('⠢')],
        '\u{2227}' => &[decode_unicode('⠹')],
        '\u{2228}' => &[decode_unicode('⠼')],
        '\u{22BB}' => &[decode_unicode('⠼'), decode_unicode('⠤')],
        '~' => &[decode_unicode('⠈'), decode_unicode('⠔')],
    },
    &META_17 => {
        '\u{2032}' => &[decode_unicode('⠤')],
        '\u{2033}' => &[decode_unicode('⠤'), decode_unicode('⠤')],
        '\u{2034}' => &[decode_unicode('⠤'), decode_unicode('⠤'), decode_unicode('⠤')],
    },
    &META_18 => {
        '\u{00B2}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠃')],
        '\u{00B3}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠉')],
        '\u{2074}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠙')],
        '\u{2075}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠑')],
        '\u{2077}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠛')],
        '\u{2079}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠊')],
        '\u{00B9}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠁')],
        '\u{2070}' => &[decode_unicode('⠘'), decode_unicode('⠼'), decode_unicode('⠚')],
        '\u{1D4F}' => &[decode_unicode('⠘'), decode_unicode('⠅')],
        '\u{1D50}' => &[decode_unicode('⠘'), decode_unicode('⠍')],
        '\u{02E3}' => &[decode_unicode('⠘'), decode_unicode('⠭')],
        '\u{207D}' => &[decode_unicode('⠘'), decode_unicode('⠦')],
        '\u{207E}' => &[decode_unicode('⠴')],
        '\u{207F}' => &[decode_unicode('⠘'), decode_unicode('⠝')],
        '\u{207B}' => &[decode_unicode('⠘'), decode_unicode('⠔')],
        '\u{207A}' => &[decode_unicode('⠘'), decode_unicode('⠢')],
    },
    &META_16 => {
        '\u{2080}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠚')],
        '\u{2081}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠁')],
        '\u{2082}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠃')],
        '\u{2083}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠉')],
        '\u{2084}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠙')],
        '\u{2085}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠑')],
        '\u{2086}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠋')],
        '\u{2087}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠛')],
        '\u{2088}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠓')],
        '\u{2089}' => &[decode_unicode('⠰'), decode_unicode('⠼'), decode_unicode('⠊')],
        '\u{208D}' => &[decode_unicode('⠰'), decode_unicode('⠦')],
        '\u{208E}' => &[decode_unicode('⠴')],
    },
    &META_19 => {
        '\u{2090}' => &[decode_unicode('⠰'), decode_unicode('⠁')],
        '\u{2098}' => &[decode_unicode('⠰'), decode_unicode('⠍')],
        '\u{2093}' => &[decode_unicode('⠰'), decode_unicode('⠭')],
        '\u{2099}' => &[decode_unicode('⠰'), decode_unicode('⠝')],
        '\u{208A}' => &[decode_unicode('⠰'), decode_unicode('⠢')],
    },
    &UNDECLARED_MATH_RULE => {
        '\u{2E29}' => &[decode_unicode('⠄')],
    },
    &META_34 => {
        '\u{0338}' => &[decode_unicode('⠨')],
        '\u{211B}' => &[decode_unicode('⠠'), decode_unicode('⠗')],
        '\u{2241}' => &[decode_unicode('⠨'), decode_unicode('⠈'), decode_unicode('⠔')],
    },
    &META_60 => {
        '\u{1D9C}' => &[decode_unicode('⠘'), decode_unicode('⠉')],
    },
    &META_61 => {
        '\u{21CF}' => &[decode_unicode('⠨'), decode_unicode('⠒'), decode_unicode('⠒'), decode_unicode('⠕')],
    },
    &META_7 => {
        '\u{2044}' => &[decode_unicode('⠌')],
    },
    &META_23 => {
        '_' => &[decode_unicode('⠠'), decode_unicode('⠤')],
        '\u{0332}' => &[decode_unicode('⠠'), decode_unicode('⠤')],
        '\u{0304}' => &[decode_unicode('⠈'), decode_unicode('⠉')],
        '\u{0305}' => &[decode_unicode('⠈'), decode_unicode('⠉')],
        '\u{00AF}' => &[decode_unicode('⠈'), decode_unicode('⠉')],
    },
    &META_21 => {
        '|' => &[decode_unicode('⠳')],
    },
    &META_KOREAN_69_APPENDIX_2 => {
        '\u{00B0}' => &[decode_unicode('⠴'), decode_unicode('⠙')],
    },
    &META_KOREAN_50 => {
        '\u{00B7}' => &[decode_unicode('⠐')],
    },
    &META_KOREAN_53 => {
        '…' => &[decode_unicode('⠠'), decode_unicode('⠠'), decode_unicode('⠠')],
        '⋯' => &[decode_unicode('⠠'), decode_unicode('⠠'), decode_unicode('⠠')],
    },
    &META_22 => {
        '\u{221A}' => &[decode_unicode('⠜')],
    },
    &META_27 => {
        '\u{2223}' => &[decode_unicode('⠳')],
        '\u{2224}' => &[decode_unicode('⠨'), decode_unicode('⠳')],
    },
    &META_39 => {
        '\u{2220}' => &[decode_unicode('⠹')],
    },
    &META_41 => {
        '\u{22A5}' => &[decode_unicode('⠴'), decode_unicode('⠄')],
    },
    &META_44 => {
        '\u{2225}' => &[decode_unicode('⠰'), decode_unicode('⠆')],
        '\u{2AFD}' => &[decode_unicode('⠰'), decode_unicode('⠆')],
    },
    &META_42 => {
        '\u{223D}' => &[decode_unicode('⠠'), decode_unicode('⠄')],
    },
    &META_43 => {
        '\u{2261}' => &[decode_unicode('⠶'), decode_unicode('⠶')],
    },
    &META_50 => {
        '\u{221E}' => &[decode_unicode('⠿')],
    },
    &META_56 => {
        '\u{222B}' => &[decode_unicode('⠮')],
    },
    &META_59 => {
        '\u{222E}' => &[decode_unicode('⠾')],
    },
    &META_58 => {
        '\u{222C}' => &[decode_unicode('⠮'), decode_unicode('⠮')],
    },
    &META_55 => {
        '\u{2207}' => &[decode_unicode('⠸'), decode_unicode('⠩')],
    },
    &META_54 => {
        '\u{2202}' => &[decode_unicode('⠫')],
    },
    &META_60 => {
        '\u{2208}' => &[decode_unicode('⠖')],
        '\u{220B}' => &[decode_unicode('⠲')],
        '\u{2209}' => &[decode_unicode('⠨'), decode_unicode('⠖')],
        '\u{220C}' => &[decode_unicode('⠨'), decode_unicode('⠲')],
        '\u{2282}' => &[decode_unicode('⠖'), decode_unicode('⠂')],
        '\u{2283}' => &[decode_unicode('⠐'), decode_unicode('⠲')],
        '\u{2284}' => &[decode_unicode('⠨'), decode_unicode('⠖'), decode_unicode('⠂')],
        '\u{2285}' => &[decode_unicode('⠨'), decode_unicode('⠐'), decode_unicode('⠲')],
        '\u{2205}' => &[decode_unicode('⠨'), decode_unicode('⠋')],
        '\u{222A}' => &[decode_unicode('⠬')],
        '\u{2229}' => &[decode_unicode('⠩')],
        '\u{22A2}' => &[decode_unicode('⠸'), decode_unicode('⠒')],
        '\u{22A3}' => &[decode_unicode('⠈'), decode_unicode('⠸'), decode_unicode('⠒')],
        '\u{22A8}' => &[decode_unicode('⠘'), decode_unicode('⠸'), decode_unicode('⠒')],
        '\u{2AE4}' => &[decode_unicode('⠨'), decode_unicode('⠸'), decode_unicode('⠒')],
        '\u{2272}' => &[decode_unicode('⠔'), decode_unicode('⠔'), decode_unicode('⠈'), decode_unicode('⠔')],
        '\u{227A}' => &[decode_unicode('⠔'), decode_unicode('⠔')],
    },
    &META_65 => {
        '\u{2234}' => &[decode_unicode('⠠'), decode_unicode('⠡')],
        '\u{2235}' => &[decode_unicode('⠈'), decode_unicode('⠌')],
        '\u{2135}' => &[decode_unicode('⠗'), decode_unicode('⠋')],
        '\u{FF03}' => &[decode_unicode('⠸'), decode_unicode('⠹')],
        '\u{0303}' => &[decode_unicode('⠈'), decode_unicode('⠈'), decode_unicode('⠔')],
        '\u{0308}' => &[decode_unicode('⠈'), decode_unicode('⠲'), decode_unicode('⠲')],
        '\u{0309}' => &[decode_unicode('⠈'), decode_unicode('⠈'), decode_unicode('⠔')],
        '\u{030A}' => &[decode_unicode('⠈'), decode_unicode('⠈'), decode_unicode('⠔')],
    },
    &META_30 => {
        '\u{224A}' => &[decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠒')],
    },
    &META_31 => {
        '\u{2243}' => &[decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠒')],
    },
    &META_32 => {
        '\u{2245}' => &[decode_unicode('⠈'), decode_unicode('⠔'), decode_unicode('⠒'), decode_unicode('⠒')],
    },
    &META_33 => {
        '\u{25B7}' => &[decode_unicode('⠸'), decode_unicode('⠜')],
        '\u{25C1}' => &[decode_unicode('⠸'), decode_unicode('⠣')],
    },
    &META_40 => {
        '\u{25A1}' => &[decode_unicode('⠸'), decode_unicode('⠶')],
        '\u{25B3}' => &[decode_unicode('⠸'), decode_unicode('⠬')],
        '\u{25B1}' => &[decode_unicode('⠸'), decode_unicode('⠌'), decode_unicode('⠌')],
        '\u{23E2}' => &[decode_unicode('⠸'), decode_unicode('⠌'), decode_unicode('⠡')],
        '\u{2302}' => &[decode_unicode('⠸'), decode_unicode('⠪'), decode_unicode('⠅')],
        '\u{2394}' => &[decode_unicode('⠸'), decode_unicode('⠪'), decode_unicode('⠕')],
        '\u{29BE}' => &[decode_unicode('⠸'), decode_unicode('⠴'), decode_unicode('⠴')],
        '\u{2206}' => &[decode_unicode('⠸'), decode_unicode('⠬')],
        '\u{2219}' => &[decode_unicode('⠸'), decode_unicode('⠲')],
    },
    &META_25 => {
        '\u{2211}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠎')],
    },
    &META_15 => {
        '\u{2295}' => &[decode_unicode('⠸'), decode_unicode('⠢')],
        '\u{2296}' => &[decode_unicode('⠸'), decode_unicode('⠔')],
        '\u{2297}' => &[decode_unicode('⠸'), decode_unicode('⠡')],
        '\u{2217}' => &[decode_unicode('⠸'), decode_unicode('⠣')],
        '\u{2218}' => &[decode_unicode('⠸'), decode_unicode('⠴')],
    },
    &META_13 => {
        '\u{03B1}' => &[decode_unicode('⠨'), decode_unicode('⠁')],
        '\u{03B2}' => &[decode_unicode('⠨'), decode_unicode('⠃')],
        '\u{03B3}' => &[decode_unicode('⠨'), decode_unicode('⠛')],
        '\u{03B4}' => &[decode_unicode('⠨'), decode_unicode('⠙')],
        '\u{03B5}' => &[decode_unicode('⠨'), decode_unicode('⠑')],
        '\u{03B6}' => &[decode_unicode('⠨'), decode_unicode('⠵')],
        '\u{03B7}' => &[decode_unicode('⠨'), decode_unicode('⠱')],
        '\u{03B8}' => &[decode_unicode('⠨'), decode_unicode('⠹')],
        '\u{03B9}' => &[decode_unicode('⠨'), decode_unicode('⠊')],
        '\u{03BA}' => &[decode_unicode('⠨'), decode_unicode('⠅')],
        '\u{03BB}' => &[decode_unicode('⠨'), decode_unicode('⠇')],
        '\u{03BC}' => &[decode_unicode('⠨'), decode_unicode('⠍')],
        '\u{03BD}' => &[decode_unicode('⠨'), decode_unicode('⠝')],
        '\u{03BE}' => &[decode_unicode('⠨'), decode_unicode('⠭')],
        '\u{03BF}' => &[decode_unicode('⠨'), decode_unicode('⠕')],
        '\u{03C0}' => &[decode_unicode('⠨'), decode_unicode('⠏')],
        '\u{03C1}' => &[decode_unicode('⠨'), decode_unicode('⠗')],
        '\u{03C3}' => &[decode_unicode('⠨'), decode_unicode('⠎')],
        '\u{03C4}' => &[decode_unicode('⠨'), decode_unicode('⠞')],
        '\u{03C5}' => &[decode_unicode('⠨'), decode_unicode('⠥')],
        '\u{03C6}' => &[decode_unicode('⠨'), decode_unicode('⠋')],
        '\u{03C7}' => &[decode_unicode('⠨'), decode_unicode('⠯')],
        '\u{03C8}' => &[decode_unicode('⠨'), decode_unicode('⠽')],
        '\u{03C9}' => &[decode_unicode('⠨'), decode_unicode('⠺')],
        '\u{0391}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠁')],
        '\u{0392}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠃')],
        '\u{0393}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠛')],
        '\u{0394}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠙')],
        '\u{0395}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠑')],
        '\u{0396}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠵')],
        '\u{0397}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠱')],
        '\u{0398}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠹')],
        '\u{0399}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠊')],
        '\u{039A}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠅')],
        '\u{039B}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠇')],
        '\u{039C}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠍')],
        '\u{039D}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠝')],
        '\u{039E}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠭')],
        '\u{039F}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠕')],
        '\u{03A0}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠏')],
        '\u{03A1}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠗')],
        '\u{03A3}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠎')],
        '\u{03A4}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠞')],
        '\u{03A5}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠥')],
        '\u{03A6}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠋')],
        '\u{03A7}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠯')],
        '\u{03A8}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠽')],
        '\u{03A9}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠺')],
        '\u{2126}' => &[decode_unicode('⠠'), decode_unicode('⠨'), decode_unicode('⠺')],
    },
    &META_35 => {
        '\u{203E}' => &[decode_unicode('⠈'), decode_unicode('⠉')],
    },
    &META_36 => {
        '\u{2322}' => &[decode_unicode('⠈'), decode_unicode('⠪')],
    },
    &META_64 => {
        '\u{0302}' => &[decode_unicode('⠈'), decode_unicode('⠈'), decode_unicode('⠢')],
    },
    &META_28 => {
        '\u{2016}' => &[decode_unicode('⠳'), decode_unicode('⠳')],
    },
    &META_9 => {
        '\u{0307}' => &[decode_unicode('⠈')],
    },
};

pub fn encode_char_math_symbol_shortcut(text: char) -> Result<&'static [u8], String> {
    math_symbol_shortcut(text).map(|shortcut| shortcut.cells)
}

pub(crate) fn math_symbol_shortcut(text: char) -> Result<&'static MathSymbolShortcut, String> {
    SHORTCUT_MAP
        .get(&text)
        .ok_or_else(|| "Invalid math symbol character".to_string())
}

pub fn is_math_symbol_char(text: char) -> bool {
    SHORTCUT_MAP.contains_key(&text)
}

#[cfg(test)]
mod test {
    use super::*;

    const UNRESOLVED_SYMBOLS: &[char] = &['∏', '⇏', '≁', 'ᶜ', 'ℛ', '⁄', '⸩'];

    #[test]
    fn every_resolved_shortcut_declares_a_real_fallback_article() {
        let missing = SHORTCUT_MAP.entries().find(|(symbol, shortcut)| {
            !UNRESOLVED_SYMBOLS.contains(symbol) && shortcut.fallback_meta.section == "?"
        });

        assert!(
            missing.is_none(),
            "resolved shortcut without article: {missing:?}"
        );
    }

    /// `⸩` stands in for LaTeX's `\right.` null delimiter, which prints nothing
    /// for an article to govern, so it keeps the placeholder rather than
    /// borrowing an article by resemblance.
    #[test]
    fn the_null_delimiter_keeps_the_honest_placeholder() {
        assert_eq!(SHORTCUT_MAP[&'⸩'].fallback_meta.section, "?");
    }

    /// 국립국어원 ruled on 2026-09-21 that the n-ary product cannot be
    /// transcribed: the standard never mentions it. Its cells are those of
    /// Greek capital pi, which makes borrowing them look reasonable and is
    /// exactly why the table must not carry it.
    #[test]
    fn the_n_ary_product_is_not_transcribable() {
        assert!(!SHORTCUT_MAP.contains_key(&'∏'));
        assert!(encode_char_math_symbol_shortcut('∏').is_err());
    }

    /// Each of these was identified by matching its cells against the notation
    /// printed in the standard, not by searching for the character itself:
    /// `⠌` is the 분수표 of 제7항 1, `⠠⠗`/`⠨⠈⠔` are 관계가있다/관계가없다 of
    /// 제34항, `⠘⠉` is 여집합 of 제60항 5, `⠨⠒⠒⠕` is 항진명제의 부정 of 제61항 4.
    #[rstest::rstest]
    #[case::fraction_slash('⁄', "7")]
    #[case::script_r('ℛ', "34")]
    #[case::not_similar('≁', "34")]
    #[case::negation_overlay('\u{0338}', "34")]
    #[case::superscript_c('ᶜ', "60")]
    #[case::not_implies('⇏', "61")]
    fn cell_matched_shortcuts_name_their_article(#[case] symbol: char, #[case] section: &str) {
        assert_eq!(SHORTCUT_MAP[&symbol].fallback_meta.section, section);
    }

    /// 제35항 to 제39항 run 선분 `@c`, 호 `@[`, 직선 `[3O`, 반직선 `3O`, 각 `?`,
    /// one article each and in that order. Three of these marks sat one article
    /// away from the one that defines them, which nothing caught because the
    /// cells were right either way. The overline is the segment bar of 제35항,
    /// not 제36항's arc; the two-headed arrow above a pair is 제37항's line, not
    /// a ray; and the single-headed one is 제38항's ray, which its 붙임 also
    /// lends to vectors, rather than 제39항's angle.
    #[rstest::rstest]
    #[case::segment_bar('\u{203E}', "35")]
    #[case::arc('\u{2322}', "36")]
    #[case::line_above('\u{20E1}', "37")]
    #[case::line_arrow('\u{2194}', "37")]
    #[case::ray_above('\u{20D7}', "38")]
    #[case::ray_arrow('\u{2192}', "38")]
    #[case::angle('\u{2220}', "39")]
    fn geometry_marks_cite_the_article_that_defines_them(
        #[case] symbol: char,
        #[case] section: &str,
    ) {
        assert_eq!(SHORTCUT_MAP[&symbol].fallback_meta.section, section);
    }

    /// 제23항 gives the bar over a variable — 켤레 복소수 and 평균값 — the same
    /// `@c` cells as 제35항's segment bar, so the two are told apart by code
    /// point alone: a combining or spacing macron marks a variable, while the
    /// overline spans a pair of points.
    #[rstest::rstest]
    #[case::combining_macron('\u{0304}')]
    #[case::combining_overline('\u{0305}')]
    #[case::spacing_macron('\u{00AF}')]
    fn a_bar_over_a_variable_stays_with_article_23(#[case] symbol: char) {
        assert_eq!(SHORTCUT_MAP[&symbol].fallback_meta.section, "23");
        assert_eq!(SHORTCUT_MAP[&symbol].cells, SHORTCUT_MAP[&'\u{203E}'].cells);
    }

    /// A second code point for a symbol the standard already defines means the
    /// same thing, so it takes the same cells and the same article. Chemistry
    /// writes its reaction arrow long and its product sign n-ary; the ohm sign
    /// is stronger still, being canonically equivalent to capital omega, so
    /// Unicode itself forbids treating the two as different characters.
    #[rstest::rstest]
    #[case::long_rightwards_arrow('\u{27F6}', '\u{2192}')]
    #[case::n_ary_times('\u{2A09}', '\u{00D7}')]
    #[case::ohm_sign('\u{2126}', '\u{03A9}')]
    fn a_glyph_variant_matches_the_symbol_it_varies(#[case] variant: char, #[case] base: char) {
        assert_eq!(SHORTCUT_MAP[&variant].cells, SHORTCUT_MAP[&base].cells);
        assert_eq!(
            SHORTCUT_MAP[&variant].fallback_meta.section,
            SHORTCUT_MAP[&base].fallback_meta.section
        );
    }

    /// 제27항 writes 나누어떨어진다 as `\` and negates it to `.\`, so the plain
    /// sign is the negated one without its leading dot.
    #[test]
    fn divides_is_the_undotted_form_of_does_not_divide() {
        let divides = SHORTCUT_MAP[&'\u{2223}'].cells;
        let does_not = SHORTCUT_MAP[&'\u{2224}'].cells;
        assert_eq!(does_not, [decode_unicode('⠨'), divides[0]]);
    }

    /// `is_math_symbol_char` true 케이스 — 연산자/그리스/집합/미적분 기호 전체.
    #[rstest::rstest]
    // basic operators
    #[case('+')]
    #[case('−')]
    #[case('×')]
    #[case('÷')]
    #[case('=')]
    // greek
    #[case('α')]
    #[case('π')]
    #[case('ω')]
    // set / logic
    #[case('∈')]
    #[case('∅')]
    #[case('∪')]
    #[case('∩')]
    // calculus
    #[case('∫')]
    #[case('∞')]
    #[case('√')]
    fn is_math_symbol_char_recognizes_math_chars(#[case] ch: char) {
        assert!(is_math_symbol_char(ch));
    }

    /// `is_math_symbol_char` false 케이스 — ASCII 알파벳은 수학 기호 아님.
    #[test]
    fn is_math_symbol_char_rejects_ascii_letter() {
        assert!(!is_math_symbol_char('a'));
    }

    /// `encode_char_math_symbol_shortcut` 표 매핑.
    /// - `²` (제곱) → `⠘⠼⠃` (^#b, 3-cell)
    /// - `≥` (크거나 같음) → `⠲⠲` (2-cell)
    /// - `≤` (작거나 같음) → `⠖⠖` (2-cell)
    #[rstest::rstest]
    #[case('²', &['⠘', '⠼', '⠃'][..])]
    #[case('≥', &['⠲', '⠲'][..])]
    #[case('≤', &['⠖', '⠖'][..])]
    fn encode_char_math_symbol_shortcut_table(
        #[case] input: char,
        #[case] expected_chars: &[char],
    ) {
        let expected: Vec<u8> = expected_chars.iter().copied().map(decode_unicode).collect();
        assert_eq!(
            encode_char_math_symbol_shortcut(input).unwrap(),
            expected.as_slice()
        );
    }
}
