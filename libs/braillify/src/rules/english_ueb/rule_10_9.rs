//! §10.9 Shortforms.
//!
//! The table is the UEB 2024 §10.9 / Appendix 1 base shortform list.  The value
//! is written in the same ASCII braille notation used by the rulebook examples:
//! letters are alphabet cells, while symbols such as `*`, `/`, `2`, `3`, `]`,
//! `\`, and `?` denote embedded UEB contractions in the abbreviation.

use phf::phf_map;

use super::contraction::{ContractionEngine, ContractionMatch};
use super::rule_10_13::WordDivision;
use crate::english::encode_english;
use crate::unicode::decode_unicode;

static SHORTFORMS: phf::Map<&'static str, &'static str> = phf_map! {
    "about" => "ab", "above" => "abv", "according" => "ac", "across" => "acr",
    "after" => "af", "afternoon" => "afn", "afterward" => "afw", "again" => "ag",
    "against" => "ag/", "almost" => "alm", "already" => "alr", "also" => "al",
    "although" => "al?", "altogether" => "alt", "always" => "alw",
    "because" => "2c", "before" => "2f", "behind" => "2h", "below" => "2l",
    "beneath" => "2n", "beside" => "2s", "between" => "2t", "beyond" => "2y",
    "blind" => "bl", "braille" => "brl", "children" => "*n",
    "conceive" => "3cv", "conceiving" => "3cvg", "could" => "cd",
    "deceive" => "dcv", "deceiving" => "dcvg", "declare" => "dcl", "declaring" => "dclg",
    "either" => "ei", "first" => "f/", "friend" => "fr", "good" => "gd", "great" => "grt",
    "herself" => "h]f", "him" => "hm", "himself" => "hmf", "immediate" => "imm",
    "its" => "xs", "itself" => "xf", "letter" => "lr", "little" => "ll",
    "much" => "m*", "must" => "m/", "myself" => "myf", "necessary" => "nec",
    "neither" => "nei", "o'clock" => "o'c", "oneself" => "\"of", "ourselves" => "\\rvs",
    "paid" => "pd", "perceive" => "p]cv", "perceiving" => "p]cvg", "perhaps" => "p]h",
    "quick" => "qk", "receive" => "rcv", "receiving" => "rcvg", "rejoice" => "rjc",
    "rejoicing" => "rjcg", "said" => "sd", "should" => "%d", "such" => "s*",
    "themselves" => "!mvs", "thyself" => "?yf", "today" => "td", "together" => "tgr",
    "tomorrow" => "tm", "tonight" => "tn", "would" => "wd", "your" => "yr",
    "yourself" => "yrf", "yourselves" => "yrvs",
};

/// UEB §10.9.1: a complete shortform word standing alone uses its abbreviation.
pub fn whole_word_cells(word: &str) -> Option<Vec<u8>> {
    notation_cells(SHORTFORMS.get(word).copied()?)
}

/// A literal all-letter abbreviation that collides with a pure-letter shortform
/// needs a grade-1 indicator before normal letter encoding (§10.9.7).
pub fn is_pure_shortform_abbreviation(word: &str) -> bool {
    SHORTFORMS
        .values()
        .any(|abbr| abbr.chars().all(|ch| ch.is_ascii_lowercase()) && *abbr == word)
}

/// Encode a word as the §10.10.2 cell-minimising contraction sequence.
///
/// §10.10.2 ("give preference to the groupsign which causes a word to occupy
/// fewer cells") is global cell minimisation, solved as a shortest-path DP from
/// the end of the word: `cost[pos]` is the fewest cells that can encode
/// `word[pos..]`, and `back[pos]` records the move (cells + characters consumed)
/// achieving it. Greedy longest-match fails the overlap cases — in `bastion` the
/// 2-letter `st` blocks the cheaper `s`+`tion`.
///
/// §10.10.1 ("unless other rules apply") makes the structural rules primary, so
/// before cell counts are compared [`candidate_moves`] removes the contractions
/// they forbid: the word-initial `ing` (§10.4.3), the `en` overlapping an `ence`
/// in encea/enced/encer (§10.10.6), and any generic contraction that would split
/// a morpheme/pronunciation-validated span (§10.11, via `protect_span`). Among
/// equal-cost moves the lower `priority` wins (§10.10.3–.7), then the longer match.
pub fn encode_with_longer_shortforms(
    word: &[char],
    contractions: &ContractionEngine,
    suppress_initial_ing: bool,
) -> Option<Vec<u8>> {
    encode_with_constraints(
        word,
        contractions,
        suppress_initial_ing,
        true,
        None,
        false,
        true,
        false,
    )
}

pub fn encode_with_optional_longer_shortforms(
    word: &[char],
    contractions: &ContractionEngine,
    suppress_initial_ing: bool,
    restricted_prefix_boundary: bool,
    allow_longer_shortforms: bool,
) -> Option<Vec<u8>> {
    encode_with_constraints(
        word,
        contractions,
        suppress_initial_ing,
        restricted_prefix_boundary,
        None,
        false,
        allow_longer_shortforms,
        false,
    )
}

/// UEB §13.2.3: anglicised foreign words, phrases, proper names and titles use
/// UEB contractions unless a contraction would distort pronunciation/structure.
/// This path is for unrecorded Roman-script loan/proper words in English context:
/// it keeps the ordinary §10.10 structural filters, but permits embedded
/// shortform candidates beyond the normal English-word whitelist.
pub fn encode_anglicised_word(
    word: &[char],
    contractions: &ContractionEngine,
    suppress_initial_ing: bool,
    restricted_prefix_boundary: bool,
) -> Option<Vec<u8>> {
    encode_with_constraints(
        word,
        contractions,
        suppress_initial_ing,
        restricted_prefix_boundary,
        None,
        false,
        true,
        true,
    )
}

/// §10.13: encode a divided word as one logical word while preventing contractions
/// that cross or are otherwise barred at the line-division point.
pub fn encode_with_division(
    word: &[char],
    contractions: &ContractionEngine,
    division: WordDivision,
    first_line_has_upper_prefix: bool,
) -> Option<Vec<u8>> {
    if division.index == 6 && starts_with(word, 0, "herein") {
        let mut out = vec![
            decode_unicode('⠐'),
            encode_english('h').ok()?,
            decode_unicode('⠔'),
        ];
        super::rule_10_13::append_break(&mut out, true);
        if word.get(division.index..) == Some(&['a', 'f', 't', 'e', 'r']) {
            out.extend([encode_english('a').ok()?, encode_english('f').ok()?]);
            return Some(out);
        }
        if word.get(division.index..) == Some(&['b', 'e', 'l', 'o', 'w']) {
            out.extend([
                encode_english('b').ok()?,
                encode_english('e').ok()?,
                encode_english('l').ok()?,
                decode_unicode('⠪'),
            ]);
            return Some(out);
        }
        out.extend(encode_with_constraints(
            &word[division.index..],
            contractions,
            false,
            true,
            None,
            first_line_has_upper_prefix,
            true,
            false,
        )?);
        return Some(out);
    }
    encode_with_constraints(
        word,
        contractions,
        false,
        true,
        Some(division),
        first_line_has_upper_prefix,
        true,
        false,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "DP constraint switches are independent rule gates"
)]
fn encode_with_constraints(
    word: &[char],
    contractions: &ContractionEngine,
    suppress_initial_ing: bool,
    restricted_prefix_boundary: bool,
    division: Option<WordDivision>,
    first_line_has_upper_prefix: bool,
    allow_longer_shortforms: bool,
    relax_shortforms: bool,
) -> Option<Vec<u8>> {
    let n = word.len();
    // §10.11.1: a contraction must not bridge the seam of a compound word. Look up
    // this word's compound seams once (empty for non-compounds → they contract).
    let word_string: String = word.iter().collect();
    let seams = super::compound::compound_seams(&word_string);
    // §10.11 (§10.10.1): positions strictly inside a protected span — a
    // morpheme/pronunciation-validated contraction (`part`, a kept `in`, `con`…)
    // must not be split by a cheaper generic contraction starting inside it.
    let mut inside_protected = vec![false; n];
    let mut max_reach = vec![0usize; n];
    for start in 0..n {
        // A gated contraction pre-empted by a longer contraction starting earlier
        // and covering this position is never used, so it must not protect
        // anything: in `hea·the·nesse` the `the` (pos 3–5) covers the `e` of the
        // `e·n` at pos 5, so that `en` is never chosen and the overlapping `ness`
        // must stay free.
        let preempted = (0..start).any(|s| max_reach[s] > start);
        for m in contractions.matches_at(word, start) {
            max_reach[start] = max_reach[start].max(start + m.consumed);
            if m.protect_span && !preempted {
                let end = (start + m.consumed).min(n);
                inside_protected[(start + 1)..end].fill(true);
            }
        }
    }
    // §10.10.2 cell-minimising DP from the end of the word.
    let mut cost = vec![usize::MAX; n + 1];
    // §10.10.3–.7: on a cell-count tie the sequence using the most-preferred
    // (lowest-priority) contraction wins — a decision about the WHOLE remaining
    // path, not just the immediate move. `path_priority[pos]` is the lowest
    // priority number used along the best path from `pos`, so a higher-preference
    // initial-letter contraction one step later (`e`→`name`, 55) beats a
    // lower-preference groupsign that overlaps its start here (`en`, 70):
    // `re·name·d`, not `r·en·amed`; `mis·time·d`, not `mis·st·imed`.
    let mut path_priority = vec![u16::MAX; n + 1];
    let mut back: Vec<Option<(Vec<u8>, usize)>> = vec![None; n + 1];
    cost[n] = 0;
    for pos in (0..n).rev() {
        // Best candidate so far: (total cells, path priority, consumed, cells).
        let mut best: Option<(usize, u16, usize, Vec<u8>)> = None;
        for (cells, consumed, priority) in candidate_moves(
            word,
            pos,
            contractions,
            suppress_initial_ing,
            restricted_prefix_boundary,
            &inside_protected,
            &seams,
            division,
            first_line_has_upper_prefix,
            allow_longer_shortforms,
            relax_shortforms,
        ) {
            let next = pos + consumed;
            let total = cells.len() + cost[next];
            // The preference of the whole remaining path: the best contraction in
            // this move or anything the tail already chose.
            let this_priority = priority.min(path_priority[next]);
            let better = best.as_ref().is_none_or(|(bt, bp, bc, _)| {
                total < *bt
                    || (total == *bt && this_priority < *bp)
                    || (total == *bt && this_priority == *bp && consumed > *bc)
            });
            if better {
                best = Some((total, this_priority, consumed, cells));
            }
        }
        let (total, pp, consumed, cells) = best?;
        cost[pos] = total;
        path_priority[pos] = pp;
        back[pos] = Some((cells, consumed));
    }
    // Reconstruct the chosen sequence from the start.
    let mut out = Vec::with_capacity(cost[0]);
    let mut pos = 0;
    while pos < n {
        let (cells, consumed) = back[pos].as_ref()?;
        out.extend(cells.iter().copied());
        pos += consumed;
        if division.is_some_and(|d| pos == d.index) {
            super::rule_10_13::append_break(&mut out, true);
        }
    }
    Some(out)
}

/// The candidate moves at `pos` for the §10.10.2 DP, after the §10.10.1
/// structural filters: an embedded §10.9 shortform, every §10.x contraction match
/// (minus those a structural rule forbids), and the §4.1/§4.2 single-letter
/// fallback. Each is `(cells, characters consumed, priority)`.
#[expect(
    clippy::too_many_arguments,
    reason = "DP candidate filtering keeps rule inputs explicit"
)]
fn candidate_moves(
    word: &[char],
    pos: usize,
    contractions: &ContractionEngine,
    suppress_initial_ing: bool,
    restricted_prefix_boundary: bool,
    inside_protected: &[bool],
    seams: &[usize],
    division: Option<WordDivision>,
    first_line_has_upper_prefix: bool,
    allow_longer_shortforms: bool,
    relax_shortforms: bool,
) -> Vec<(Vec<u8>, usize, u16)> {
    let mut moves = Vec::new();
    // §10.9 longer-word shortform placement (preferred on a cost tie → priority 0).
    if allow_longer_shortforms {
        let longer = if relax_shortforms {
            relaxed_longer_match(word, pos)
        } else if let Some(d) = division {
            longer_match_for_division(word, pos, d)
        } else {
            longer_match(word, pos)
        };
        if let Some((len, cells)) = longer
            && division.is_none_or(|d| !d.blocks_span(pos, len))
        {
            moves.push((cells, len, 0));
        }
    }
    if relax_shortforms && let Some((cells, len)) = anglicised_initial_contraction(word, pos) {
        moves.push((cells, len, 55));
    }
    let protected_here = inside_protected[pos];
    for m in contractions.matches_at(word, pos) {
        // §10.11.1: a GROUPSIGN must not bridge a compound-word seam —
        // `an[t·h]ill`, `cart[·h]orse`, `nor[the]ast` spell the bridging digraph
        // out. An initial-letter contraction (§10.7 `upon`, priority 55) and a
        // whole-word sign for the compound itself (`cannot`→⠸⠉) are NOT groupsigns,
        // so they keep their contraction; `seams` is empty for non-compounds.
        if m.consumed >= 2
            && m.priority != 55
            && m.consumed != word.len()
            && seams.iter().any(|&s| pos < s && s < pos + m.consumed)
        {
            continue;
        }
        // §10.11.5: `ence` at the START of a new component is just as misleading as
        // one that bridges through the seam: `electro|encephalogram` spells
        // `en` + `ce…`, not initial `ence`. Other final-letter groupsigns remain
        // available at a suffix/component start when they do not bridge the seam
        // (`atone|ment`, `never|the|less`).
        if seams.contains(&pos)
            && word.get(pos..pos + m.consumed) == Some(&['e', 'n', 'c', 'e'])
            && super::rule_10_8::final_groupsign_cells(
                &word[pos..pos + m.consumed].iter().collect::<String>(),
            )
            .is_some()
        {
            continue;
        }
        // §10.4.3 with §10.11.6 exceptions: when `ing` starts a component exposed by
        // a prefix seam (`dis|ingenuous`), it is at a component beginning and spells
        // `in` + `g`, not the strong `ing` groupsign.
        if seams.contains(&pos)
            && m.consumed == 3
            && word.get(pos..pos + 3) == Some(&['i', 'n', 'g'])
        {
            continue;
        }
        // §10.4.3: the `ing` groupsign is never used at the start of a word — drop
        // it so the `in` lower groupsign (⠔) + `g` is chosen instead.
        if suppress_initial_ing && pos == 0 && m.consumed == 3 && word.starts_with(&['i', 'n', 'g'])
        {
            continue;
        }
        // §10.6.2: restricted lower groupsigns `be`/`con`/`dis` are used only at
        // the beginning of a word; a slash or internal case split is not a word
        // beginning (`concave/convex`, `conCUR`, `MetroDisco`).
        if !restricted_prefix_boundary && pos == 0 && m.priority == 65 {
            continue;
        }
        // §10.10.8: select the groupsign that more nearly approximates usual
        // pronunciation. In these printed exceptions `st` is not the spoken cluster,
        // so `th` is preferred.
        if m.consumed == 2
            && word.get(pos..pos + 2) == Some(&['s', 't'])
            && matches!(
                word,
                ['a', 's', 't', 'h', 'm', 'a'] | ['i', 's', 't', 'h', 'm', 'u', 's']
            )
        {
            continue;
        }
        // §10.6.10: an all-lower-sign word ending before apostrophe must not end
        // with a final lower groupsign unless surrounding non-quote indicators
        // contribute an upper-dot sign (handled by the engine).
        if final_lower_groupsign_ends_unqualified_lower_sequence(word, pos, &m, contractions) {
            continue;
        }
        // §10.10.6: use the `ence` final groupsign in `encea`/`enced`/`encer` —
        // drop the overlapping 2-cell `en` lower groupsign so `ence` (a cell tie)
        // is the one the minimiser keeps (`Sp·ence·r`, `comm·ence·d`).
        if m.consumed == 2
            && word.get(pos) == Some(&'e')
            && word.get(pos + 1) == Some(&'n')
            && is_ence_context(word, pos)
        {
            continue;
        }
        // §10.11: a generic contraction may not start strictly inside a protected
        // span, so a morpheme/pronunciation-validated contraction is never split
        // by a cheaper one (`a·part·heid`, `captain·ess`).
        if protected_here && !m.protect_span && m.consumed >= 2 {
            continue;
        }
        if relax_shortforms && anglicised_blocks_match(word, pos, &m) {
            continue;
        }
        if division.is_some_and(|d| {
            d.blocks_span(pos, m.consumed)
                || d.blocks_initial_ing(word, pos, m.consumed)
                || d.blocks_restricted_lower(word, pos, m.consumed)
                || d.blocks_middle_lower(word, pos, m.consumed)
                || (d.blocks_final_letter(pos, m.consumed)
                    && super::rule_10_8::final_groupsign_cells(
                        &word[pos..pos + m.consumed].iter().collect::<String>(),
                    )
                    .is_some())
                || d.blocks_lower_sequence(
                    word,
                    pos,
                    m.consumed,
                    &m.cells,
                    first_line_has_upper_prefix,
                )
        }) {
            continue;
        }
        moves.push((m.cells, m.consumed, m.priority));
    }
    // §4.2 accent / §4.1 single letter — always available so the DP never stalls.
    if let Some(cells) = super::rule_12::early_letter(word[pos]) {
        moves.push((cells, 1, u16::MAX));
    } else if let Some(cells) = super::rule_4::accent_cells(word[pos]) {
        moves.push((cells, 1, u16::MAX));
    } else if let Ok(cell) = encode_english(word[pos]) {
        moves.push((vec![cell], 1, u16::MAX));
    }
    moves
}

/// §13.2.3 anglicised words may use ordinary UEB contractions even when CMUdict
/// has no entry for the borrowed/proper word.  Initial-letter contractions whose
/// English phonology gate cannot fire for an unrecorded word are safe when the
/// printed consonant cluster exposes the English word (`-had-` before a vowel in
/// `Ferhadija`-shaped names).
fn anglicised_initial_contraction(word: &[char], pos: usize) -> Option<(Vec<u8>, usize)> {
    if pos > 0
        && word.get(pos..pos + 3) == Some(&['h', 'a', 'd'])
        && word
            .get(pos + 3)
            .is_some_and(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'))
    {
        return Some((vec![decode_unicode('⠸'), decode_unicode('⠓')], 3));
    }
    None
}

/// §13.2.3 still obeys §10.10.8 pronunciation: Romance `ce` before a front vowel
/// in anglicised loans/names is not the English final-letter `ance/ence` unit, so
/// a final-letter groupsign starting at the preceding vowel would distort it.
fn anglicised_blocks_match(word: &[char], pos: usize, m: &ContractionMatch) -> bool {
    m.priority == 80
        && (matches!(
            word.get(pos..pos + m.consumed),
            Some(['a' | 'e', 'n', 'c', 'e'])
        ) || matches!(word.get(pos..pos + m.consumed), Some(['s', 'i', 'o', 'n'])))
        && word
            .get(pos + m.consumed)
            .is_some_and(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 't' | 'd'))
}

/// §10.10.6: whether `pos` begins the letters-sequence `encea`/`enced`/`encer`.
fn is_ence_context(word: &[char], pos: usize) -> bool {
    word.get(pos..pos + 5).is_some_and(|s| {
        s[0] == 'e' && s[1] == 'n' && s[2] == 'c' && s[3] == 'e' && matches!(s[4], 'a' | 'd' | 'r')
    })
}

fn final_lower_groupsign_ends_unqualified_lower_sequence(
    word: &[char],
    pos: usize,
    m: &ContractionMatch,
    contractions: &ContractionEngine,
) -> bool {
    pos > 0
        && pos + m.consumed == word.len()
        && m.consumed >= 2
        && cells_have_no_upper_dots(&m.cells)
        && all_lower_sequence_prefix_cells(word, pos, contractions).is_some()
}

pub(crate) fn all_lower_sequence_cells(
    word: &[char],
    contractions: &ContractionEngine,
) -> Option<Vec<u8>> {
    all_lower_sequence_prefix_cells(word, word.len(), contractions)
}

fn all_lower_sequence_prefix_cells(
    word: &[char],
    end: usize,
    contractions: &ContractionEngine,
) -> Option<Vec<u8>> {
    let mut back: Vec<Option<(usize, Vec<u8>)>> = vec![None; end + 1];
    back[0] = Some((0, Vec::new()));
    for pos in 0..end {
        if back[pos].is_none() {
            continue;
        }
        for m in contractions.matches_at(word, pos) {
            let next = pos + m.consumed;
            if next <= end && cells_have_no_upper_dots(&m.cells) {
                back[next] = Some((pos, m.cells));
            }
        }
    }
    let mut out = Vec::new();
    let mut pos = end;
    while pos > 0 {
        let (prev, cells) = back[pos].as_ref()?;
        out.extend(cells.iter().rev().copied());
        pos = *prev;
    }
    out.reverse();
    Some(out)
}

fn cells_have_no_upper_dots(cells: &[u8]) -> bool {
    cells.iter().all(|cell| cell & 0b0000_1001 == 0)
}

fn longer_match(word: &[char], pos: usize) -> Option<(usize, Vec<u8>)> {
    if pos == 0 && starts_with(word, 0, "herein") && is_herein_exception(word) {
        return Some((6, notation_cells("\"h9")?));
    }
    let mut best: Option<(usize, Vec<u8>)> = None;
    for (shortform, notation) in SHORTFORMS.entries() {
        let len = shortform.len();
        if pos + len <= word.len()
            && starts_with(word, pos, shortform)
            && word.len() > len
            && longer_use_allowed(word, pos, shortform)
            && best.as_ref().is_none_or(|(best_len, _)| len > *best_len)
        {
            best = Some((len, notation_cells(notation)?));
        }
    }
    best
}

pub fn shortform_part_cells(word: &[char], pos: usize) -> Option<(usize, Vec<u8>)> {
    if pos == 0 && starts_with(word, 0, "herein") && is_herein_exception(word) {
        return Some((6, notation_cells("\"h9")?));
    }
    let mut best: Option<(usize, Vec<u8>)> = None;
    for (shortform, notation) in SHORTFORMS.entries() {
        let len = shortform.len();
        if pos + len <= word.len()
            && starts_with(word, pos, shortform)
            && word.len() > len
            && longer_use_allowed(word, pos, shortform)
            && best.as_ref().is_none_or(|(best_len, _)| len > *best_len)
        {
            best = Some((len, notation_cells(notation)?));
        }
    }
    best
}

fn relaxed_longer_match(word: &[char], pos: usize) -> Option<(usize, Vec<u8>)> {
    let mut best: Option<(usize, Vec<u8>)> = None;
    for (shortform, notation) in SHORTFORMS.entries() {
        let len = shortform.len();
        if pos + len <= word.len()
            && starts_with(word, pos, shortform)
            && word.len() > len
            && (relaxed_shortform_allowed(shortform) || longer_use_allowed(word, pos, shortform))
            && best.as_ref().is_none_or(|(best_len, _)| len > *best_len)
        {
            best = Some((len, notation_cells(notation)?));
        }
    }
    best
}

/// §10.9.3 in §13.2.3 anglicised words: relaxed contraction mode keeps
/// legitimate embedded forms such as `letter` in `Newsletter`, but must not make
/// arbitrary proper-name strings read as common shortform words (`Portlittle`,
/// `Bisquick`, `Goodena`).
fn relaxed_shortform_allowed(shortform: &str) -> bool {
    !matches!(shortform, "good" | "little" | "quick")
}

fn longer_match_for_division(
    word: &[char],
    pos: usize,
    division: WordDivision,
) -> Option<(usize, Vec<u8>)> {
    if pos == 0 && starts_with(word, 0, "herein") && is_herein_exception(word) {
        return Some((6, notation_cells("\"h9")?));
    }
    let mut best: Option<(usize, Vec<u8>)> = None;
    for (shortform, notation) in SHORTFORMS.entries() {
        let len = shortform.len();
        if pos + len <= word.len()
            && starts_with(word, pos, shortform)
            && (longer_use_allowed(word, pos, shortform)
                || division_shortform_allowed(word, pos, shortform, division))
            && best.as_ref().is_none_or(|(best_len, _)| len > *best_len)
        {
            best = Some((len, notation_cells(notation)?));
        }
    }
    best
}

fn division_shortform_allowed(
    word: &[char],
    pos: usize,
    shortform: &str,
    division: WordDivision,
) -> bool {
    let end = pos + shortform.len();
    match shortform {
        "friend" => pos == division.index && end == word.len(),
        "after" => pos == division.index && end == word.len(),
        "herein" => end == division.index,
        "immediate" => end == division.index,
        "necessary" => pos == division.index,
        _ => false,
    }
}

fn longer_use_allowed(word: &[char], pos: usize, shortform: &str) -> bool {
    let word_string: String = word.iter().collect();
    if super::rule_10_9_list::listed_or_added_s(&word_string) {
        return true;
    }
    match shortform {
        "braille" | "great" => true,
        "children" => !is_followed_by_vowel_or_y(word, pos, shortform),
        "above" | "afternoon" | "afterward" => true,
        "paid" => pos > 0,
        // §10.9.3: `about` may be used within a longer word. Beyond the
        // about-face / about-turn / …-abouts compounds, it also ends the locative
        // `-about` adverbs `here·about`, `there·about`, `where·about`.
        "about" => is_about_compound(word, pos) || has_locative_about_prefix(word, pos),
        "good" => pos == 0 && good_prefix_allowed(word),
        "quick" => pos == 0 && quick_prefix_allowed(word),
        // §10.9.3: `such` is generative in listed compounds (`suchlike`) or
        // ending `some·such`; ordinary names like `Suchet` take the `ch` groupsign.
        "such" => {
            super::rule_10_9_list::listed_or_added_s(&word_string)
                || matches!(&word[..pos], ['s', 'o', 'm', 'e'])
        }
        "blind" | "first" | "friend" | "letter" | "little" => {
            pos == 0 && !is_followed_by_vowel_or_y(word, pos, shortform)
        }
        "below" => pos == 0 && !is_followed_by_vowel_or_y(word, pos, shortform),
        _ => false,
    }
}

fn is_herein_exception(word: &[char]) -> bool {
    matches!(
        suffix_after(word, 0, "herein").as_deref(),
        Some("before" | "below")
    )
}

fn is_about_compound(word: &[char], pos: usize) -> bool {
    matches!(
        suffix_after(word, pos, "about").as_deref(),
        Some("face" | "faced" | "facer" | "facing" | "turn" | "turned")
    ) || (pos > 0 && matches!(suffix_after(word, pos, "about").as_deref(), Some("s")))
}

fn has_locative_about_prefix(word: &[char], pos: usize) -> bool {
    let prefix = &word[..pos];
    prefix == ['h', 'e', 'r', 'e']
        || prefix == ['t', 'h', 'e', 'r', 'e']
        || prefix == ['w', 'h', 'e', 'r', 'e']
}

fn good_prefix_allowed(word: &[char]) -> bool {
    !is_followed_by_vowel_or_y(word, 0, "good")
        || matches!(suffix_after(word, 0, "good").as_deref(), Some("afternoon"))
}

fn quick_prefix_allowed(word: &[char]) -> bool {
    !is_followed_by_vowel_or_y(word, 0, "quick")
        || matches!(suffix_after(word, 0, "quick").as_deref(), Some("er" | "ly"))
}

fn suffix_after(word: &[char], pos: usize, prefix: &str) -> Option<String> {
    let end = pos + prefix.len();
    (end <= word.len()).then(|| word[end..].iter().collect())
}

fn is_followed_by_vowel_or_y(word: &[char], pos: usize, shortform: &str) -> bool {
    word.get(pos + shortform.len())
        .is_some_and(|ch| matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u' | 'y'))
}

fn starts_with(word: &[char], pos: usize, needle: &str) -> bool {
    needle
        .chars()
        .zip(&word[pos..])
        .all(|(expected, actual)| expected == *actual)
}

fn notation_cells(notation: &str) -> Option<Vec<u8>> {
    notation.chars().map(notation_cell).collect()
}

fn notation_cell(ch: char) -> Option<u8> {
    match ch {
        'a'..='z' => encode_english(ch).ok(),
        '*' => Some(decode_unicode('⠡')),
        '%' => Some(decode_unicode('⠩')),
        '?' => Some(decode_unicode('⠹')),
        '\\' => Some(decode_unicode('⠳')),
        '/' => Some(decode_unicode('⠌')),
        '2' => Some(decode_unicode('⠆')),
        '3' => Some(decode_unicode('⠒')),
        '9' => Some(decode_unicode('⠔')),
        ']' => Some(decode_unicode('⠻')),
        '!' => Some(decode_unicode('⠮')),
        '"' => Some(decode_unicode('⠐')),
        '\'' => Some(decode_unicode('⠄')),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::contraction::{ContractionMatch, ContractionRule};
    use super::*;

    struct TestRule {
        pattern: &'static [char],
        cell: char,
        priority: u16,
        protect_span: bool,
    }

    impl ContractionRule for TestRule {
        fn try_match(&self, word: &[char], pos: usize) -> Option<ContractionMatch> {
            word.get(pos..pos + self.pattern.len())
                .is_some_and(|slice| slice == self.pattern)
                .then(|| ContractionMatch {
                    cells: vec![decode_unicode(self.cell)],
                    consumed: self.pattern.len(),
                    priority: self.priority,
                    protect_span: self.protect_span,
                })
        }
    }

    fn engine_with(rule: TestRule) -> ContractionEngine {
        let mut engine = ContractionEngine::default();
        engine.register(Box::new(rule));
        engine
    }

    fn chars(word: &str) -> Vec<char> {
        word.chars().collect()
    }

    fn cells(s: &str) -> Vec<u8> {
        s.chars().map(decode_unicode).collect()
    }

    #[rstest::rstest]
    #[case::about("about", "⠁⠃")]
    #[case::against("against", "⠁⠛⠌")]
    #[case::because("because", "⠆⠉")]
    #[case::should("should", "⠩⠙")]
    #[case::ourselves("ourselves", "⠳⠗⠧⠎")]
    fn whole_shortform_cells_follow_standard_notation(#[case] word: &str, #[case] expected: &str) {
        let expected_cells: Vec<u8> = expected.chars().map(decode_unicode).collect();
        assert_eq!(whole_word_cells(word), Some(expected_cells));
    }

    #[rstest::rstest]
    #[case::gd("gd")]
    #[case::wd("wd")]
    #[case::rjc("rjc")]
    fn pure_abbreviations_are_collisions(#[case] word: &str) {
        assert!(is_pure_shortform_abbreviation(word));
    }

    #[rstest::rstest]
    #[case::hereinafter("hereinafter", 0, "herein", 6, true)]
    #[case::hereinbelow("hereinbelow", 0, "herein", 6, true)]
    #[case::herein_midline("hereinafter", 0, "herein", 4, false)]
    fn section_10_13_12_keeps_herein_shortform_at_division(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] shortform: &str,
        #[case] break_at: usize,
        #[case] expected: bool,
    ) {
        let chars: Vec<char> = word.chars().collect();
        assert_eq!(
            division_shortform_allowed(&chars, pos, shortform, WordDivision { index: break_at }),
            expected
        );
    }

    #[rstest::rstest]
    #[case::hereinafter("hereinafter", "⠐⠓⠔⠤\n⠁⠋")]
    #[case::hereinbelow("hereinbelow", "⠐⠓⠔⠤\n⠃⠑⠇⠪")]
    #[case::hereinx("hereinx", "⠐⠓⠔⠤\n⠭")]
    fn encode_with_division_handles_herein_exceptions(#[case] word: &str, #[case] expected: &str) {
        let word = chars(word);
        let got = encode_with_division(
            &word,
            &ContractionEngine::default(),
            WordDivision { index: 6 },
            false,
        );
        let expected: Vec<u8> = expected
            .chars()
            .map(|c| if c == '\n' { 255 } else { decode_unicode(c) })
            .collect();
        assert_eq!(got, Some(expected));
    }

    #[rstest::rstest]
    #[case::blocks_span("abcd", &['b', 'c'], 1, WordDivision { index: 2 })]
    #[case::blocks_initial_ing("thing", &['i', 'n', 'g'], 2, WordDivision { index: 2 })]
    #[case::blocks_restricted_lower("because", &['b', 'e'], 0, WordDivision { index: 2 })]
    #[case::blocks_middle_lower("peanut", &['e', 'a'], 1, WordDivision { index: 3 })]
    #[case::blocks_final_letter("abation", &['a', 't', 'i', 'o', 'n'], 2, WordDivision { index: 2 })]
    #[case::blocks_lower_sequence("a--", &['-'], 2, WordDivision { index: 1 })]
    fn candidate_moves_drops_matches_for_division_rules(
        #[case] word: &str,
        #[case] pattern: &'static [char],
        #[case] pos: usize,
        #[case] division: WordDivision,
    ) {
        let word = chars(word);
        let engine = engine_with(TestRule {
            pattern,
            cell: '⠆',
            priority: 65,
            protect_span: false,
        });
        let moves = candidate_moves(
            &word,
            pos,
            &engine,
            false,
            true,
            &vec![false; word.len()],
            &[],
            Some(division),
            false,
            false,
            false,
        );
        assert!(moves.iter().all(|(cells, consumed, _)| {
            *consumed != pattern.len() || cells != &vec![decode_unicode('⠆')]
        }));
    }

    #[rstest::rstest]
    #[case::friend_at_break("penfriend", 3, "friend", 3, true)]
    #[case::after_at_break("hereinafter", 6, "after", 6, true)]
    #[case::immediate_before_break("immediatecare", 0, "immediate", 9, true)]
    #[case::necessary_after_break("unnecessary", 2, "necessary", 2, true)]
    #[case::good_not_division_exception("goodness", 0, "good", 4, false)]
    fn division_shortform_allowed_only_for_listed_divisions(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] shortform: &str,
        #[case] break_at: usize,
        #[case] expected: bool,
    ) {
        let chars: Vec<char> = word.chars().collect();
        assert_eq!(
            division_shortform_allowed(&chars, pos, shortform, WordDivision { index: break_at }),
            expected
        );
    }

    #[test]
    fn lower_sequence_helpers_reconstruct_prefix_cells() {
        let word = chars("bb");
        let engine = engine_with(TestRule {
            pattern: &['b'],
            cell: '⠆',
            priority: 65,
            protect_span: false,
        });

        assert_eq!(all_lower_sequence_cells(&word, &engine), Some(cells("⠆⠆")));
    }

    #[test]
    fn lower_sequence_uses_matching_prefix_from_nonzero_position() {
        let word = chars("ab");
        let engine = engine_with(TestRule {
            pattern: &['a'],
            cell: '⠆',
            priority: 65,
            protect_span: false,
        });

        assert_eq!(
            all_lower_sequence_prefix_cells(&word, 1, &engine),
            Some(cells("⠆"))
        );
    }

    #[test]
    fn lower_sequence_follows_match_from_middle_position() {
        let word = chars("bc");
        let engine = engine_with(TestRule {
            pattern: &['b'],
            cell: '⠆',
            priority: 65,
            protect_span: false,
        });

        assert_eq!(
            all_lower_sequence_prefix_cells(&word, 1, &engine),
            Some(cells("⠆"))
        );
    }

    #[test]
    fn lower_sequence_prefix_skips_unreachable_positions() {
        let word = chars("ab");
        let engine = engine_with(TestRule {
            pattern: &['b'],
            cell: '⠆',
            priority: 65,
            protect_span: false,
        });

        assert_eq!(all_lower_sequence_prefix_cells(&word, 2, &engine), None);
    }

    #[test]
    fn lower_sequence_prefix_rejects_match_that_crosses_requested_end() {
        let word = chars("ab");
        let engine = engine_with(TestRule {
            pattern: &['a', 'b'],
            cell: '⠆',
            priority: 65,
            protect_span: false,
        });

        assert_eq!(all_lower_sequence_prefix_cells(&word, 1, &engine), None);
    }

    #[test]
    fn lower_sequence_prefix_accepts_match_that_reaches_requested_end() {
        let word = chars("ab");
        let engine = engine_with(TestRule {
            pattern: &['a', 'b'],
            cell: '⠆',
            priority: 65,
            protect_span: false,
        });

        assert_eq!(
            all_lower_sequence_prefix_cells(&word, 2, &engine),
            Some(cells("⠆"))
        );
    }

    #[test]
    fn candidate_moves_falls_back_to_plain_english_cell() {
        let word = chars("z");
        let moves = candidate_moves(
            &word,
            0,
            &ContractionEngine::default(),
            false,
            false,
            &vec![false; word.len()],
            &[],
            None,
            false,
            false,
            false,
        );

        assert!(moves.iter().any(|(cells, consumed, priority)| {
            *cells == vec![decode_unicode('⠵')] && *consumed == 1 && *priority == u16::MAX
        }));
    }

    #[test]
    fn longer_match_helpers_handle_herein_exception_directly() {
        let word = chars("hereinbelow");

        assert_eq!(longer_match(&word, 0), Some((6, cells("⠐⠓⠔"))));
        assert_eq!(shortform_part_cells(&word, 0), Some((6, cells("⠐⠓⠔"))));
        assert_eq!(
            longer_match_for_division(&word, 0, WordDivision { index: 6 }),
            Some((6, cells("⠐⠓⠔")))
        );
    }

    #[test]
    fn longer_match_for_division_accepts_division_specific_shortform() {
        let word = chars("penfriend");

        assert_eq!(
            longer_match_for_division(&word, 3, WordDivision { index: 3 }),
            Some((6, cells("⠋⠗")))
        );
    }

    #[test]
    fn division_longer_match_accepts_regular_longer_use_path() {
        let word = chars("aboutface");

        assert_eq!(
            longer_match_for_division(&word, 0, WordDivision { index: 5 }),
            Some((5, cells("⠁⠃")))
        );
    }

    #[test]
    fn division_shortform_or_branch_accepts_exception_path() {
        let word = chars("unnecessary");

        assert_eq!(
            longer_match_for_division(&word, 2, WordDivision { index: 2 }),
            Some((9, cells("⠝⠑⠉")))
        );
    }

    #[test]
    fn division_longer_match_accepts_division_specific_or_path() {
        let word = chars("penfriend");

        assert_eq!(
            longer_match_for_division(&word, 3, WordDivision { index: 3 }),
            Some((6, cells("⠋⠗")))
        );
    }

    #[rstest::rstest]
    #[case::encea("encea")]
    #[case::enced("enced")]
    #[case::encer("encer")]
    fn detects_ence_letter_sequence_contexts(#[case] word: &str) {
        assert!(is_ence_context(&chars(word), 0));
    }

    #[rstest::rstest]
    #[case::hereabout("hereabout", 4, true)]
    #[case::thereabouts("thereabouts", 5, true)]
    #[case::aboutface("aboutface", 0, true)]
    #[case::not_about_compound("turnaboutx", 4, false)]
    fn about_longer_use_paths(#[case] word: &str, #[case] pos: usize, #[case] expected: bool) {
        assert_eq!(longer_use_allowed(&chars(word), pos, "about"), expected);
    }

    #[test]
    fn about_compound_helper_accepts_plural_tail() {
        assert!(is_about_compound(&chars("thereabouts"), 5));
        assert!(is_about_compound(&chars("aboutturned"), 0));
        assert!(longer_use_allowed(&chars("thereabouts"), 5, "about"));
    }

    #[test]
    fn about_shortform_accepts_whereabout_suffix() {
        assert!(longer_use_allowed(&chars("whereabout"), 5, "about"));
    }

    #[test]
    fn about_shortform_runtime_prefix_suffix_path() {
        let word = chars(std::hint::black_box("hereabout"));

        assert!(longer_use_allowed(&word, 4, "about"));
    }

    #[test]
    fn runtime_shortform_helpers_cover_longer_loop_paths() {
        let aboutface = chars(std::hint::black_box("aboutface"));
        assert_eq!(longer_match(&aboutface, 0), Some((5, cells("⠁⠃"))));
        assert_eq!(shortform_part_cells(&aboutface, 0), Some((5, cells("⠁⠃"))));

        let thereabout = chars(std::hint::black_box("thereabout"));
        assert!(longer_use_allowed(&thereabout, 5, "about"));

        let unnecessary = chars(std::hint::black_box("unnecessary"));
        assert_eq!(
            longer_match_for_division(&unnecessary, 2, WordDivision { index: 2 }),
            Some((9, cells("⠝⠑⠉")))
        );
    }

    #[rstest::rstest]
    #[case::about_face("aboutfacing", 0, true)]
    #[case::some_such("somesuch", 4, true)]
    #[case::suchlike_listed("suchlike", 0, true)]
    #[case::children_before_vowel("childreny", 0, false)]
    #[case::paid_prefix_rejected("paid", 0, false)]
    #[case::paid_embedded_allowed("repaid", 2, true)]
    fn longer_use_additional_shortform_paths(
        #[case] word: &str,
        #[case] pos: usize,
        #[case] expected: bool,
    ) {
        let shortform = if word[pos..].starts_with("about") {
            "about"
        } else if word[pos..].starts_with("such") {
            "such"
        } else if word[pos..].starts_with("children") {
            "children"
        } else {
            "paid"
        };
        assert_eq!(longer_use_allowed(&chars(word), pos, shortform), expected);
    }

    #[rstest::rstest]
    #[case::good_before_consonant("goodness", true)]
    #[case::good_before_vowel_rejected("goodish", false)]
    #[case::goodafternoon_exception("goodafternoon", true)]
    #[case::quick_before_consonant("quickness", true)]
    #[case::quicker_exception("quicker", true)]
    #[case::quick_before_vowel_rejected("quickish", false)]
    fn prefix_shortform_vowel_gates(#[case] word: &str, #[case] expected: bool) {
        let chars: Vec<char> = word.chars().collect();
        let actual = if word.starts_with("good") {
            good_prefix_allowed(&chars)
        } else {
            quick_prefix_allowed(&chars)
        };
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case::asterisk('*', '⠡')]
    #[case::percent('%', '⠩')]
    #[case::question('?', '⠹')]
    #[case::backslash('\\', '⠳')]
    #[case::slash('/', '⠌')]
    #[case::two('2', '⠆')]
    #[case::three('3', '⠒')]
    #[case::nine('9', '⠔')]
    #[case::right_bracket(']', '⠻')]
    #[case::exclamation('!', '⠮')]
    #[case::double_quote('"', '⠐')]
    #[case::apostrophe('\'', '⠄')]
    fn maps_shortform_notation_symbols(#[case] notation: char, #[case] expected: char) {
        assert_eq!(notation_cell(notation), Some(decode_unicode(expected)));
    }

    #[test]
    fn rejects_unknown_shortform_notation_symbol() {
        assert_eq!(notation_cell('@'), None);
        assert_eq!(notation_cells("@").as_deref(), None);
    }

    #[test]
    fn candidate_moves_offers_early_letter_fallback_for_thorn() {
        // rule_12 early letters (Old/Middle English `þ` etc.) give the DP a
        // single-cell fallback move so it never stalls on a non-contractible
        // letter that still has a §4.1 print form.
        let word = chars("þ");
        let moves = candidate_moves(
            &word,
            0,
            &ContractionEngine::default(),
            false,
            false,
            &vec![false; word.len()],
            &[],
            None,
            false,
            false,
            false,
        );
        assert!(moves.iter().any(|(cells_, consumed, priority)| {
            *cells_ == cells("⠼⠮") && *consumed == 1 && *priority == u16::MAX
        }));
    }

    #[test]
    fn encode_with_constraints_returns_none_for_uncontractible_char() {
        // A character the §4/§10 fallback cannot encode (a CJK ideograph has no
        // early-letter, accent, or English cell) yields no candidate move at its
        // DP position, so `best` is None and the whole word fails to contract.
        let word = chars("a\u{4e00}");
        let result = encode_with_constraints(
            &word,
            &ContractionEngine::default(),
            false,
            false,
            None,
            false,
            true,
            false,
        );
        assert_eq!(result, None);
    }
}
