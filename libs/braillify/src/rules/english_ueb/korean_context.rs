//! Korean-context UEB prefix matcher for 제28항 English letters.

use phf::phf_map;

use super::contraction::{ContractionMatch, ContractionRule};
use super::rule_10_3::StrongContractionRule;
use super::rule_10_4::StrongGroupsignRule;
use super::rule_10_6::LowerGroupsignRule;
use crate::unicode::decode_unicode;

/// §10.5/§10.6 restricted lower wordsigns/groupsigns whose Korean-context use is
/// governed by the [`Gate`] (be/in boundaries). The middle lower groupsigns
/// `ea bb cc ff gg` are sourced from the shared
/// [`super::rule_10_6::MIDDLE_LOWER_GROUPSIGNS`] (single source of truth), so only
/// `be`/`con` remain local here.
static KOREAN_RESTRICTED_LOWER: phf::Map<&'static str, u8> = phf_map! {
    "be" => decode_unicode('⠆'),
    "con" => decode_unicode('⠒'),
};

/// §10.7 initial-letter contractions (prefix cell ⠐ + first letter) that the
/// pure-English engine defers as pronunciation/morphology-dependent (see
/// [`super::rule_10_7`]). Inside an explicit Roman section (제37항) they are
/// applied per their §10.7 letter→cells definition. These are single-contraction
/// definitions — `every`/`knowledge`/`part` are then *derived* (`ever`+`y`,
/// `know`+`ledge`, `part`) rather than mapped whole-word.
static KOREAN_INITIAL_CONTRACTIONS: phf::Map<&'static str, &'static [u8]> = phf_map! {
    "ever" => &[decode_unicode('⠐'), decode_unicode('⠑')],
    "know" => &[decode_unicode('⠐'), decode_unicode('⠅')],
    "part" => &[decode_unicode('⠐'), decode_unicode('⠏')],
};

/// Korean-context prefix match request.
pub(crate) struct KoreanPrefixInput<'a> {
    pub(crate) word: &'a [char],
    pub(crate) pos: usize,
    pub(crate) wrap_active: bool,
    pub(crate) is_all_uppercase: bool,
    pub(crate) at_entry: bool,
    pub(crate) standalone_wordsign: bool,
}

/// Match result for a Korean-context UEB prefix.
pub(crate) struct KoreanPrefixMatch {
    pub(crate) cells: Vec<u8>,
    pub(crate) consumed: usize,
}

/// Match the current English position using the legacy Korean-context cascade.
pub(crate) fn match_korean_prefix(input: KoreanPrefixInput<'_>) -> Option<KoreanPrefixMatch> {
    let word = lowercase_word(input.word);
    if input.pos >= word.len() {
        return None;
    }

    if input.at_entry
        && input.standalone_wordsign
        && let Some(matched) = korean_wordsign_match(&word)
    {
        return Some(matched);
    }

    let gate = Gate::new(&word, &input);
    if gate.try_lower_entry
        && let Some(matched) = korean_lower_match(&word, input.pos)
    {
        return Some(matched);
    }
    if gate.try_strong
        && let Some(matched) = combined_strong_match(&word, input.pos)
    {
        return Some(matched);
    }
    if let Some(matched) = korean_ong_match(&word, input.pos) {
        return Some(matched);
    }
    if gate.try_lower_middle
        && let Some(matched) = korean_lower_match(&word, input.pos)
    {
        return Some(matched);
    }
    None
}

fn korean_wordsign_match(word: &[char]) -> Option<KoreanPrefixMatch> {
    let text = word_as_str(word)?;
    super::rule_10_1::wordsign(&text)
        .or_else(|| super::rule_10_2::wordsign(&text))
        .map(|cell| KoreanPrefixMatch {
            cells: vec![cell],
            consumed: word.len(),
        })
}

struct Gate {
    try_lower_entry: bool,
    try_lower_middle: bool,
    try_strong: bool,
}

impl Gate {
    fn new(word: &[char], input: &KoreanPrefixInput<'_>) -> Self {
        let be_boundary = boundary_non_alpha(word, input.pos, "be");
        let in_boundary = boundary_non_alpha(word, input.pos, "in");
        let whole_in_or_be =
            input.pos == 0 && matches!(word_as_str(word).as_deref(), Some("be" | "in"));
        let allow_lower = !(input.is_all_uppercase
            || (!input.wrap_active && be_boundary)
            || (!input.wrap_active && in_boundary)
            || (!input.wrap_active && whole_in_or_be));
        let allow_strong = !(input.is_all_uppercase
            || (!input.wrap_active && in_boundary)
            || (!input.wrap_active && whole_in_or_be && starts_with(word, input.pos, "in")));
        Self {
            try_lower_entry: input.at_entry && allow_lower,
            try_lower_middle: !input.at_entry && input.wrap_active && allow_lower,
            try_strong: allow_strong,
        }
    }
}

/// §10.7 initial-letter contraction match at `pos` (longest key wins).
fn korean_initial_match(word: &[char], pos: usize) -> Option<ContractionMatch> {
    let mut best: Option<(usize, &'static [u8])> = None;
    for (key, &cells) in KOREAN_INITIAL_CONTRACTIONS.entries() {
        let len = key.chars().count();
        if starts_with(word, pos, key) && best.is_none_or(|(bl, _)| len > bl) {
            best = Some((len, cells));
        }
    }
    best.map(|(consumed, cells)| ContractionMatch {
        cells: cells.to_vec(),
        consumed,
        priority: 55,
        protect_span: false,
    })
}

/// Best contraction at `pos` across the §10.7/§10.4/§10.3 rules — **longest
/// match wins**, ties broken by lower priority (the §10.10 preference rule). This
/// is what makes `rather` derive as `r a the(⠮) r` rather than `r a th(⠹) e r`.
fn combined_strong_match(word: &[char], pos: usize) -> Option<KoreanPrefixMatch> {
    [
        korean_initial_match(word, pos),
        LowerGroupsignRule.try_match(word, pos),
        StrongGroupsignRule.try_match(word, pos),
        StrongContractionRule.try_match(word, pos),
    ]
    .into_iter()
    .flatten()
    .max_by_key(|m| (m.consumed, u16::MAX - m.priority))
    .map(to_korean_match)
}

fn korean_lower_match(word: &[char], pos: usize) -> Option<KoreanPrefixMatch> {
    let core = LowerGroupsignRule.try_match(word, pos);
    let korean = match_korean_lower_table(word, pos);
    [core, korean]
        .into_iter()
        .flatten()
        .max_by_key(|m| (m.consumed, u16::MAX - m.priority))
        .map(to_korean_match)
}

fn match_korean_lower_table(word: &[char], pos: usize) -> Option<ContractionMatch> {
    let mut best: Option<(usize, u8)> = None;
    let entries = super::rule_10_6::MIDDLE_LOWER_GROUPSIGNS
        .entries()
        .chain(KOREAN_RESTRICTED_LOWER.entries());
    for (key, &cell) in entries {
        let len = key.chars().count();
        if starts_with(word, pos, key) && best.is_none_or(|(best_len, _)| len > best_len) {
            best = Some((len, cell));
        }
    }
    best.map(|(consumed, cell)| ContractionMatch {
        cells: vec![cell],
        consumed,
        priority: 70,
        protect_span: false,
    })
}

fn korean_ong_match(word: &[char], pos: usize) -> Option<KoreanPrefixMatch> {
    if !starts_with(word, pos, "ong") {
        return None;
    }
    let cells = super::rule_10_8::final_groupsign_cells("ong")?;
    Some(KoreanPrefixMatch {
        cells: cells.to_vec(),
        consumed: 3,
    })
}

fn to_korean_match(matched: ContractionMatch) -> KoreanPrefixMatch {
    KoreanPrefixMatch {
        cells: matched.cells,
        consumed: matched.consumed,
    }
}

fn lowercase_word(word: &[char]) -> Vec<char> {
    word.iter().map(|ch| ch.to_ascii_lowercase()).collect()
}

fn word_as_str(word: &[char]) -> Option<String> {
    word.iter()
        .all(|ch| ch.is_ascii_alphabetic())
        .then(|| word.iter().collect())
}

fn boundary_non_alpha(word: &[char], pos: usize, key: &str) -> bool {
    let len = key.chars().count();
    starts_with(word, pos, key)
        && word
            .get(pos + len)
            .is_none_or(|ch| !ch.is_ascii_alphabetic())
}

fn starts_with(word: &[char], pos: usize, key: &str) -> bool {
    let len = key.chars().count();
    pos + len <= word.len()
        && key
            .chars()
            .zip(&word[pos..pos + len])
            .all(|(lhs, rhs)| lhs == *rhs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::be_spells("be", 0, false, None)]
    #[case::in_spells("in", 0, false, None)]
    #[case::tea_has_no_middle_ea("tea", 1, false, None)]
    #[case::pyeongchang_ong("pyeongchang", 3, false, Some((vec![decode_unicode('⠰'), decode_unicode('⠛')], 3)))]
    // `part` = §10.7 initial contraction (whole word ⠐⠏); `every` derives as the
    // `ever`(⠐⠑) prefix + `y`, so the prefix matcher returns just ⠐⠑ here.
    #[case::part_initial("Part", 0, false, Some((vec![decode_unicode('⠐'), decode_unicode('⠏')], 4)))]
    #[case::ever_prefix("Every", 0, false, Some((vec![decode_unicode('⠐'), decode_unicode('⠑')], 4)))]
    #[case::wrap_active_in("in", 0, true, Some((vec![decode_unicode('⠔')], 2)))]
    fn matches_korean_context_prefixes(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] wrap_active: bool,
        #[case] expected: Option<(Vec<u8>, usize)>,
    ) {
        let chars: Vec<char> = word.chars().collect();
        let got = match_korean_prefix(KoreanPrefixInput {
            word: &chars,
            pos,
            wrap_active,
            is_all_uppercase: false,
            at_entry: pos == 0,
            standalone_wordsign: false,
        })
        .map(|matched| (matched.cells, matched.consumed));
        assert_eq!(got, expected);
    }

    #[rstest::rstest]
    #[case::alphabetic_you("you", decode_unicode('⠽'), 3)]
    #[case::strong_this("this", decode_unicode('⠹'), 4)]
    fn matches_standalone_wordsigns(
        #[case] word: &str,
        #[case] expected_cell: u8,
        #[case] expected_consumed: usize,
    ) {
        let chars: Vec<char> = word.chars().collect();
        let got = match_korean_prefix(KoreanPrefixInput {
            word: &chars,
            pos: 0,
            wrap_active: false,
            is_all_uppercase: false,
            at_entry: true,
            standalone_wordsign: true,
        })
        .map(|matched| (matched.cells, matched.consumed));
        assert_eq!(got, Some((vec![expected_cell], expected_consumed)));
    }

    #[test]
    fn korean_initial_match_finds_initial_contraction() {
        let chars: Vec<char> = std::hint::black_box("ever").chars().collect();
        let got = korean_initial_match(&chars, std::hint::black_box(0))
            .map(|matched| (matched.cells, matched.consumed));

        assert_eq!(
            got,
            Some((vec![decode_unicode('⠐'), decode_unicode('⠑')], 4))
        );
    }

    #[test]
    fn combined_strong_match_chooses_longer_contraction_over_groupsign() {
        let chars: Vec<char> = std::hint::black_box("rather").chars().collect();
        let got = combined_strong_match(&chars, std::hint::black_box(2))
            .map(|matched| (matched.cells, matched.consumed));

        assert_eq!(got, Some((vec![decode_unicode('⠮')], 3)));
    }

    #[rstest::rstest]
    #[case::core_middle_lower("enough", 0, decode_unicode('⠢'), 2)]
    #[case::korean_restricted_lower("become", 0, decode_unicode('⠆'), 2)]
    #[case::no_lower_match("xyz", 0, 0, 0)]
    fn korean_lower_match_paths(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] expected_cell: u8,
        #[case] expected_consumed: usize,
    ) {
        let chars: Vec<char> = std::hint::black_box(word).chars().collect();
        let got = korean_lower_match(&chars, std::hint::black_box(pos))
            .map(|matched| (matched.cells, matched.consumed));
        let expected = (expected_consumed > 0).then_some((vec![expected_cell], expected_consumed));
        assert_eq!(got, expected);
    }

    #[test]
    fn korean_lower_table_prefers_longest_runtime_match() {
        let word: Vec<char> = std::hint::black_box("become").chars().collect();
        let matched = match_korean_lower_table(&word, std::hint::black_box(0))
            .expect("restricted lower match should exist");

        assert_eq!(matched.consumed, 2);
        assert_eq!(matched.cells, vec![decode_unicode('⠆')]);
    }

    #[test]
    fn non_ascii_word_is_not_wordsign() {
        let chars: Vec<char> = "yoü".chars().collect();
        assert!(korean_wordsign_match(&chars).is_none());
    }

    #[test]
    fn prefix_match_rejects_position_past_word_end() {
        let word = std::hint::black_box("be");
        let chars: Vec<char> = word.chars().collect();
        let pos = std::hint::black_box(chars.len());

        assert!(
            match_korean_prefix(KoreanPrefixInput {
                word: &chars,
                pos,
                wrap_active: false,
                is_all_uppercase: false,
                at_entry: false,
                standalone_wordsign: false,
            })
            .is_none()
        );
    }

    #[test]
    fn lowercase_word_maps_ascii_uppercase() {
        assert_eq!(lowercase_word(&['B', 'e']), vec!['b', 'e']);
    }
}
