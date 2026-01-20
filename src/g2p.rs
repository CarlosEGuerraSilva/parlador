//! Text-to-phoneme conversion for English and Spanish.
//!
//! This module implements grapheme-to-phoneme (G2P) conversion rules
//! for converting text to phoneme sequences.

use crate::error::{Result, SynthesizerError};
use crate::phoneme::PhonemeInventory;
use std::collections::HashMap;

/// Grapheme-to-phoneme converter.
pub struct G2PConverter {
    /// Language code.
    language: String,
    /// Phoneme inventory.
    inventory: PhonemeInventory,
    /// Letter-to-phoneme rules.
    rules: HashMap<String, Vec<G2PRule>>,
    /// Exception dictionary.
    exceptions: HashMap<String, String>,
}

/// A grapheme-to-phoneme conversion rule.
#[derive(Debug, Clone)]
struct G2PRule {
    /// Pattern to match (graphemes).
    pattern: String,
    /// Left context (regex-like pattern, empty = any).
    left_context: String,
    /// Right context (regex-like pattern, empty = any).
    right_context: String,
    /// Resulting phonemes.
    phonemes: String,
    /// Priority (higher = applied first).
    priority: i32,
}

impl G2PConverter {
    /// Creates a new G2P converter for English.
    pub fn english() -> Self {
        let mut converter = Self {
            language: "en".to_string(),
            inventory: PhonemeInventory::english(),
            rules: HashMap::new(),
            exceptions: HashMap::new(),
        };
        converter.load_english_rules();
        converter.load_english_exceptions();
        converter
    }

    /// Creates a new G2P converter for Spanish.
    pub fn spanish() -> Self {
        let mut converter = Self {
            language: "es".to_string(),
            inventory: PhonemeInventory::spanish(),
            rules: HashMap::new(),
            exceptions: HashMap::new(),
        };
        converter.load_spanish_rules();
        converter
    }

    /// Load English G2P rules.
    fn load_english_rules(&mut self) {
        // Basic vowel rules
        self.add_rule("a", "", "e$", "e", 10); // 'ate' -> /eɪt/
        self.add_rule("a", "", "", "&", 1);    // default 'a' -> /æ/
        self.add_rule("e", "", "e$", "i", 10); // 'ee' at end
        self.add_rule("e", "", "$", "", 5);    // silent 'e' at end
        self.add_rule("e", "", "", "E", 1);    // default 'e' -> /ɛ/
        self.add_rule("i", "", "e$", "aI", 10); // 'ite' -> /aɪt/
        self.add_rule("i", "", "", "I", 1);    // default 'i' -> /ɪ/
        self.add_rule("o", "", "e$", "o", 10); // 'ose' -> /oʊz/
        self.add_rule("o", "", "", "A", 1);    // default 'o' -> /ɑ/
        self.add_rule("u", "", "e$", "u", 10); // 'ute' -> /uːt/
        self.add_rule("u", "", "", "@", 1);    // default 'u' -> /ʌ/

        // Consonant combinations
        self.add_rule("ch", "", "", "tS", 20);
        self.add_rule("sh", "", "", "S", 20);
        self.add_rule("th", "", "", "T", 15);  // voiceless th
        self.add_rule("ng", "", "", "N", 20);
        self.add_rule("ph", "", "", "f", 20);
        self.add_rule("wh", "", "", "w", 15);
        self.add_rule("ck", "", "", "k", 20);
        self.add_rule("ght", "", "", "t", 25);
        self.add_rule("gh", "", "", "", 20);   // silent gh

        // Single consonants
        self.add_rule("b", "", "", "b", 1);
        self.add_rule("c", "", "[ei]", "s", 10); // soft c
        self.add_rule("c", "", "", "k", 1);      // hard c
        self.add_rule("d", "", "", "d", 1);
        self.add_rule("f", "", "", "f", 1);
        self.add_rule("g", "", "[ei]", "dZ", 8); // soft g (sometimes)
        self.add_rule("g", "", "", "g", 1);      // hard g
        self.add_rule("h", "", "", "h", 1);
        self.add_rule("j", "", "", "dZ", 1);
        self.add_rule("k", "", "", "k", 1);
        self.add_rule("l", "", "", "l", 1);
        self.add_rule("m", "", "", "m", 1);
        self.add_rule("n", "", "", "n", 1);
        self.add_rule("p", "", "", "p", 1);
        self.add_rule("qu", "", "", "kw", 15);
        self.add_rule("r", "", "", "r", 1);
        self.add_rule("s", "", "", "s", 1);
        self.add_rule("t", "", "", "t", 1);
        self.add_rule("v", "", "", "v", 1);
        self.add_rule("w", "", "", "w", 1);
        self.add_rule("x", "", "", "ks", 1);
        self.add_rule("y", "^", "", "j", 10);    // 'y' at start -> /j/
        self.add_rule("y", "", "", "i", 1);      // 'y' elsewhere -> /ɪ/
        self.add_rule("z", "", "", "z", 1);

        // Vowel combinations
        self.add_rule("ea", "", "", "i", 15);
        self.add_rule("ee", "", "", "i", 15);
        self.add_rule("oo", "", "", "u", 15);
        self.add_rule("ou", "", "", "aU", 15);
        self.add_rule("ow", "", "", "aU", 10);
        self.add_rule("oi", "", "", "OI", 15);
        self.add_rule("oy", "", "", "OI", 15);
        self.add_rule("ai", "", "", "e", 15);
        self.add_rule("ay", "", "", "e", 15);
        self.add_rule("au", "", "", "O", 15);
        self.add_rule("aw", "", "", "O", 15);
    }

    /// Load English exception dictionary.
    fn load_english_exceptions(&mut self) {
        // Common exceptions
        self.exceptions.insert("the".to_string(), "D @".to_string());
        self.exceptions.insert("a".to_string(), "@".to_string());
        self.exceptions.insert("is".to_string(), "I z".to_string());
        self.exceptions.insert("are".to_string(), "A r".to_string());
        self.exceptions.insert("was".to_string(), "w A z".to_string());
        self.exceptions.insert("were".to_string(), "w 3 r".to_string());
        self.exceptions.insert("have".to_string(), "h & v".to_string());
        self.exceptions.insert("has".to_string(), "h & z".to_string());
        self.exceptions.insert("had".to_string(), "h & d".to_string());
        self.exceptions.insert("do".to_string(), "d u".to_string());
        self.exceptions.insert("does".to_string(), "d @ z".to_string());
        self.exceptions.insert("did".to_string(), "d I d".to_string());
        self.exceptions.insert("to".to_string(), "t u".to_string());
        self.exceptions.insert("of".to_string(), "@ v".to_string());
        self.exceptions.insert("for".to_string(), "f O r".to_string());
        self.exceptions.insert("with".to_string(), "w I T".to_string());
        self.exceptions.insert("you".to_string(), "j u".to_string());
        self.exceptions.insert("this".to_string(), "D I s".to_string());
        self.exceptions.insert("that".to_string(), "D & t".to_string());
        self.exceptions.insert("one".to_string(), "w @ n".to_string());
        self.exceptions.insert("two".to_string(), "t u".to_string());
        self.exceptions.insert("hello".to_string(), "h E l o".to_string());
        self.exceptions.insert("world".to_string(), "w 3 r l d".to_string());
    }

    /// Load Spanish G2P rules.
    fn load_spanish_rules(&mut self) {
        // Spanish vowels (very regular)
        self.add_rule("a", "", "", "a", 1);
        self.add_rule("e", "", "", "e", 1);
        self.add_rule("i", "", "", "i", 1);
        self.add_rule("o", "", "", "o", 1);
        self.add_rule("u", "", "", "u", 1);

        // Accented vowels (same sounds)
        self.add_rule("á", "", "", "a", 1);
        self.add_rule("é", "", "", "e", 1);
        self.add_rule("í", "", "", "i", 1);
        self.add_rule("ó", "", "", "o", 1);
        self.add_rule("ú", "", "", "u", 1);
        self.add_rule("ü", "", "", "u", 1);

        // Consonant combinations
        self.add_rule("ch", "", "", "tS", 20);
        self.add_rule("ll", "", "", "L", 20);
        self.add_rule("rr", "", "", "rr", 20);
        self.add_rule("ñ", "", "", "J", 20);
        self.add_rule("qu", "", "[ei]", "k", 20);
        self.add_rule("gu", "", "[ei]", "g", 20);

        // C rules
        self.add_rule("c", "", "[ei]", "T", 10);  // ceceo/seseo (using ceceo)
        self.add_rule("c", "", "", "k", 1);

        // G rules
        self.add_rule("g", "", "[ei]", "x", 10);  // soft g
        self.add_rule("g", "", "", "g", 1);       // hard g

        // Single consonants
        self.add_rule("b", "", "", "b", 1);
        self.add_rule("d", "", "", "d", 1);
        self.add_rule("f", "", "", "f", 1);
        self.add_rule("h", "", "", "", 1);        // silent h
        self.add_rule("j", "", "", "x", 1);
        self.add_rule("k", "", "", "k", 1);
        self.add_rule("l", "", "", "l", 1);
        self.add_rule("m", "", "", "m", 1);
        self.add_rule("n", "", "", "n", 1);
        self.add_rule("p", "", "", "p", 1);
        self.add_rule("r", "^", "", "rr", 5);     // initial r is trilled
        self.add_rule("r", "", "", "r", 1);
        self.add_rule("s", "", "", "s", 1);
        self.add_rule("t", "", "", "t", 1);
        self.add_rule("v", "", "", "b", 1);       // v = b in Spanish
        self.add_rule("w", "", "", "w", 1);
        self.add_rule("x", "", "", "ks", 1);
        self.add_rule("y", "", "$", "i", 10);     // 'y' at end -> /i/
        self.add_rule("y", "", "", "j", 1);       // 'y' elsewhere
        self.add_rule("z", "", "", "T", 1);       // ceceo
    }

    /// Add a G2P rule.
    fn add_rule(&mut self, pattern: &str, left_context: &str, right_context: &str, phonemes: &str, priority: i32) {
        let rule = G2PRule {
            pattern: pattern.to_string(),
            left_context: left_context.to_string(),
            right_context: right_context.to_string(),
            phonemes: phonemes.to_string(),
            priority,
        };

        let key = pattern.chars().next().unwrap_or('?').to_string();
        self.rules.entry(key.clone()).or_default().push(rule);
        
        // Sort rules by priority (descending) and pattern length (descending)
        if let Some(rules) = self.rules.get_mut(&key) {
            rules.sort_by(|a, b| {
                b.priority.cmp(&a.priority)
                    .then_with(|| b.pattern.len().cmp(&a.pattern.len()))
            });
        }
    }

    /// Convert text to phoneme sequence.
    pub fn convert(&self, text: &str) -> Result<String> {
        let normalized = self.normalize(text);
        let words: Vec<&str> = normalized.split_whitespace().collect();
        let mut result = Vec::new();

        for word in words {
            let phonemes = self.convert_word(word)?;
            if !phonemes.is_empty() {
                result.push(phonemes);
            }
        }

        Ok(result.join(" _ "))
    }

    /// Normalize text for processing.
    fn normalize(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphabetic() || c.is_whitespace() || *c == '\'' || *c == '-')
            .collect()
    }

    /// Convert a single word to phonemes.
    fn convert_word(&self, word: &str) -> Result<String> {
        // Check exceptions first
        if let Some(phonemes) = self.exceptions.get(word) {
            return Ok(phonemes.clone());
        }

        let chars: Vec<char> = word.chars().collect();
        let mut result = Vec::new();
        let mut i = 0;

        while i < chars.len() {
            let remaining = &word[word.char_indices().nth(i).map(|(idx, _)| idx).unwrap_or(word.len())..];
            
            if let Some((phonemes, consumed)) = self.apply_rules(&chars, i, remaining) {
                if !phonemes.is_empty() {
                    result.push(phonemes);
                }
                i += consumed;
            } else {
                // No rule matched, skip character
                i += 1;
            }
        }

        Ok(result.join(" "))
    }

    /// Apply G2P rules at the current position.
    fn apply_rules(&self, chars: &[char], pos: usize, remaining: &str) -> Option<(String, usize)> {
        let current_char = chars.get(pos)?.to_string();
        
        if let Some(rules) = self.rules.get(&current_char) {
            for rule in rules {
                if remaining.starts_with(&rule.pattern) {
                    // Check contexts
                    if self.check_left_context(&rule.left_context, chars, pos)
                        && self.check_right_context(&rule.right_context, remaining, rule.pattern.len())
                    {
                        return Some((rule.phonemes.clone(), rule.pattern.chars().count()));
                    }
                }
            }
        }

        // Fallback: skip unrecognized character
        Some((String::new(), 1))
    }

    /// Check left context pattern.
    fn check_left_context(&self, context: &str, _chars: &[char], pos: usize) -> bool {
        if context.is_empty() {
            return true;
        }
        
        match context {
            "^" => pos == 0,  // Start of word
            _ => true,        // TODO: Implement more complex patterns
        }
    }

    /// Check right context pattern.
    fn check_right_context(&self, context: &str, remaining: &str, pattern_len: usize) -> bool {
        if context.is_empty() {
            return true;
        }

        let after_pattern = &remaining[remaining.char_indices()
            .nth(pattern_len)
            .map(|(idx, _)| idx)
            .unwrap_or(remaining.len())..];

        match context {
            "$" => after_pattern.is_empty(),  // End of word
            "[ei]" => after_pattern.starts_with('e') || after_pattern.starts_with('i'),
            "e$" => after_pattern == "e",     // 'e' at end
            _ => true,  // TODO: Implement more patterns
        }
    }

    /// Get the phoneme inventory.
    pub fn inventory(&self) -> &PhonemeInventory {
        &self.inventory
    }

    /// Get the language code.
    pub fn language(&self) -> &str {
        &self.language
    }
}

/// Convert text to IPA phoneme representation.
pub fn text_to_ipa(text: &str, language: &str) -> Result<String> {
    let converter = match language {
        "en" | "english" => G2PConverter::english(),
        "es" | "spanish" => G2PConverter::spanish(),
        _ => return Err(SynthesizerError::UnsupportedLanguage(language.to_string())),
    };

    let phonemes = converter.convert(text)?;
    
    // Convert ASCII phonemes to IPA
    let inventory = converter.inventory();
    let ipa_result: Vec<String> = phonemes
        .split_whitespace()
        .map(|p| {
            if p == "_" {
                " ".to_string()
            } else if let Some(phoneme) = inventory.get(p) {
                phoneme.ipa.to_string()
            } else {
                p.to_string()
            }
        })
        .collect();

    Ok(ipa_result.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_g2p_basic() {
        let g2p = G2PConverter::english();
        let result = g2p.convert("hello").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_english_exception() {
        let g2p = G2PConverter::english();
        let result = g2p.convert("the").unwrap();
        assert_eq!(result, "D @");
    }

    #[test]
    fn test_spanish_g2p_basic() {
        let g2p = G2PConverter::spanish();
        let result = g2p.convert("hola").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_spanish_vowels() {
        let g2p = G2PConverter::spanish();
        let result = g2p.convert("aeiou").unwrap();
        assert_eq!(result, "a e i o u");
    }

    #[test]
    fn test_text_to_ipa_english() {
        let result = text_to_ipa("hello", "en").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_unsupported_language() {
        let result = text_to_ipa("test", "fr");
        assert!(result.is_err());
    }
}
