# Math Subsystem Inventory & Performance Analysis

## Executive Summary

**File Count**: 71 total files (66 rule files + 5 infrastructure files)
**Build Status**: ✅ Clean (release build 3.12s)
**Test Status**: ✅ 106/106 math tests passing
**Architecture**: Hand-written recursive descent parser + token-based rule engine
**Parsing Strategy**: Per-call instantiation (no caching)
**Allocation Hotspots**: parser.rs (9 Vec/String allocations), rule_35/17/21 (3-4 each)

---

## A. File Inventory & Purposes

### Infrastructure Files (5)

| File | Lines | Purpose | Allocations |
|------|-------|---------|-------------|
| **parser.rs** | 1164 | LaTeX/math expression tokenizer (hand-written recursive descent) | 9 (Vec::new, String::new, to_string) |
| **encoder.rs** | 848 | Math token → braille byte encoder; rule registration & dispatch | 1 (Vec::new) |
| **function.rs** | 130 | PHF-based function name lookup (sin, cos, log, lim, etc.) | 0 (static PHF) |
| **math_token_rule.rs** | 127 | MathTokenRule trait & MathTokenEngine dispatch | 1 (Vec::new) |
| **mod.rs** | 93 | Module declarations (rule_1..rule_66) | 0 |

### Rule Files (66)

**Size Distribution** (by lines of code):

| Tier | Rules | Avg Lines | Examples |
|------|-------|-----------|----------|
| **Large** (200+) | 4 | ~300 | rule_12 (446), rule_47 (293), rule_19 (309), rule_18 (258) |
| **Medium** (50-200) | 12 | ~100 | rule_2 (206), rule_7 (251), rule_46 (129), rule_54 (106), rule_57 (101) |
| **Small** (<50) | 50 | ~20 | Most rules (rule_1..rule_66 except above) |

**Allocation Hotspots** (Vec/String per-call):

- **rule_35.rs**: 4 allocations (segment notation, uppercase handling)
- **rule_17.rs**: 3 allocations (prime mark variants)
- **rule_21.rs**: 3 allocations (absolute value/norm)
- **rule_34.rs**: 2 allocations (relation notation)
- **rule_47.rs**: 2 allocations (log base encoding)
- **rule_54.rs**: 4 allocations (partial derivative fractions)

---

## B. LaTeX Parser Deep Dive

### B.1 Parsing Strategy

**Type**: Hand-written recursive descent parser (NOT regex-driven, NOT nom-based)

**Entry Point**: \parse_math_expression(input: &str) -> Result<Vec<MathToken>, String>\

**Key Characteristics**:
- **Per-call instantiation**: Parser state (bracket stack, token buffer) created fresh on each call
- **No caching**: Tokenization tables (operators, function names, symbols) computed per call
- **Lookahead**: Limited lookahead for bracket matching, function detection, subscript/superscript sequences
- **Normalization**: Unicode Mathematical Alphanumeric Symbols (U+1D400–U+1D7FF) normalized to ASCII on input

### B.2 Parsing Phases

1. **Input Normalization** (lines 282-284)
   - Maps Unicode math variants (bold, italic, script, fraktur) to ASCII
   - Allocates: \String\ for normalized input

2. **Special Case Handling** (lines 285-334)
   - Factorial fractions: \
!/m!\ → reversed tokens
   - Underline notation: \X̲\ → fraction conversion
   - Allocates: \Vec<MathToken>\ for special patterns

3. **Main Tokenization Loop** (lines 336-990)
   - Character-by-character scan with lookahead
   - Bracket stack tracking (depth, Korean content, arithmetic operators)
   - Function name detection via \unction::match_function_prefix()\
   - Allocates: \Vec<MathToken>\, \String\ for numbers/Korean phrases, \Vec<GroupState>\ for bracket stack

4. **Post-Processing** (lines 992-1096)
   - Overline wrapping detection
   - Permutation/combination spacing normalization
   - Square root grouping insertion
   - Allocates: \Vec<MathToken>\ for insertions

### B.3 Tokenization Tables

**Function Names** (function.rs):
- **Type**: Static PHF map (compile-time perfect hash)
- **Lookup**: \unction::match_function_prefix(text)\ → O(1) amortized
- **Entries**: 13 functions (sin, cos, tan, sinh, cosh, tanh, csc, sec, cot, arcsin, arccos, arctan, cosec, log, lim)
- **Caching**: ✅ Fully cached (PHF is static)

**Math Symbols** (math_symbol_shortcut.rs):
- **Type**: Static PHF map (compile-time)
- **Lookup**: \math_symbol_shortcut::is_math_symbol_char(c)\ → O(1)
- **Caching**: ✅ Fully cached

**Bracket Kinds** (parser.rs):
- **Type**: Enum (5 variants: MathParen, Grouping, Hangul, Square, Curly)
- **Tracking**: Via \racket_stack: Vec<GroupState>\ (per-call allocation)
- **Caching**: ❌ Stack allocated per call

### B.4 Instantiation & Caching

| Component | Instantiation | Caching | Impact |
|-----------|----------------|---------|--------|
| Parser state | Per call | ❌ None | O(n) allocations per expression |
| Bracket stack | Per call | ❌ None | Grows with nesting depth |
| Token buffer | Per call | ❌ None | Grows with expression length |
| Function lookup | Per call | ✅ PHF (static) | O(1) amortized |
| Symbol lookup | Per call | ✅ PHF (static) | O(1) amortized |
| Normalized input | Per call | ❌ None | String allocation for Unicode normalization |

**Conclusion**: Parser is **NOT cached**. Each \parse_math_expression()\ call allocates fresh state.

---

## C. Encoder Architecture

### C.1 Separation from Main Encoder

**Location**: \libs/braillify/src/rules/math/encoder.rs\ (separate from \libs/braillify/src/encoder.rs\)

**Interaction**:
- **Main encoder** (\src/encoder.rs\): Orchestrates token rules, character encoding
- **Math encoder** (\ules/math/encoder.rs\): Handles math token → braille byte conversion
- **Entry point**: \ncode_math_expression(input: &str) -> Result<Vec<u8>, String>\
- **Called by**: Token-level rules (LatexMathRule, MathExpressionTokenRule) in main encoder

### C.2 Rule Registration & Dispatch

**Engine**: \MathTokenEngine\ (math_token_rule.rs)

**Registration** (encoder.rs lines 781-814):
- Priority 10 — lookahead rules (7 rules)
- Priority 50 — core token rules (13 rules)
- Priority 100 — math symbol dispatch (2 rules)

**Dispatch** (math_token_rule.rs lines 75-103):
- Linear scan through sorted rules
- First matching rule wins
- No caching of rule matches

**Allocations**:
- \Vec<Box<dyn MathTokenRule>>\ created per \uild_math_engine()\ call
- **Problem**: \uild_math_engine()\ called **per math expression** (encoder.rs line 823)

### C.3 MathSymbolRule Dispatch Chain

**Priority**: 100 (runs last, catches all MathSymbol tokens)

**Dispatch Chain** (encoder.rs lines 601-707):
- 30+ dispatch branches (rule_3, rule_4, rule_5, ... rule_65)
- No regex: Pure character matching (Unicode code point checks)
- No caching: Each symbol triggers full dispatch chain scan

---

## D. Function Name Lookup

**Type**: PHF (Perfect Hash Function) — compile-time static map

**Lookup Functions**:
- \match_function_prefix(text: &str) -> Option<(&'static str, &'static [u8])>\ — O(1) amortized
- \ncode_function(name: &str) -> Option<&'static [u8]>\ — O(1) amortized
- \starts_with_function(text: &str) -> bool\ — O(1) amortized

**Caching**: ✅ **Fully cached** (PHF is static, no per-call allocation)

**Performance**: Excellent (no allocations, O(1) lookup)

---

## E. Thread-Local State in rule_12

**Location**: rule_12.rs lines 12-20

**Declarations**:
- \MATRIX_CONTEXT_ACTIVE\: Set when input contains "행렬" (matrix) keyword
- \MATH_MODE_ACTIVE\: Set when testcase has \"context": "math"\

**Lifecycle**:
- Initialization: \Cell::new(false)\ at thread startup
- Activation: Set to \	rue\ during \ncode()\ call
- Deactivation: Set to \alse\ after encoding completes
- Scope: Thread-local (per-thread, not per-call)

**Concern**: ⚠️ **Not reset on error** — if encoding fails mid-call, flag may remain \	rue\ for next call on same thread.

---

## F. Allocation Histogram (rule_*.rs)

### Top 10 Most-Allocating Rules

| Rank | Rule | Allocations | Type | Context |
|------|------|-------------|------|---------|
| 1 | rule_35 | 4 | Vec/String | Segment notation, uppercase handling |
| 2 | rule_17 | 3 | Vec/String | Prime mark variants |
| 3 | rule_21 | 3 | Vec/String | Absolute value/norm |
| 4 | rule_54 | 4 | Vec/String | Partial derivative fractions |
| 5 | rule_34 | 2 | Vec/String | Relation notation |
| 6 | rule_47 | 2 | Vec/String | Log base encoding |
| 7 | rule_27 | 2 | Vec/String | Divisibility symbol |
| 8 | rule_41 | 2 | Vec/String | Perpendicular symbol |
| 9 | rule_25 | 1 | Vec | Sigma bounds |
| 10 | rule_28 | 1 | Vec | Norm symbol |

---

## G. Re-Tokenization & Re-Parsing

**Rules That Re-Parse LaTeX**:
- **rule_47** (log/lim): Normalizes subscript content, maps \/\ → \\u{2044}\
- **rule_54** (partial derivative): Extracts numerator/denominator, re-encodes
- **rule_7** (fraction reversal): Extracts left/right operands, re-encodes

**Rules With Regex/HashMap Per-Call**: None found

---

## H. Suspicious Literal Mappings (꼼수 Audit)

**✅ PASS**: No syllable-level or expression-level literal mappings found.

**Evidence**:
- No \match input { "..." => "..." }\ patterns
- No hardcoded test case lookups
- No input-to-output direct mappings

**Assessment**: All rules use PDF-defined patterns, not test-driven implementations.

---

## I. Performance Hotspots & Optimization Opportunities

| Hotspot | Location | Issue | Impact | Fix |
|---------|----------|-------|--------|-----|
| **Parser per-call** | parser.rs:281 | Fresh Vec/String allocation per expression | O(n) allocations | Cache parser state (Wave 8) |
| **Engine per-call** | encoder.rs:823 | \uild_math_engine()\ called per expression | 22 Box allocations | Cache engine (singleton or thread-local) |
| **Bracket stack** | parser.rs:338 | Vec grows with nesting depth | O(d) allocations | Pre-allocate with capacity |
| **Token buffer** | parser.rs:337 | Vec grows with expression length | O(n) allocations | Pre-allocate with capacity |

---

## J. Summary Table

| Aspect | Status | Details |
|--------|--------|---------|
| **Parsing Strategy** | Hand-written | Recursive descent, per-call instantiation |
| **Caching** | Partial | PHF (function, symbols) cached; parser state not cached |
| **Allocations** | Moderate | 31 Vec/String allocations across 15 rules |
| **Regex Usage** | None | No regex in math subsystem |
| **Literal Mappings** | None | ✅ No 꼼수 violations |
| **Thread-Local State** | 2 flags | MATRIX_CONTEXT_ACTIVE, MATH_MODE_ACTIVE |
| **Build Status** | Clean | ✅ Release build 3.12s |
| **Test Status** | Passing | ✅ 106/106 tests |
| **Performance** | Good | O(n) parsing, O(1) symbol lookup |
| **Wave 8 Ready** | Yes | Parser state caching recommended |

---

## K. Recommendations for Wave 8

1. **Cache MathTokenEngine**: Build once, reuse (singleton or thread-local)
2. **Pool parser allocations**: Reuse Vec/String across calls
3. **Pre-allocate bracket stack**: Estimate from input length
4. **Benchmark**: Measure allocation overhead in real workloads
5. **Consider nom migration**: If parser becomes bottleneck (currently not)
