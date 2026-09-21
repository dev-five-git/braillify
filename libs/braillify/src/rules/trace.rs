//! Rule provenance — which registered rule produced which output cells.
//!
//! Tracing is opt-in. [`crate::encode`] never builds a [`Trace`]; only
//! [`crate::encode_with_trace`] passes a sink down to the rule engine, so the
//! untraced path keeps its shape.
//!
//! # What a trace can and cannot tell you
//!
//! Only the Korean character-level rule engine ([`super::engine::RuleEngine`])
//! is instrumented. Input owned by the UEB grade-2 engine or by the math token
//! engine produces **no events at all** — [`Trace::path`] reports which engine
//! ran so that an empty event list is never mistaken for "no rule applied".
//!
//! Within the Korean engine the unit of attribution is the *registered rule*,
//! not the standard's article. `RuleKorean` covers all ordinary syllable
//! composition, so `안녕` attributes to one rule per syllable rather than to
//! 제1항/제7항 individually. Narrowing that boundary means moving the dispatch
//! one level inward, not reinterpreting the events recorded here.
//!
//! # Rules that match but do not contribute
//!
//! A rule that returns [`RuleResult::Skip`](super::traits::RuleResult::Skip)
//! after matching produced nothing, so it is **not** recorded. "The rule ran"
//! and "the rule explains this output" are different claims, and only the
//! second one is worth reporting.

use std::ops::Range;
use std::sync::LazyLock;

use super::RuleMeta;

/// Identifier of a rule in one of the engines that make up the encoder.
///
/// The id space is partitioned by engine ([`RuleKind`]); within a partition the
/// value is the rule's position in that engine's dispatch order. Deriving ids
/// from the engines themselves (rather than from a hand-written table) means the
/// id space cannot drift away from the rules that can actually fire:
/// `rules/korean/` declares roughly twice as many [`RuleMeta`] literals as the
/// character engine registers, and the rest are unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuleId(pub u16);

/// Which engine a [`RuleId`] belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleKind {
    /// Character-level Korean rules ([`super::traits::BrailleRule`]).
    Korean,
    /// Token-level rewrites run before character encoding
    /// ([`super::token_rule::TokenRule`]).
    Token,
    /// Math expression rules ([`super::math::math_token_rule::MathTokenRule`]).
    Math,
    /// Jamo-level articles applied while composing one Korean syllable
    /// ([`JamoRule`]). These are code paths inside `korean_char`, not registered
    /// rule objects, so they carry their own metadata.
    Jamo,
    /// UEB grade-2 contraction rules
    /// ([`super::english_ueb::contraction::ContractionRule`]).
    EnglishUeb,
    /// Cells the emitter writes directly, outside any rule object.
    Emitter,
}

/// The article behind each step of composing one Korean syllable.
///
/// `RuleKorean` is a single registered rule covering all ordinary syllables, so
/// without this split every syllable reports the same composite entry. Sections
/// are taken from the PDF-derived fixtures in `test_cases/korean/`: `rule_6.json`
/// holds 아/야/어/여/오/요/우/유 and `rule_7.json` holds ㅐ/ㅒ/ㅔ/ㅖ/ㅘ/ㅙ, which
/// is what separates the two vowel articles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JamoRule {
    /// 제1항 — 초성 자음자.
    Choseong = 0,
    /// 제2항 — 된소리 첫소리 표.
    DoubleChoseong = 1,
    /// 제3항 — 받침 자음자.
    Jongseong = 2,
    /// 제6항 — 기본 모음자.
    Jungseong = 3,
    /// 제7항 — 그 밖의 모음자.
    JungseongExtended = 4,
    /// 제13항 — 글자 약자.
    Shortcut = 5,
}

impl JamoRule {
    const ALL: [Self; 6] = [
        Self::Choseong,
        Self::DoubleChoseong,
        Self::Jongseong,
        Self::Jungseong,
        Self::JungseongExtended,
        Self::Shortcut,
    ];

    /// 제6항's ten basic vowels, listed in `test_cases/korean/rule_6.json`;
    /// every other vowel belongs to 제7항.
    const BASIC_VOWELS: [char; 10] = ['ㅏ', 'ㅑ', 'ㅓ', 'ㅕ', 'ㅗ', 'ㅛ', 'ㅜ', 'ㅠ', 'ㅡ', 'ㅣ'];

    pub(crate) fn for_vowel(jung: char) -> Self {
        if Self::BASIC_VOWELS.contains(&jung) {
            Self::Jungseong
        } else {
            Self::JungseongExtended
        }
    }

    fn meta(self) -> &'static RuleMeta {
        &JAMO_METAS[self as usize]
    }
}

static JAMO_METAS: [RuleMeta; 6] = [
    RuleMeta {
        section: "1",
        subsection: None,
        name: "syllable_choseong",
        standard_ref: "2024 Korean Braille Standard, 제1항",
        description: "음절 첫소리 자음자",
    },
    RuleMeta {
        section: "2",
        subsection: None,
        name: "syllable_double_choseong",
        standard_ref: "2024 Korean Braille Standard, 제2항",
        description: "된소리 첫소리 표",
    },
    RuleMeta {
        section: "3",
        subsection: None,
        name: "syllable_jongseong",
        standard_ref: "2024 Korean Braille Standard, 제3항",
        description: "음절 받침 자음자",
    },
    RuleMeta {
        section: "6",
        subsection: None,
        name: "syllable_jungseong",
        standard_ref: "2024 Korean Braille Standard, 제6항",
        description: "기본 모음자",
    },
    RuleMeta {
        section: "7",
        subsection: None,
        name: "syllable_jungseong_extended",
        standard_ref: "2024 Korean Braille Standard, 제7항",
        description: "그 밖의 모음자",
    },
    RuleMeta {
        section: "13",
        subsection: None,
        name: "syllable_shortcut",
        standard_ref: "2024 Korean Braille Standard, 제13항",
        description: "글자 약자",
    },
];

impl RuleId {
    const TOKEN_BASE: u16 = 1000;
    const MATH_BASE: u16 = 2000;
    const JAMO_BASE: u16 = 3000;
    const UEB_BASE: u16 = 4000;
    const EMITTER_BASE: u16 = 5000;

    /// A rule with no registry entry.
    pub const UNATTRIBUTED: Self = Self(u16::MAX);

    pub(crate) fn korean(index: usize) -> Self {
        Self::within(0, Self::TOKEN_BASE, index)
    }

    pub(crate) fn token(index: usize) -> Self {
        Self::within(Self::TOKEN_BASE, Self::MATH_BASE, index)
    }

    pub(crate) fn math(index: usize) -> Self {
        Self::within(Self::MATH_BASE, Self::JAMO_BASE, index)
    }

    pub(crate) fn jamo(slot: JamoRule) -> Self {
        Self(Self::JAMO_BASE + slot as u16)
    }

    pub(crate) fn ueb(index: usize) -> Self {
        Self::within(Self::UEB_BASE, Self::EMITTER_BASE, index)
    }

    pub(crate) fn emitter(slot: EmitterRule) -> Self {
        Self(Self::EMITTER_BASE + slot as u16)
    }

    fn within(base: u16, limit: u16, index: usize) -> Self {
        u16::try_from(index)
            .ok()
            .and_then(|i| base.checked_add(i))
            .filter(|id| *id < limit)
            .map_or(Self::UNATTRIBUTED, Self)
    }

    /// Which engine this id belongs to, or `None` for [`Self::UNATTRIBUTED`].
    pub fn kind(self) -> Option<RuleKind> {
        match self.0 {
            _ if self == Self::UNATTRIBUTED => None,
            id if id < Self::TOKEN_BASE => Some(RuleKind::Korean),
            id if id < Self::MATH_BASE => Some(RuleKind::Token),
            id if id < Self::JAMO_BASE => Some(RuleKind::Math),
            id if id < Self::UEB_BASE => Some(RuleKind::Jamo),
            id if id < Self::EMITTER_BASE => Some(RuleKind::EnglishUeb),
            _ => Some(RuleKind::Emitter),
        }
    }

    /// Metadata for this rule, or `None` for [`Self::UNATTRIBUTED`].
    pub fn meta(self) -> Option<&'static RuleMeta> {
        rule_meta(self)
    }
}

/// How a recorded rule ended its dispatch.
///
/// [`RuleResult::Skip`](super::traits::RuleResult::Skip) has no variant here —
/// a skipping rule contributed nothing and is never recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleOutcome {
    /// The rule fully handled the character; no later rule ran for it.
    Consumed,
    /// The rule contributed and let later rules run for the same character.
    Continued,
}

/// One rule dispatch that contributed to the output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    /// The rule that ran. Resolve with [`RuleId::meta`].
    pub rule: RuleId,
    /// How the dispatch ended.
    pub outcome: RuleOutcome,
    /// Index into the token stream **as it exists after token rules ran**.
    /// Token rules rewrite the stream, so this does not index the input text.
    pub token_index: u32,
    /// Character range consumed within the current word, word-local.
    pub word_chars: Range<u32>,
    /// Cell range produced in the final output. Exact, and the anchor a
    /// consumer should key on. An empty range means the rule changed encoder
    /// state (mode, number context) without emitting cells.
    pub output: Range<u32>,
}

/// Which engine produced the output, and therefore how far the events reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TracePath {
    /// The Korean character-rule engine ran. Events describe its dispatches.
    #[default]
    KoreanRules,
    /// The UEB grade-2 engine owned the input. Events identify contraction rules
    /// selected by its cell-minimising path.
    EnglishUeb,
    /// The math token engine owned the input. Not instrumented; `events` is
    /// empty.
    MathExpression,
}

/// Rule dispatches recorded during one encode.
///
/// Events never account for the whole output. Cells emitted by token-level
/// rules — 약자 abbreviations, fractions, mode indicators — bypass the character
/// engine entirely, so `그래서` encodes to two cells with **zero** events.
/// [`Self::attributed_cells`] against [`Self::output_len`] states how much of
/// the output the events actually explain, so that gap is a number a caller can
/// check rather than a silence they have to interpret.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Trace {
    events: Vec<TraceEvent>,
    path: TracePath,
    output_len: u32,
}

/// Cells the emitter writes itself, with no rule object behind them.
///
/// These are structural: the emitter, not a rule, decides where an inter-word
/// blank goes. They are listed so those cells are attributed rather than
/// appearing as an unexplained gap in the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitterRule {
    /// The blank cell between two print words.
    WordSpace = 0,
    /// A pre-encoded run a token rule produced but whose rule is undeclared.
    UndeclaredTokenOutput = 1,
    /// 제29항 로마자표/연속표/종료표. The emitter opens, resumes and closes a
    /// Roman section from the token stream, so no character rule sees it.
    RomanSectionMarker = 2,
}

impl EmitterRule {
    const ALL: [Self; 3] = [
        Self::WordSpace,
        Self::UndeclaredTokenOutput,
        Self::RomanSectionMarker,
    ];

    fn meta(self) -> &'static RuleMeta {
        match self {
            Self::WordSpace => &WORD_SPACE_META,
            Self::UndeclaredTokenOutput => &UNDECLARED_TOKEN_OUTPUT_META,
            Self::RomanSectionMarker => &ROMAN_SECTION_MARKER_META,
        }
    }
}

static ROMAN_SECTION_MARKER_META: RuleMeta = RuleMeta {
    section: "29",
    subsection: None,
    name: "roman_section_marker",
    standard_ref: "2024 Korean Braille Standard, 제29항",
    description: "로마자표·로마자 종료표",
};

/// The id of a registered Korean character rule, found by its metadata name.
///
/// The emitter sometimes produces cells on behalf of a rule that also exists in
/// the engine, and this keeps both reporting the same id instead of minting a
/// second one for the same article.
pub(crate) fn korean_rule_id(name: &str) -> RuleId {
    REGISTRIES
        .korean
        .iter()
        .position(|meta| meta.name == name)
        .map_or(RuleId::UNATTRIBUTED, RuleId::korean)
}

static WORD_SPACE_META: RuleMeta = RuleMeta {
    section: "-",
    subsection: None,
    name: "word_space",
    standard_ref: "어절 사이 빈칸",
    description: "Inter-word blank cell written by the emitter",
};

static UNDECLARED_TOKEN_OUTPUT_META: RuleMeta = RuleMeta {
    section: "?",
    subsection: None,
    name: "undeclared_token_output",
    standard_ref: "",
    description: "Cells from a token rule that has not declared its article",
};

/// Which rule produced each token, maintained alongside the token stream.
///
/// Token rules rewrite the stream rather than emitting cells, so their output
/// only becomes cells later in [`super::emit`]. Recording the producing rule per
/// token slot is what lets those cells be attributed at emit time. The vector is
/// kept the same length as the token stream; [`Self::len`] is asserted against
/// it after every rewrite.
#[derive(Debug, Default)]
pub struct TokenOrigins {
    origins: Vec<Option<RuleId>>,
}

impl TokenOrigins {
    pub(crate) fn seeded(len: usize) -> Self {
        Self {
            origins: vec![None; len],
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.origins.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<RuleId> {
        self.origins.get(index).copied().flatten()
    }

    pub(crate) fn set(&mut self, index: usize, rule: RuleId) {
        if let Some(slot) = self.origins.get_mut(index) {
            *slot = Some(rule);
        }
    }

    pub(crate) fn splice(&mut self, range: core::ops::Range<usize>, rule: RuleId, count: usize) {
        let end = range.end.min(self.origins.len());
        let start = range.start.min(end);
        self.origins.splice(start..end, vec![Some(rule); count]);
    }

    #[cfg(test)]
    pub(crate) fn remove(&mut self, index: usize) {
        if index < self.origins.len() {
            self.origins.remove(index);
        }
    }
}

/// A [`Trace`] bound to the token currently being emitted.
///
/// The token index lives here rather than on
/// [`RuleContext`](super::context::RuleContext) so that the twenty-odd places
/// that build a context — nearly all of them rule unit tests — stay untouched.
pub struct TraceSink<'a> {
    pub(crate) trace: &'a mut Trace,
    pub(crate) token_index: u32,
}

impl<'a> TraceSink<'a> {
    pub(crate) fn new(trace: &'a mut Trace) -> Self {
        Self {
            trace,
            token_index: 0,
        }
    }

    pub(crate) fn at_token(&mut self, token_index: usize) -> TraceSink<'_> {
        TraceSink {
            trace: self.trace,
            token_index: token_index as u32,
        }
    }

    pub(crate) fn reborrow(&mut self) -> TraceSink<'_> {
        TraceSink {
            trace: self.trace,
            token_index: self.token_index,
        }
    }

    pub(crate) fn token_index(&self) -> u32 {
        self.token_index
    }

    pub(crate) fn record_span(
        &mut self,
        rule: RuleId,
        token_index: usize,
        output: core::ops::Range<usize>,
    ) {
        self.trace.push(TraceEvent {
            rule,
            outcome: RuleOutcome::Consumed,
            token_index: token_index as u32,
            word_chars: 0..0,
            output: output.start as u32..output.end as u32,
        });
    }
}

impl Trace {
    /// Every recorded dispatch, in the order it happened.
    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }

    /// Which engine produced the output. Read this before concluding anything
    /// from an empty [`Self::events`].
    pub fn path(&self) -> TracePath {
        self.path
    }

    /// Total cells in the encoded output.
    pub fn output_len(&self) -> u32 {
        self.output_len
    }

    /// How many output cells at least one event accounts for.
    pub fn attributed_cells(&self) -> u32 {
        let mut covered = vec![false; self.output_len as usize];
        for event in &self.events {
            for cell in event.output.clone() {
                if let Some(slot) = covered.get_mut(cell as usize) {
                    *slot = true;
                }
            }
        }
        covered.iter().filter(|seen| **seen).count() as u32
    }

    /// Output cells no event accounts for. A non-zero value means part of the
    /// output came from an uninstrumented path, not that it came from nowhere.
    pub fn unattributed_cells(&self) -> u32 {
        self.output_len - self.attributed_cells()
    }

    /// Rules that emitted at least one cell, in order of first contribution,
    /// without repeats.
    pub fn contributing_rules(&self) -> Vec<RuleId> {
        let mut seen = Vec::new();
        for event in &self.events {
            if !event.output.is_empty() && !seen.contains(&event.rule) {
                seen.push(event.rule);
            }
        }
        seen
    }

    /// The rules whose output covers `cell`, innermost dispatch last.
    pub fn rules_at_cell(&self, cell: u32) -> Vec<RuleId> {
        self.events
            .iter()
            .filter(|event| event.output.contains(&cell))
            .map(|event| event.rule)
            .collect()
    }

    pub(crate) fn set_path(&mut self, path: TracePath) {
        self.path = path;
    }

    pub(crate) fn set_output_len(&mut self, len: u32) {
        self.output_len = len;
    }

    pub(crate) fn push(&mut self, event: TraceEvent) {
        self.events.push(event);
    }

    /// Number of events recorded so far, for [`Self::rollback_to`].
    pub(crate) fn mark(&self) -> usize {
        self.events.len()
    }

    /// Drop everything recorded after `mark`.
    ///
    /// The math pipeline encodes speculatively and falls back to the Korean
    /// encoder on error, having already emitted cells for the tokens it did
    /// consume. Those cells never ship, so crediting their rules would name
    /// rules that did not produce the output.
    pub(crate) fn rollback_to(&mut self, mark: usize) {
        self.events.truncate(mark);
    }

    /// Shift every output range by `delta` cells.
    ///
    /// 제37항 wraps an isolated Roman section by inserting the Roman indicator
    /// at index 0 after encoding, which moves every cell already recorded.
    pub(crate) fn shift_output(&mut self, delta: u32) {
        for event in &mut self.events {
            event.output = (event.output.start + delta)..(event.output.end + delta);
        }
    }
}

/// Metadata of every rule the Korean engine registers, indexed by [`RuleId`].
///
/// Built from a throwaway [`crate::encoder::Encoder`] so the registry *is* the
/// registration list in `Encoder::new`, by construction.
struct Registries {
    korean: Vec<&'static RuleMeta>,
    token: Vec<&'static RuleMeta>,
}

static REGISTRIES: LazyLock<Registries> = LazyLock::new(|| {
    let mut probe = crate::encoder::Encoder::new(false);
    Registries {
        korean: probe.char_rule_registry(),
        token: probe.token_rule_registry(),
    }
});

static MATH_REGISTRY: LazyLock<Vec<&'static RuleMeta>> =
    LazyLock::new(crate::rules::math::encoder::math_rule_registry);

static UEB_REGISTRY: LazyLock<Vec<&'static RuleMeta>> =
    LazyLock::new(crate::rules::english_ueb::ueb_rule_registry);

/// Metadata for `id`, or `None` when the id has no registry entry.
pub fn rule_meta(id: RuleId) -> Option<&'static RuleMeta> {
    let offset = |base: u16| (id.0 - base) as usize;
    match id.kind()? {
        RuleKind::Korean => REGISTRIES.korean.get(id.0 as usize).copied(),
        RuleKind::Token => REGISTRIES.token.get(offset(RuleId::TOKEN_BASE)).copied(),
        RuleKind::Math => MATH_REGISTRY.get(offset(RuleId::MATH_BASE)).copied(),
        RuleKind::Jamo => JamoRule::ALL
            .get(offset(RuleId::JAMO_BASE))
            .map(|slot| slot.meta()),
        RuleKind::EnglishUeb => UEB_REGISTRY.get(offset(RuleId::UEB_BASE)).copied(),
        RuleKind::Emitter => EmitterRule::ALL
            .get(offset(RuleId::EMITTER_BASE))
            .map(|slot| slot.meta()),
    }
}

/// Every rule that can fire, grouped by the engine it belongs to.
pub fn registered_rules(kind: RuleKind) -> &'static [&'static RuleMeta] {
    match kind {
        RuleKind::Korean => &REGISTRIES.korean,
        RuleKind::Token => &REGISTRIES.token,
        RuleKind::Math => &MATH_REGISTRY,
        RuleKind::Jamo => &JAMO_METAS_REFS,
        RuleKind::EnglishUeb => &UEB_REGISTRY,
        RuleKind::Emitter => &[],
    }
}

static JAMO_METAS_REFS: [&RuleMeta; 6] = [
    &JAMO_METAS[0],
    &JAMO_METAS[1],
    &JAMO_METAS[2],
    &JAMO_METAS[3],
    &JAMO_METAS[4],
    &JAMO_METAS[5],
];

#[cfg(test)]
mod tests {
    use super::*;

    fn event(rule: u16, output: Range<u32>) -> TraceEvent {
        TraceEvent {
            rule: RuleId(rule),
            outcome: RuleOutcome::Consumed,
            token_index: 0,
            word_chars: 0..1,
            output,
        }
    }

    #[rstest::rstest]
    #[case::korean(RuleKind::Korean)]
    #[case::token(RuleKind::Token)]
    #[case::math(RuleKind::Math)]
    fn every_engine_registers_rules_resolvable_by_id(#[case] kind: RuleKind) {
        let rules = registered_rules(kind);
        assert!(!rules.is_empty(), "{kind:?} registers rules");

        let first = match kind {
            RuleKind::Korean => RuleId::korean(0),
            RuleKind::Token => RuleId::token(0),
            RuleKind::Math => RuleId::math(0),
            RuleKind::Jamo | RuleKind::EnglishUeb | RuleKind::Emitter => {
                unreachable!("only registry-backed engines are cased here")
            }
        };
        assert_eq!(first.kind(), Some(kind));
        assert_eq!(rule_meta(first), Some(rules[0]));
    }

    /// The UEB partition reserves its first slots for move sources that are not
    /// rule objects, so a contraction rule's id sits at a fixed offset.
    #[test]
    fn ueb_partition_reserves_slots_before_the_contraction_rules() {
        let rules = registered_rules(RuleKind::EnglishUeb);

        assert!(rules.len() > crate::rules::english_ueb::UEB_RESERVED_SLOTS);
        assert_eq!(RuleId::ueb(0).meta().map(|m| m.name), Some("ueb_shortform"));
        assert_eq!(
            RuleId::ueb(crate::rules::english_ueb::UEB_RESERVED_SLOTS)
                .meta()
                .map(|m| m.section),
            Some("10.3"),
            "the first contraction rule follows the reserved slots"
        );
    }

    #[test]
    fn unattributed_has_no_metadata_and_no_kind() {
        assert_eq!(RuleId::UNATTRIBUTED.meta(), None);
        assert_eq!(RuleId::UNATTRIBUTED.kind(), None);
    }

    #[rstest::rstest]
    #[case::word_space(EmitterRule::WordSpace, "word_space")]
    #[case::undeclared(EmitterRule::UndeclaredTokenOutput, "undeclared_token_output")]
    #[case::roman_section(EmitterRule::RomanSectionMarker, "roman_section_marker")]
    fn emitter_slots_resolve_to_their_metadata(#[case] slot: EmitterRule, #[case] name: &str) {
        let id = RuleId::emitter(slot);
        assert_eq!(id.kind(), Some(RuleKind::Emitter));
        assert_eq!(id.meta().map(|m| m.name), Some(name));
    }

    /// The jamo articles are listed like any other engine's rules, while the
    /// emitter's structural cells are reachable by id but are not rules anyone
    /// can enumerate as candidates.
    #[test]
    fn the_remaining_engines_report_their_own_rule_lists() {
        assert_eq!(registered_rules(RuleKind::Jamo).len(), JamoRule::ALL.len());
        assert_eq!(
            registered_rules(RuleKind::Jamo)[0].section,
            JamoRule::Choseong.meta().section
        );
        assert!(!registered_rules(RuleKind::EnglishUeb).is_empty());
        assert!(registered_rules(RuleKind::Emitter).is_empty());
    }

    /// An index past its engine's partition would collide with the next engine,
    /// so it resolves to nothing instead.
    #[rstest::rstest]
    #[case::korean(RuleId::korean(RuleId::TOKEN_BASE as usize))]
    #[case::token(RuleId::token(RuleId::MATH_BASE as usize))]
    #[case::math(RuleId::math(RuleId::JAMO_BASE as usize))]
    #[case::ueb(RuleId::ueb(RuleId::EMITTER_BASE as usize))]
    fn an_index_past_its_partition_resolves_to_nothing(#[case] id: RuleId) {
        assert_eq!(id, RuleId::UNATTRIBUTED);
    }

    /// The emitter borrows a character rule's id by name so both report the
    /// same article; a name the engine never registered borrows nothing.
    #[test]
    fn borrowing_a_rule_id_by_name_needs_a_registered_name() {
        let registered = registered_rules(RuleKind::Korean)[0].name;

        assert_eq!(korean_rule_id(registered), RuleId::korean(0));
        assert_eq!(korean_rule_id("no_such_rule"), RuleId::UNATTRIBUTED);
    }

    /// 제6항 lists the ten basic vowels; every other vowel is 제7항.
    #[rstest::rstest]
    #[case::basic('ㅏ', JamoRule::Jungseong)]
    #[case::extended('ㅘ', JamoRule::JungseongExtended)]
    fn a_vowel_belongs_to_the_article_that_lists_it(
        #[case] vowel: char,
        #[case] expected: JamoRule,
    ) {
        assert_eq!(JamoRule::for_vowel(vowel), expected);
    }

    #[test]
    fn korean_registry_holds_no_duplicate_rule_names() {
        let mut names: Vec<_> = registered_rules(RuleKind::Korean)
            .iter()
            .map(|m| m.name)
            .collect();
        let before = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(before, names.len(), "each registered rule name is unique");
    }

    #[test]
    fn rollback_to_drops_events_recorded_after_the_mark() {
        let mut trace = Trace::default();
        trace.push(event(1, 0..1));
        let mark = trace.mark();
        trace.push(event(2, 1..2));

        trace.rollback_to(mark);

        assert_eq!(trace.events().len(), 1);
        assert_eq!(trace.events()[0].rule, RuleId(1));
    }

    #[test]
    fn token_origins_survive_a_splice_that_changes_length() {
        let mut origins = TokenOrigins::seeded(3);
        origins.set(0, RuleId::token(4));

        origins.splice(1..2, RuleId::token(7), 3);

        assert_eq!(origins.len(), 5);
        assert_eq!(origins.get(0), Some(RuleId::token(4)));
        assert_eq!(origins.get(1), Some(RuleId::token(7)));
        assert_eq!(origins.get(3), Some(RuleId::token(7)));
        assert_eq!(origins.get(4), None);
    }

    #[test]
    fn contributing_rules_skips_cellless_events_and_repeats() {
        let mut trace = Trace::default();
        trace.push(event(1, 0..2));
        trace.push(event(2, 2..2)); // state-only, emitted nothing
        trace.push(event(1, 2..4)); // repeat of an already-listed rule

        assert_eq!(trace.contributing_rules(), vec![RuleId(1)]);
    }

    #[rstest::rstest]
    #[case::before_first(0, vec![RuleId(1)])]
    #[case::inside_first(1, vec![RuleId(1)])]
    #[case::inside_second(2, vec![RuleId(2)])]
    #[case::past_the_end(9, vec![])]
    fn rules_at_cell_selects_covering_events(#[case] cell: u32, #[case] expected: Vec<RuleId>) {
        let mut trace = Trace::default();
        trace.push(event(1, 0..2));
        trace.push(event(2, 2..3));

        assert_eq!(trace.rules_at_cell(cell), expected);
    }

    #[test]
    fn shift_output_moves_every_recorded_range() {
        let mut trace = Trace::default();
        trace.push(event(1, 0..2));
        trace.push(event(2, 2..5));

        trace.shift_output(1);

        assert_eq!(trace.events()[0].output, 1..3);
        assert_eq!(trace.events()[1].output, 3..6);
    }

    #[test]
    fn default_path_is_the_korean_engine() {
        assert_eq!(Trace::default().path(), TracePath::KoreanRules);
    }

    #[rstest::rstest]
    #[case::ueb(TracePath::EnglishUeb)]
    #[case::math(TracePath::MathExpression)]
    #[case::korean(TracePath::KoreanRules)]
    fn set_path_records_the_engine_that_ran(#[case] path: TracePath) {
        let mut trace = Trace::default();
        trace.set_path(path);
        assert_eq!(trace.path(), path);
    }
}
