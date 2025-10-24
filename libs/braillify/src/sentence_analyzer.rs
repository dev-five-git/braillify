use crate::utils;

/// 문장의 언어 주체를 판단하는 분석기
pub struct SentenceAnalyzer;

impl SentenceAnalyzer {
    /// 문장을 공백으로 단어 분리
    pub fn parse_sentence(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
    
    /// 단어가 혼합 단어인지 확인 (영어+한글+특수기호)
    pub fn is_mixed_word(word: &str) -> bool {
        let has_english = word.chars().any(|c| c.is_ascii_alphabetic());
        let has_korean = word.chars().any(|c| utils::is_korean_char(c));
        
        // 영어와 한글이 모두 있으면 혼합 단어
        has_english && has_korean
    }
    
    /// 문장에 혼합 단어가 있는지 확인
    pub fn has_mixed_words(text: &str) -> bool {
        let words = Self::parse_sentence(text);
        words.iter().any(|word| Self::is_mixed_word(word))
    }
    
    /// 문장의 언어 주체 판단
    pub fn determine_language_dominance(text: &str) -> LanguageDominance {
        // 1. 혼합 단어가 있으면 한글 문장
        if Self::has_mixed_words(text) {
            return LanguageDominance::Korean;
        }
        
        // 2. 혼합 단어가 없으면 비율 기반 판단
        let words = Self::parse_sentence(text);
        let total_words = words.len();
        
        if total_words == 0 {
            return LanguageDominance::English;
        }
        
        let korean_words = words.iter()
            .filter(|word| word.chars().any(|c| utils::is_korean_char(c)))
            .count();
        
        let english_words = words.iter()
            .filter(|word| word.chars().all(|c| c.is_ascii_alphabetic()))
            .count();
        
        let korean_ratio = korean_words as f64 / total_words as f64;
        let english_ratio = english_words as f64 / total_words as f64;
        
        if korean_ratio > english_ratio {
            LanguageDominance::Korean
        } else if english_ratio > korean_ratio {
            LanguageDominance::English
        } else {
            // 비율이 같으면 첫 단어 기준
            if let Some(first_word) = words.first() {
                if first_word.chars().any(|c| utils::is_korean_char(c)) {
                    LanguageDominance::Korean
                } else {
                    LanguageDominance::English
                }
            } else {
                LanguageDominance::English
            }
        }
    }
}

/// 문장의 언어 주체
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LanguageDominance {
    Korean,   // 한글이 주체
    English,  // 영어가 주체
}

