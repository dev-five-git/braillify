//! Math Braille rules (수학 점자 규정).
//!
//! Each module implements one or more articles from the
//! 2024 Korean Braille Standard — Math section (pages 51-84).
//!
//! Modules are organized by rule number (제N항) for easy reference
//! against the standard document.

// ── Core infrastructure ─────────────────────
pub mod encoder;
pub mod function;
pub mod math_token_rule;
pub mod parser;

// ── 제1항–제10항: 숫자, 연산, 등식, 비교, 괄호, 분수, 소수, 비 ──
pub mod rule_1;
pub mod rule_10;
pub mod rule_2;
pub mod rule_3;
pub mod rule_4;
pub mod rule_5;
pub mod rule_6;
pub mod rule_7;
pub mod rule_8;
pub mod rule_9;

// ── 제11항–제20항: 수식 문장, 로마자, 그리스 문자, 로마 숫자 등 ──
pub mod rule_11;
pub mod rule_12;
pub mod rule_13;
pub mod rule_14;
pub mod rule_15;
pub mod rule_16;
pub mod rule_17;
pub mod rule_18;
pub mod rule_19;
pub mod rule_20;

// ── 제21항–제30항: 절댓값, 근호, 오버라인, 수열, 합, 약수 등 ──
pub mod rule_21;
pub mod rule_22;
pub mod rule_23;
pub mod rule_24;
pub mod rule_25;
pub mod rule_26;
pub mod rule_27;
pub mod rule_28;
pub mod rule_29;
pub mod rule_30;

// ── 제31항–제40항: 합동, 기하 연산, 관계, 선분, 호, 각 등 ──
pub mod rule_31;
pub mod rule_32;
pub mod rule_33;
pub mod rule_34;
pub mod rule_35;
pub mod rule_36;
pub mod rule_37;
pub mod rule_38;
pub mod rule_39;
pub mod rule_40;

// ── 제41항–제50항: 수직, 닮음, 합동, 평행, 함수, 로그, 삼각 등 ──
pub mod rule_41;
pub mod rule_42;
pub mod rule_43;
pub mod rule_44;
pub mod rule_45;
pub mod rule_46;
pub mod rule_47;
pub mod rule_48;
pub mod rule_49;
pub mod rule_50;

// ── 제51항–제60항: 극한, 델타, 미분, 적분, 집합 등 ──
pub mod rule_51;
// 제52항 (Δ, U+0394) is fully captured by `rule_13::is_greek_symbol` and the
// generic math-symbol shortcut table; the dedicated module had only the
// `is_delta_symbol` predicate and the `encode_delta_symbol` wrapper, neither
// of which was reachable from any production path. Removed to satisfy the
// "dead-code elimination after `unreachable!()` probe" policy.
pub mod rule_53;
pub mod rule_54;
pub mod rule_55;
pub mod rule_56;
pub mod rule_57;
pub mod rule_58;
pub mod rule_59;
pub mod rule_60;

// ── 제61항–제66항: 논리, 팩토리얼, 확률, 모자, 그러므로 등 ──
pub mod rule_61;
pub mod rule_62;
pub mod rule_63;
pub mod rule_64;
pub mod rule_65;
pub mod rule_66;
