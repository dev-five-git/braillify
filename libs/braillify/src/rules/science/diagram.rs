//! 과학 제9·10·15·16·25항 — 묵자에서 2차원으로 그린 구조식·전자 점식·가계도.
//!
//! 입력은 묵자 배치를 줄마다 옮긴 여러 줄 글이다. 구조식과 전자 점식은 기호 표기
//! 형식(제10항 3, 제16항 3·4)으로 풀어 한 줄에 적고, 가계도는 세대마다 한 줄씩
//! 적는다(제25항).

use super::formula::{self, Item};
use super::genotype::gene_cells;
use crate::unicode::decode_unicode;

const LINE_BREAK: u8 = 255;

/// 두 원소를 잇는 것 — 결합선(제10항 2)이나 전자(제16항 1).
#[derive(Clone, Copy)]
enum Link {
    Bond(u8),
    Electrons(usize),
}

impl Link {
    fn item(self) -> Item {
        match self {
            Link::Bond(order) => Item::Bond(order),
            Link::Electrons(count) => Item::Electrons(count),
        }
    }
}

fn electrons(mark: char) -> Option<usize> {
    match mark {
        ':' | '‥' => Some(2),
        '⋮' => Some(3),
        '∷' => Some(4),
        _ => None,
    }
}

fn horizontal_link(gap: &[char]) -> Option<Link> {
    let marks: Vec<char> = gap.iter().copied().filter(|c| *c != ' ').collect();
    match marks.as_slice() {
        [] => None,
        ['-' | '–'] => Some(Link::Bond(1)),
        ['='] => Some(Link::Bond(2)),
        ['≡'] => Some(Link::Bond(3)),
        dots => dots
            .iter()
            .map(|mark| electrons(*mark))
            .sum::<Option<usize>>()
            .map(Link::Electrons),
    }
}

fn vertical_link(mark: char) -> Option<Link> {
    match mark {
        '|' | '│' => Some(Link::Bond(1)),
        '‖' => Some(Link::Bond(2)),
        '⦀' => Some(Link::Bond(3)),
        _ => electrons(mark).map(Link::Electrons),
    }
}

/// 원소 앞뒤에 홀로 붙은 전자. 비어 있으면 `Some(None)`, 결합선이면 `None`.
fn dangling(gap: &[char]) -> Option<Option<Link>> {
    match horizontal_link(gap) {
        None if gap.iter().all(|c| *c == ' ') => Some(None),
        Some(link @ Link::Electrons(_)) => Some(Some(link)),
        _ => None,
    }
}

fn is_atom_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ('₀'..='₉').contains(&ch)
}

/// 원소 기호와 그 아래 첨자로 된 묶음(`C`, `OH`, `CH₃`) 하나.
struct Atom {
    row: usize,
    columns: std::ops::Range<usize>,
    items: Vec<Item>,
}

/// 한 줄에 가로로 이어진 원소들.
struct Chain {
    lead: Option<Link>,
    atoms: Vec<usize>,
    links: Vec<Link>,
    trail: Option<Link>,
}

fn atom_row(row: usize, chars: &[char], atoms: &mut Vec<Atom>) -> Option<Chain> {
    let mut gaps: Vec<Vec<char>> = vec![Vec::new()];
    let mut members = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        if !is_atom_char(chars[at]) {
            gaps.last_mut()?.push(chars[at]);
            at += 1;
            continue;
        }
        let start = at;
        while chars.get(at).copied().is_some_and(is_atom_char) {
            at += 1;
        }
        let items = formula::parse(&chars[start..at].iter().collect::<String>())?;
        if !items
            .iter()
            .all(|item| matches!(item, Item::Capital { element: true, .. } | Item::Sub(_)))
        {
            return None;
        }
        members.push(atoms.len());
        atoms.push(Atom {
            row,
            columns: start..at,
            items,
        });
        gaps.push(Vec::new());
    }
    let links = gaps[1..gaps.len() - 1]
        .iter()
        .map(|gap| horizontal_link(gap))
        .collect::<Option<Vec<_>>>()?;
    Some(Chain {
        lead: dangling(&gaps[0])?,
        atoms: members,
        links,
        trail: dangling(&gaps[gaps.len() - 1])?,
    })
}

const UP: usize = 0;
const DOWN: usize = 1;

/// 원소 하나에 달린 것 — 위·아래의 결합과 그 끝 원소, 같은 줄의 왼쪽·오른쪽.
#[derive(Default)]
struct Neighbours {
    vertical: [Option<(Link, Option<usize>)>; 2],
    left: Option<(Link, usize)>,
    right: Option<(Link, usize)>,
}

struct Walk<'a> {
    atoms: &'a [Atom],
    neighbours: &'a [Neighbours],
    seen: Vec<bool>,
    vertical_links: usize,
    items: Vec<Item>,
}

impl Walk<'_> {
    fn visit(&mut self, atom: usize) -> Option<()> {
        if std::mem::replace(&mut self.seen[atom], true) {
            return None;
        }
        self.items.extend(self.atoms[atom].items.iter().cloned());
        Some(())
    }

    fn link(&mut self, link: Link, atom: usize) -> Option<()> {
        self.items.push(link.item());
        self.visit(atom)
    }

    /// 제10항 3 가·나, 제16항 3 — 중심 원소의 위(⠬)와 아래(⠩)를 적는다. 적었으면
    /// `true` — 그 뒤 오른쪽 원소나 전자 앞에 ⠤ 을 적는다(제10항 3 라).
    fn branches(&mut self, atom: usize) -> Option<bool> {
        let mut branched = false;
        for (side, mark) in [(UP, '⠬'), (DOWN, '⠩')] {
            let Some((link, target)) = self.neighbours[atom].vertical[side] else {
                continue;
            };
            branched = true;
            self.vertical_links += 1;
            self.items.push(Item::Branch(mark));
            self.items.push(link.item());
            if let Some(target) = target {
                self.visit(target)?;
                self.side_chain(target, side)?;
            }
        }
        Some(branched)
    }

    /// 제10항 3 다, 제16항 4 — 측쇄에 또 달린 측쇄는 왼쪽 ⠣, 오른쪽 ⠜ 을 먼저 적는다.
    fn side_chain(&mut self, atom: usize, side: usize) -> Option<()> {
        if self.neighbours[atom].vertical[side].is_some() {
            return None;
        }
        for (mark, rightward) in [('⠣', false), ('⠜', true)] {
            let step = |n: &Neighbours| if rightward { n.right } else { n.left };
            let mut next = step(&self.neighbours[atom]);
            if next.is_some() {
                self.items.push(Item::Branch(mark));
            }
            while let Some((link, target)) = next {
                self.link(link, target)?;
                next = step(&self.neighbours[target]);
            }
        }
        Some(())
    }

    fn main_chain(&mut self, chain: &Chain) -> Option<()> {
        self.items.extend(chain.lead.map(Link::item));
        let mut branched = false;
        for (at, &atom) in chain.atoms.iter().enumerate() {
            if at > 0 {
                self.right_of(branched, chain.links[at - 1]);
            }
            self.visit(atom)?;
            branched = self.branches(atom)?;
        }
        if let Some(trail) = chain.trail {
            self.right_of(branched, trail);
        }
        Some(())
    }

    fn right_of(&mut self, branched: bool, link: Link) {
        if branched {
            self.items.push(Item::Branch('⠤'));
        }
        self.items.push(link.item());
    }
}

/// 과학 제9항 1·제10항 3·제15항 1·제16항 — 2차원 구조식과 전자 점식을 기호 표기
/// 형식으로 적는다. 왼쪽에서 오른쪽으로, 위쪽에서 아래쪽으로 풀어 적는다.
fn structural(text: &str) -> Option<Vec<u8>> {
    let rows: Vec<Vec<char>> = text.lines().map(|line| line.chars().collect()).collect();
    let mut atoms = Vec::new();
    let mut chains = Vec::new();
    let mut verticals = Vec::new();
    for (row, chars) in rows.iter().enumerate() {
        if chars.iter().copied().any(is_atom_char) {
            chains.push(atom_row(row, chars, &mut atoms)?);
            continue;
        }
        let marks: Vec<(usize, char)> = chars
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, ch)| *ch != ' ')
            .collect();
        if marks.is_empty() {
            return None;
        }
        for (column, mark) in marks {
            verticals.push((row, column, vertical_link(mark)?));
        }
    }
    if verticals.is_empty() {
        return None;
    }

    let mut neighbours: Vec<Neighbours> = atoms.iter().map(|_| Neighbours::default()).collect();
    for chain in &chains {
        for (at, link) in chain.links.iter().enumerate() {
            let (left, right) = (chain.atoms[at], chain.atoms[at + 1]);
            neighbours[left].right = Some((*link, right));
            neighbours[right].left = Some((*link, left));
        }
    }
    let atom_at = |row: Option<usize>, column: usize| {
        let row = row?;
        atoms
            .iter()
            .position(|atom| atom.row == row && atom.columns.contains(&column))
    };
    for &(row, column, link) in &verticals {
        let above = atom_at(row.checked_sub(1), column);
        let below = atom_at(Some(row + 1), column);
        match (above, below, link) {
            (None, None, _) | (None, _, Link::Bond(_)) | (_, None, Link::Bond(_)) => return None,
            _ => {}
        }
        for (end, side, other) in [(above, DOWN, below), (below, UP, above)] {
            if let Some(end) = end {
                let slot = &mut neighbours[end].vertical[side];
                if slot.replace((link, other)).is_some() {
                    return None;
                }
            }
        }
    }

    let main = chains.iter().min_by_key(|chain| {
        (
            atoms[chain.atoms[0]].columns.start,
            atoms[chain.atoms[0]].row,
        )
    })?;
    if chains
        .iter()
        .any(|chain| !std::ptr::eq(chain, main) && (chain.lead.is_some() || chain.trail.is_some()))
    {
        return None;
    }
    let mut walk = Walk {
        atoms: &atoms,
        neighbours: &neighbours,
        seen: vec![false; atoms.len()],
        vertical_links: 0,
        items: Vec::new(),
    };
    walk.main_chain(main)?;
    let complete = walk.seen.iter().all(|seen| *seen) && walk.vertical_links == verticals.len();
    complete
        .then(|| formula::encode(&walk.items).ok())
        .flatten()
}

fn is_connector_row(line: &str) -> bool {
    !line.trim().is_empty()
        && line.chars().all(|ch| {
            matches!(
                ch,
                ' ' | '|' | '│' | '┌' | '┐' | '┬' | '┴' | '─' | '└' | '┘' | '├' | '┤'
            )
        })
}

/// 세대를 나타내는 문자 — 어버이 `P`, 자손 `F₁`·`F₂`. 홀로 선 `P` 는 통일영어점자
/// §5.7.1 에 따라 1급 점자 기호표를 앞세운다.
fn generation_label(word: &str) -> Option<Vec<u8>> {
    let items = formula::parse(word)?;
    let cells = formula::encode(&items).ok()?;
    match items.as_slice() {
        [Item::Capital { symbol, .. }] if symbol == "P" => {
            Some([vec![decode_unicode('⠰')], cells].concat())
        }
        [Item::Capital { symbol, .. }, Item::Sub(number)]
            if symbol == "F" && number.chars().all(|c| c.is_ascii_digit()) =>
        {
            Some(cells)
        }
        _ => None,
    }
}

fn genes(word: &str) -> Option<Vec<u8>> {
    let chars: Vec<char> = word.chars().collect();
    let pairs = !chars.is_empty()
        && chars.len().is_multiple_of(2)
        && chars.iter().all(char::is_ascii_alphabetic)
        && chars
            .chunks(2)
            .all(|pair| pair[0].eq_ignore_ascii_case(&pair[1]));
    pairs.then(|| gene_cells(&chars)).flatten()
}

/// 제25항 2·4 — 곱하기 기호 앞뒤는 한 칸씩 띄고, 한 세대의 여럿은 묶음 괄호로 묶는다.
fn generation(words: &[&str]) -> Option<Vec<u8>> {
    match words {
        [] => None,
        [one] => genes(one),
        [mother, "×", father] => Some(
            [
                genes(mother)?,
                vec![0, decode_unicode('⠡'), 0],
                genes(father)?,
            ]
            .concat(),
        ),
        siblings => {
            let mut out = vec![decode_unicode('⠷')];
            for (at, word) in siblings.iter().enumerate() {
                if at > 0 {
                    out.push(0);
                }
                out.extend(genes(word)?);
            }
            out.push(decode_unicode('⠾'));
            Some(out)
        }
    }
}

/// 과학 제25항 — 가계도. 세대마다 한 줄에 세대 문자와 유전자를 두 칸 띄어 적고,
/// 다음 세대가 있으면 끝에 오른쪽 화살표를 적는다.
fn pedigree(text: &str) -> Option<Vec<u8>> {
    let mut generations = Vec::new();
    for line in text.lines().filter(|line| !is_connector_row(line)) {
        let words: Vec<&str> = line.split_whitespace().collect();
        let (label, content) = words.split_first()?;
        generations.push((generation_label(label)?, generation(content)?));
    }
    if generations.len() < 2 {
        return None;
    }
    let last = generations.len() - 1;
    let mut out = Vec::new();
    for (at, (label, content)) in generations.into_iter().enumerate() {
        if at > 0 {
            out.push(LINE_BREAK);
        }
        out.extend(label);
        out.extend([0, 0]);
        out.extend(content);
        if at < last {
            out.extend([0, decode_unicode('⠒'), decode_unicode('⠕')]);
        }
    }
    Some(out)
}

/// 여러 줄로 그린 과학 도식을 점자로 적는다. 도식이 아니면 `None`.
pub(crate) fn encode(text: &str) -> Option<Vec<u8>> {
    if !text.contains('\n') {
        return None;
    }
    structural(text).or_else(|| pedigree(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn braille(cells: &[u8]) -> String {
        cells
            .iter()
            .map(|cell| crate::unicode::encode_unicode(*cell))
            .collect()
    }

    #[rstest::rstest]
    #[case::ammonia("  H\n  |\nH-N\n  |\n  H", "⠠⠠⠠⠓⠰⠂⠝⠬⠰⠂⠓⠩⠰⠂⠓⠠⠄")]
    #[case::ethane_carbon("  H\n  |\nH-C-H\n  |\n  H", "⠠⠠⠠⠓⠰⠂⠉⠬⠰⠂⠓⠩⠰⠂⠓⠤⠰⠂⠓⠠⠄")]
    #[case::side_chain_left("  O\n  ‖\nH-C\n  |\nH-O", "⠠⠠⠠⠓⠰⠂⠉⠬⠰⠆⠕⠩⠰⠂⠕⠣⠰⠂⠓⠠⠄")]
    #[case::grouped_atoms("CH₃ - CH₂\n       |\n       OH", "⠠⠠⠠⠉⠓⠰⠼⠉⠰⠂⠉⠓⠰⠼⠃⠩⠰⠂⠕⠓⠠⠄")]
    #[case::lone_pairs(" ‥\n:F:\n ‥", "⠔⠔⠠⠋⠬⠔⠔⠩⠔⠔⠤⠔⠔")]
    #[case::triple_bonds("C≡N\n⦀\nN", "⠠⠠⠠⠉⠩⠰⠒⠝⠤⠰⠒⠝⠠⠄")]
    #[case::three_and_four_electrons("H⋮N\n  ∷\n  H", "⠠⠠⠠⠓⠔⠔⠔⠝⠩⠔⠔⠔⠔⠓⠠⠄")]
    fn writes_diagrams_in_symbol_form(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(
            encode(text).map(|cells| braille(&cells)).as_deref(),
            Some(expected)
        );
    }

    #[rstest::rstest]
    #[case::one_line("H-O-H")]
    #[case::no_vertical_link("H-O\nO-H")]
    #[case::blank_row("H\n\n|\nH")]
    #[case::unknown_mark("H\n*\nH")]
    #[case::dangling_bond("H\n|")]
    #[case::bond_into_nothing("H-N\n  |")]
    #[case::double_vertical("H H\n| |\n H")]
    #[case::not_an_element("A\n|\nH")]
    #[case::english_words("hello\n|\nworld")]
    #[case::missing_link("H  N\n|\nH")]
    #[case::bond_before_atom("-H\n |\n H")]
    #[case::side_row_electrons("H\n|\nH:")]
    #[case::branch_beyond_branch("H\n|\nO\n|\nH")]
    #[case::ring("H-O\n| |\nH-H")]
    #[case::two_links_one_slot("OH\n||\nOH")]
    #[case::unreached_atom("H\n|\nH\nO")]
    #[case::mixed_marks("H-:O\n|\nH")]
    fn leaves_other_text_alone(#[case] text: &str) {
        assert_eq!(encode(text), None);
    }

    #[rstest::rstest]
    #[case::cross_then_offspring("P  BB × bb\n    |\nF₁  Bb", "⠰⠠⠏⠀⠀⠠⠠⠃⠃⠀⠡⠀⠃⠃⠀⠒⠕\n⠠⠋⠰⠼⠁⠀⠀⠠⠃⠃")]
    #[case::siblings("F₁  Bb\n  ┌──┐\nF₂  BB bb", "⠠⠋⠰⠼⠁⠀⠀⠠⠃⠃⠀⠒⠕\n⠠⠋⠰⠼⠃⠀⠀⠷⠠⠠⠃⠃⠀⠃⠃⠾")]
    fn writes_pedigrees(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(
            encode(text).map(|cells| braille(&cells)).as_deref(),
            Some(expected)
        );
    }

    #[rstest::rstest]
    #[case::single_generation("P  BB\n |")]
    #[case::unknown_label("Q  BB\nF₁  Bb")]
    #[case::label_only("P\nF₁  Bb")]
    #[case::not_genes("P  BBB\nF₁  Bb")]
    #[case::sibling_not_genes("P  BB\nF₁  Bb xyz")]
    #[case::letter_subscript("P  BB\nFₐ  Bb")]
    #[case::empty_line("P  BB\n\nF₁  Bb")]
    fn leaves_other_pedigrees_alone(#[case] text: &str) {
        assert_eq!(pedigree(text), None);
    }
}
