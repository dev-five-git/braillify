//! 과학 제9·10·13·15·16·25항 — 묵자에서 2차원으로 그린 구조식·전자 점식·가계도.
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
    /// 전자 기호 하나에 든 전자 수와 그 기호의 개수(`::` 은 둘씩 둘).
    Electrons(usize, usize),
}

impl Link {
    fn item(self) -> Item {
        match self {
            Link::Bond(order) => Item::Bond(order),
            Link::Electrons(count, marks) => Item::Electrons(count * marks),
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
        dots => {
            let counts: Vec<usize> = dots
                .iter()
                .map(|mark| electrons(*mark))
                .collect::<Option<_>>()?;
            counts
                .iter()
                .all(|count| *count == counts[0])
                .then(|| Link::Electrons(counts[0], counts.len()))
        }
    }
}

fn vertical_link(mark: char) -> Option<Link> {
    match mark {
        '|' | '│' => Some(Link::Bond(1)),
        '‖' => Some(Link::Bond(2)),
        '⦀' => Some(Link::Bond(3)),
        _ => electrons(mark).map(|count| Link::Electrons(count, 1)),
    }
}

/// 원소 앞뒤에 홀로 붙은 전자. 비어 있으면 `Some(None)`, 결합선이면 `None`.
fn dangling(gap: &[char]) -> Option<Option<Link>> {
    match horizontal_link(gap) {
        None if gap.iter().all(|c| *c == ' ') => Some(None),
        Some(link @ Link::Electrons(..)) => Some(Some(link)),
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

struct Mark {
    row: usize,
    column: usize,
    link: Link,
}

struct Diagram {
    atoms: Vec<Atom>,
    chains: Vec<Chain>,
    marks: Vec<Mark>,
    neighbours: Vec<Neighbours>,
    /// 도식 위에 따로 적은 화학식(제11항의 `CH₄`) — 첫 줄에서 어디에도 잇지 않은 원소.
    caption: Option<usize>,
    main: usize,
}

fn parse(text: &str) -> Option<Diagram> {
    let mut atoms = Vec::new();
    let mut chains = Vec::new();
    let mut marks = Vec::new();
    for (row, line) in text.lines().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        if chars.iter().copied().any(is_atom_char) {
            chains.push(atom_row(row, &chars, &mut atoms)?);
            continue;
        }
        let row_marks: Vec<(usize, char)> = chars
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, ch)| *ch != ' ')
            .collect();
        if row_marks.is_empty() {
            return None;
        }
        for (column, mark) in row_marks {
            marks.push(Mark {
                row,
                column,
                link: vertical_link(mark)?,
            });
        }
    }
    if marks.is_empty() {
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
    for mark in &marks {
        let above = atom_at(&atoms, mark.row.checked_sub(1), mark.column);
        let below = atom_at(&atoms, Some(mark.row + 1), mark.column);
        match (above, below, mark.link) {
            (None, None, _) | (None, _, Link::Bond(_)) | (_, None, Link::Bond(_)) => return None,
            _ => {}
        }
        for (end, side, other) in [(above, DOWN, below), (below, UP, above)] {
            if let Some(end) = end {
                let slot = &mut neighbours[end].vertical[side];
                if slot.replace((mark.link, other)).is_some() {
                    return None;
                }
            }
        }
    }

    let isolated = |atom: usize| {
        let n = &neighbours[atom];
        n.left.is_none() && n.right.is_none() && n.vertical.iter().all(Option::is_none)
    };
    let caption = chains
        .first()
        .filter(|chain| {
            chains.len() > 1
                && chain.atoms.len() == 1
                && chain.lead.is_none()
                && chain.trail.is_none()
                && atoms[chain.atoms[0]].row == 0
                && isolated(chain.atoms[0])
        })
        .map(|chain| chain.atoms[0]);
    let main = (0..chains.len())
        .filter(|at| caption != Some(chains[*at].atoms[0]))
        .min_by_key(|at| {
            let first = &atoms[chains[*at].atoms[0]];
            (first.columns.start, first.row)
        })?;
    Some(Diagram {
        atoms,
        chains,
        marks,
        neighbours,
        caption,
        main,
    })
}

fn atom_at(atoms: &[Atom], row: Option<usize>, column: usize) -> Option<usize> {
    let row = row?;
    atoms
        .iter()
        .position(|atom| atom.row == row && atom.columns.contains(&column))
}

/// 과학 제9항 1·제10항 3·제15항 1·제16항 — 2차원 구조식과 전자 점식을 기호 표기
/// 형식으로 적는다. 왼쪽에서 오른쪽으로, 위쪽에서 아래쪽으로 풀어 적는다. 도식
/// 위의 화학식은 적지 않는다(제10항의 예).
fn symbol_form(diagram: &Diagram) -> Option<Vec<u8>> {
    let side_electrons =
        diagram.chains.iter().enumerate().any(|(at, chain)| {
            at != diagram.main && (chain.lead.is_some() || chain.trail.is_some())
        });
    if side_electrons {
        return None;
    }
    let mut walk = Walk {
        atoms: &diagram.atoms,
        neighbours: &diagram.neighbours,
        seen: vec![false; diagram.atoms.len()],
        vertical_links: 0,
        items: Vec::new(),
    };
    if let Some(caption) = diagram.caption {
        walk.seen[caption] = true;
    }
    walk.main_chain(&diagram.chains[diagram.main])?;
    let complete = walk.seen.iter().all(|seen| *seen) && walk.vertical_links == diagram.marks.len();
    complete
        .then(|| formula::encode(&walk.items).ok())
        .flatten()
}

/// 전자 기호마다 전자 수만큼 ⠔ 을 적고 기호 사이를 한 칸 띄운다(제17항 2·3).
fn electron_cells(count: usize, marks: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for mark in 0..marks {
        if mark > 0 {
            out.push(0);
        }
        out.extend(std::iter::repeat_n(decode_unicode('⠔'), count));
    }
    out
}

fn bond_count(order: u8) -> u8 {
    decode_unicode(match order {
        1 => '⠂',
        2 => '⠆',
        _ => '⠒',
    })
}

/// 공간 표기 형식의 결합선과 전자 — 가로 결합선 ⠠⠤, 세로 결합선 ⠸ 뒤에 결합 수를
/// 적는다(제11항 4).
fn link_cells(link: Link, vertical: bool) -> Vec<u8> {
    match link {
        Link::Bond(order) if vertical => vec![decode_unicode('⠸'), bond_count(order)],
        Link::Bond(order) => vec![decode_unicode('⠠'), decode_unicode('⠤'), bond_count(order)],
        Link::Electrons(count, marks) => electron_cells(count, marks),
    }
}

struct Row {
    cells: Vec<u8>,
    starts: Vec<(usize, usize)>,
}

type Encoder = fn(&[Item]) -> Result<Vec<u8>, String>;

/// 과학 제13항 6 — 세로 결합선은 원소 묶음에서 수소가 아닌 첫 원소(`H₂C` 의 `C`)의 첫
/// 칸에 닿는다. 그 칸이 묶음의 몇째 칸인지.
fn bonding_cell(items: &[Item], encode: Encoder) -> usize {
    let Some(element) = items
        .iter()
        .position(|item| matches!(item, Item::Capital { symbol, .. } if symbol != "H"))
    else {
        return 0;
    };
    let through = encode(&items[..=element]).map_or(0, |cells| cells.len());
    let alone = encode(&items[element..=element]).map_or(0, |cells| cells.len());
    through.saturating_sub(alone)
}

fn lay_out_chain(chain: &Chain, atom_cells: &[Vec<u8>], dots: bool) -> Row {
    let mut row = Row {
        cells: Vec::new(),
        starts: Vec::new(),
    };
    let mut segments: Vec<(Vec<u8>, Option<usize>)> = Vec::new();
    segments.extend(chain.lead.map(|link| (link_cells(link, false), None)));
    for (at, &atom) in chain.atoms.iter().enumerate() {
        if at > 0 {
            segments.push((link_cells(chain.links[at - 1], false), None));
        }
        segments.push((atom_cells[atom].clone(), Some(atom)));
    }
    segments.extend(chain.trail.map(|link| (link_cells(link, false), None)));
    for (at, (cells, atom)) in segments.into_iter().enumerate() {
        if dots && at > 0 {
            row.cells.push(0);
        }
        if let Some(atom) = atom {
            row.starts.push((atom, row.cells.len()));
        }
        row.cells.extend(cells);
    }
    row
}

/// 과학 제9항 2·제11항·제13항 6·제15항 2·제17항 — 구조식과 전자 점식을 모양대로 여러
/// 줄에 적는다. 세로로 잇는 결합선·전자와 위아래 원소는 이어지는 원소의 첫 칸(대문자
/// 기호표가 있으면 그 칸)에 맞추고, 도식 위의 화학식은 첫 줄에 적는다.
fn spatial_form(diagram: &Diagram) -> Option<Vec<u8>> {
    let links = diagram
        .chains
        .iter()
        .flat_map(|chain| chain.links.iter().chain(&chain.lead).chain(&chain.trail))
        .chain(diagram.marks.iter().map(|mark| &mark.link));
    let dots = links
        .clone()
        .any(|link| matches!(link, Link::Electrons(..)));
    // 제11항 2 — 한 글자 원소가 붙어 나오는 묶음(`OH`, `CH₃`)이 있으면 대문자 구절표로
    // 감싸고 원소 기호마다의 대문자 기호표는 적지 않는다.
    let passage = !dots
        && diagram.atoms.iter().enumerate().any(|(at, atom)| {
            Some(at) != diagram.caption
                && atom
                    .items
                    .iter()
                    .filter(|item| matches!(item, Item::Capital { symbol, element: true, .. } if symbol.len() == 1))
                    .count()
                    >= 2
        });
    let encoder = |at: usize| -> Encoder {
        if passage && Some(at) != diagram.caption {
            formula::encode_in_phrase
        } else {
            formula::encode
        }
    };
    let atom_cells = diagram
        .atoms
        .iter()
        .enumerate()
        .map(|(at, atom)| encoder(at)(&atom.items))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let rows: Vec<Row> = diagram
        .chains
        .iter()
        .map(|chain| lay_out_chain(chain, &atom_cells, dots))
        .collect();
    let mut chain_of = vec![0; diagram.atoms.len()];
    let mut anchor_of = vec![0; diagram.atoms.len()];
    for (at, row) in rows.iter().enumerate() {
        for &(atom, start) in &row.starts {
            chain_of[atom] = at;
            anchor_of[atom] = start + bonding_cell(&diagram.atoms[atom].items, encoder(atom));
        }
    }

    let mut offsets: Vec<Option<isize>> = vec![None; rows.len()];
    offsets[diagram.main] = Some(0);
    let column = |atom: usize, offsets: &[Option<isize>]| {
        offsets[chain_of[atom]].map(|offset| offset + anchor_of[atom] as isize)
    };
    loop {
        let mut placed = false;
        for mark in &diagram.marks {
            let above = atom_at(&diagram.atoms, mark.row.checked_sub(1), mark.column);
            let below = atom_at(&diagram.atoms, Some(mark.row + 1), mark.column);
            let (Some(above), Some(below)) = (above, below) else {
                continue;
            };
            match (column(above, &offsets), column(below, &offsets)) {
                (Some(top), None) => {
                    offsets[chain_of[below]] = Some(top - anchor_of[below] as isize);
                    placed = true;
                }
                (None, Some(bottom)) => {
                    offsets[chain_of[above]] = Some(bottom - anchor_of[above] as isize);
                    placed = true;
                }
                (Some(top), Some(bottom)) if top != bottom => return None,
                _ => {}
            }
        }
        if !placed {
            break;
        }
    }
    let unplaced = offsets.iter().enumerate().any(|(at, offset)| {
        offset.is_none() && Some(diagram.chains[at].atoms[0]) != diagram.caption
    });
    if unplaced {
        return None;
    }
    let margin = offsets.iter().flatten().copied().min().unwrap_or(0);

    let mut lines: Vec<(usize, Vec<u8>)> = Vec::new();
    for (at, row) in rows.iter().enumerate() {
        let line_row = diagram.atoms[diagram.chains[at].atoms[0]].row;
        match offsets[at] {
            Some(offset) => {
                let mut line = vec![0; (offset - margin) as usize];
                line.extend(&row.cells);
                lines.push((line_row, line));
            }
            None => lines.push((line_row, row.cells.clone())),
        }
    }
    let mut mark_lines: std::collections::BTreeMap<usize, Vec<(usize, Vec<u8>)>> =
        std::collections::BTreeMap::new();
    for mark in &diagram.marks {
        let anchor = atom_at(&diagram.atoms, mark.row.checked_sub(1), mark.column)
            .or_else(|| atom_at(&diagram.atoms, Some(mark.row + 1), mark.column))?;
        let at = (column(anchor, &offsets)? - margin) as usize;
        mark_lines
            .entry(mark.row)
            .or_default()
            .push((at, link_cells(mark.link, true)));
    }
    for (row, mut placed) in mark_lines {
        placed.sort_by_key(|(at, _)| *at);
        let mut line = Vec::new();
        for (at, cells) in placed {
            (line.len() <= at).then_some(())?;
            line.resize(at, 0);
            line.extend(cells);
        }
        lines.push((row, line));
    }
    lines.sort_by_key(|(row, _)| *row);

    let caption_row = diagram.caption.map(|atom| diagram.atoms[atom].row);
    let (caption, body): (Vec<_>, Vec<_>) = lines
        .into_iter()
        .partition(|(row, _)| Some(*row) == caption_row);
    let mut out_lines: Vec<Vec<u8>> = caption.into_iter().map(|(_, line)| line).collect();
    if passage {
        out_lines.push(cells_of("⠐⠐⠿⠠⠠⠠"));
    }
    out_lines.extend(body.into_iter().map(|(_, line)| line));
    if passage {
        out_lines.push(cells_of("⠐⠐⠿⠠⠄"));
    }
    Some(out_lines.join(&LINE_BREAK))
}

fn cells_of(pattern: &str) -> Vec<u8> {
    pattern.chars().map(decode_unicode).collect()
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

/// 여러 줄로 그린 과학 도식을 점자로 적는다. 도식이 아니면 `None`. `spatial` 이면
/// 구조식과 전자 점식을 공간 표기 형식으로 적는다(제9항·제15항은 두 형식을 다 둔다).
pub(crate) fn encode(text: &str, spatial: bool) -> Option<Vec<u8>> {
    if !text.contains('\n') {
        return None;
    }
    parse(text)
        .and_then(|diagram| {
            if spatial {
                spatial_form(&diagram)
            } else {
                symbol_form(&diagram)
            }
        })
        .or_else(|| pedigree(text))
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
    #[case::caption_is_left_out("OH₂\n  H\n  |\nH-O", "⠠⠠⠠⠓⠰⠂⠕⠬⠰⠂⠓⠠⠄")]
    fn writes_diagrams_in_symbol_form(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(
            encode(text, false).map(|cells| braille(&cells)).as_deref(),
            Some(expected)
        );
    }

    #[rstest::rstest]
    #[case::ammonia(
        "  H\n  |\nH-N\n  |\n  H",
        "     ,h\n     _1\n,h,-1,n\n     _1\n     ,h"
    )]
    #[case::caption_first(
        "NH₃\n  H\n  |\nH-N\n  |\n  H",
        ",n,h;#c\n     ,h\n     _1\n,h,-1,n\n     _1\n     ,h"
    )]
    #[case::grouped_atoms_in_a_passage("CH₃-OH\n|\nH", "\"\"=,,,\nch;#c,-1oh\n_1\nh\n\"\"=,'")]
    #[case::caption_outside_the_passage(
        "CH₄O\nCH₃-OH\n|\nH",
        ",,,ch;#do,'\n\"\"=,,,\nch;#c,-1oh\n_1\nh\n\"\"=,'"
    )]
    #[case::row_above_aligns_to_its_atom("  H-O\n  |\nH-C", "     ,h,-1,o\n     _1\n,h,-1,c")]
    #[case::electron_pairs(" ‥\n:F:F:", "   99\n99 ,f 99 ,f 99")]
    #[case::shared_pair_between_rows(
        "  :O:\n   ‥\n:O:P:O:",
        "      99 ,o 99\n         99\n99 ,o 99 ,p 99 ,o 99"
    )]
    #[case::triple_vertical("N\n⦀\nN", ",n\n_3\n,n")]
    #[case::three_electrons("N⋮N\n‥", ",n 999 ,n\n99")]
    #[case::square_ring(
        "H₂C - CH₂\n  |     |\nH₂C - CH₂",
        "\"\"=,,,\nh;#b\"c,-1ch;#b\n     _1  _1\nh;#b\"c,-1ch;#b\n\"\"=,'"
    )]
    #[case::bond_under_the_carbon(
        "H₃C-OH\n  |\n  H",
        "\"\"=,,,\nh;#c\"c,-1oh\n     _1\n     h\n\"\"=,'"
    )]
    fn writes_diagrams_in_spatial_form(#[case] text: &str, #[case] internal: &str) {
        let expected: String = internal
            .chars()
            .map(|ch| match ch {
                '\n' => '\n',
                ' ' => '⠀',
                ',' => '⠠',
                '-' => '⠤',
                '_' => '⠸',
                '"' => '⠐',
                '=' => '⠿',
                '\'' => '⠄',
                ';' => '⠰',
                '#' => '⠼',
                '1' => '⠂',
                '2' => '⠆',
                '3' => '⠒',
                '9' => '⠔',
                'a' => '⠁',
                'b' => '⠃',
                'c' => '⠉',
                'd' => '⠙',
                'f' => '⠋',
                'h' => '⠓',
                'n' => '⠝',
                'o' => '⠕',
                'p' => '⠏',
                _ => unreachable!("unmapped internal cell {ch}"),
            })
            .collect();
        assert_eq!(
            encode(text, true).map(|cells| braille(&cells)).as_deref(),
            Some(expected.as_str())
        );
    }

    #[rstest::rstest]
    #[case::unconnected_row("H\n|\nH\nO-H")]
    #[case::rows_disagree("O:O\n‥ ‥\nO⋮O")]
    #[case::pedigree_is_not_spatial("P  BB × bb\n    |\nF₁  Bb")]
    fn leaves_other_text_alone_in_spatial_form(#[case] text: &str) {
        assert_eq!(parse(text).and_then(|diagram| spatial_form(&diagram)), None);
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
        assert_eq!(encode(text, false), None);
    }

    #[rstest::rstest]
    #[case::cross_then_offspring("P  BB × bb\n    |\nF₁  Bb", "⠰⠠⠏⠀⠀⠠⠠⠃⠃⠀⠡⠀⠃⠃⠀⠒⠕\n⠠⠋⠰⠼⠁⠀⠀⠠⠃⠃")]
    #[case::siblings("F₁  Bb\n  ┌──┐\nF₂  BB bb", "⠠⠋⠰⠼⠁⠀⠀⠠⠃⠃⠀⠒⠕\n⠠⠋⠰⠼⠃⠀⠀⠷⠠⠠⠃⠃⠀⠃⠃⠾")]
    fn writes_pedigrees(#[case] text: &str, #[case] expected: &str) {
        assert_eq!(
            encode(text, false).map(|cells| braille(&cells)).as_deref(),
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
