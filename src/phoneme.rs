//! Phoneme definitions and mappings for the speech synthesizer.
//!
//! This module defines the phoneme inventory for English and Spanish,
//! with mappings to IPA (International Phonetic Alphabet) and acoustic
//! parameters for formant synthesis.

use std::collections::HashMap;

/// A phoneme with its acoustic properties for formant synthesis.
#[derive(Debug, Clone)]
pub struct Phoneme {
    /// The phoneme symbol (ASCII representation).
    pub symbol: &'static str,
    /// IPA representation.
    pub ipa: &'static str,
    /// Phoneme category.
    pub category: PhonemeCategory,
    /// Duration in milliseconds (base value).
    pub duration_ms: u32,
    /// Formant frequencies (F1, F2, F3) in Hz for vowels.
    pub formants: Option<FormantValues>,
    /// Voicing information.
    pub voiced: bool,
}

/// Formant frequency values for vowel synthesis.
#[derive(Debug, Clone, Copy)]
pub struct FormantValues {
    /// First formant frequency (Hz).
    pub f1: f32,
    /// Second formant frequency (Hz).
    pub f2: f32,
    /// Third formant frequency (Hz).
    pub f3: f32,
    /// Bandwidth for F1 (Hz).
    pub b1: f32,
    /// Bandwidth for F2 (Hz).
    pub b2: f32,
    /// Bandwidth for F3 (Hz).
    pub b3: f32,
}

impl FormantValues {
    /// Creates new formant values with default bandwidths.
    pub const fn new(f1: f32, f2: f32, f3: f32) -> Self {
        Self {
            f1,
            f2,
            f3,
            b1: 60.0,
            b2: 90.0,
            b3: 150.0,
        }
    }

    /// Creates new formant values with custom bandwidths.
    pub const fn with_bandwidths(f1: f32, f2: f32, f3: f32, b1: f32, b2: f32, b3: f32) -> Self {
        Self { f1, f2, f3, b1, b2, b3 }
    }
}

/// Categories of phonemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhonemeCategory {
    /// Vowel sounds (monophthongs).
    Vowel,
    /// Diphthong sounds.
    Diphthong,
    /// Plosive/stop consonants.
    Plosive,
    /// Fricative consonants.
    Fricative,
    /// Affricate consonants.
    Affricate,
    /// Nasal consonants.
    Nasal,
    /// Lateral consonants.
    Lateral,
    /// Rhotic consonants.
    Rhotic,
    /// Approximant consonants.
    Approximant,
    /// Silence/pause.
    Silence,
}

/// Phoneme inventory for a language.
#[derive(Debug, Clone)]
pub struct PhonemeInventory {
    /// Map from symbol to phoneme.
    phonemes: HashMap<&'static str, Phoneme>,
    /// Language code.
    pub language: &'static str,
}

impl PhonemeInventory {
    /// Creates a new phoneme inventory for English.
    pub fn english() -> Self {
        let mut phonemes = HashMap::new();

        // Vowels (based on General American English)
        // Formant values are approximate averages for adult speakers
        phonemes.insert("i", Phoneme {
            symbol: "i",
            ipa: "iː",
            category: PhonemeCategory::Vowel,
            duration_ms: 120,
            formants: Some(FormantValues::new(270.0, 2290.0, 3010.0)),
            voiced: true,
        });
        phonemes.insert("I", Phoneme {
            symbol: "I",
            ipa: "ɪ",
            category: PhonemeCategory::Vowel,
            duration_ms: 100,
            formants: Some(FormantValues::new(390.0, 1990.0, 2550.0)),
            voiced: true,
        });
        phonemes.insert("e", Phoneme {
            symbol: "e",
            ipa: "eɪ",
            category: PhonemeCategory::Diphthong,
            duration_ms: 140,
            formants: Some(FormantValues::new(530.0, 1840.0, 2480.0)),
            voiced: true,
        });
        phonemes.insert("E", Phoneme {
            symbol: "E",
            ipa: "ɛ",
            category: PhonemeCategory::Vowel,
            duration_ms: 100,
            formants: Some(FormantValues::new(610.0, 1900.0, 2530.0)),
            voiced: true,
        });
        phonemes.insert("&", Phoneme {
            symbol: "&",
            ipa: "æ",
            category: PhonemeCategory::Vowel,
            duration_ms: 120,
            formants: Some(FormantValues::new(660.0, 1720.0, 2410.0)),
            voiced: true,
        });
        phonemes.insert("A", Phoneme {
            symbol: "A",
            ipa: "ɑː",
            category: PhonemeCategory::Vowel,
            duration_ms: 130,
            formants: Some(FormantValues::new(730.0, 1090.0, 2440.0)),
            voiced: true,
        });
        phonemes.insert("O", Phoneme {
            symbol: "O",
            ipa: "ɔː",
            category: PhonemeCategory::Vowel,
            duration_ms: 120,
            formants: Some(FormantValues::new(570.0, 840.0, 2410.0)),
            voiced: true,
        });
        phonemes.insert("o", Phoneme {
            symbol: "o",
            ipa: "oʊ",
            category: PhonemeCategory::Diphthong,
            duration_ms: 140,
            formants: Some(FormantValues::new(450.0, 1030.0, 2380.0)),
            voiced: true,
        });
        phonemes.insert("U", Phoneme {
            symbol: "U",
            ipa: "ʊ",
            category: PhonemeCategory::Vowel,
            duration_ms: 100,
            formants: Some(FormantValues::new(440.0, 1020.0, 2240.0)),
            voiced: true,
        });
        phonemes.insert("u", Phoneme {
            symbol: "u",
            ipa: "uː",
            category: PhonemeCategory::Vowel,
            duration_ms: 120,
            formants: Some(FormantValues::new(300.0, 870.0, 2240.0)),
            voiced: true,
        });
        phonemes.insert("@", Phoneme {
            symbol: "@",
            ipa: "ə",
            category: PhonemeCategory::Vowel,
            duration_ms: 80,
            formants: Some(FormantValues::new(500.0, 1500.0, 2500.0)),
            voiced: true,
        });
        phonemes.insert("3", Phoneme {
            symbol: "3",
            ipa: "ɜː",
            category: PhonemeCategory::Vowel,
            duration_ms: 120,
            formants: Some(FormantValues::new(580.0, 1380.0, 2530.0)),
            voiced: true,
        });

        // Diphthongs
        phonemes.insert("aI", Phoneme {
            symbol: "aI",
            ipa: "aɪ",
            category: PhonemeCategory::Diphthong,
            duration_ms: 180,
            formants: Some(FormantValues::new(700.0, 1200.0, 2600.0)),
            voiced: true,
        });
        phonemes.insert("aU", Phoneme {
            symbol: "aU",
            ipa: "aʊ",
            category: PhonemeCategory::Diphthong,
            duration_ms: 180,
            formants: Some(FormantValues::new(700.0, 1000.0, 2400.0)),
            voiced: true,
        });
        phonemes.insert("OI", Phoneme {
            symbol: "OI",
            ipa: "ɔɪ",
            category: PhonemeCategory::Diphthong,
            duration_ms: 180,
            formants: Some(FormantValues::new(570.0, 1000.0, 2500.0)),
            voiced: true,
        });

        // Consonants - Plosives
        phonemes.insert("p", Phoneme {
            symbol: "p",
            ipa: "p",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: false,
        });
        phonemes.insert("b", Phoneme {
            symbol: "b",
            ipa: "b",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: true,
        });
        phonemes.insert("t", Phoneme {
            symbol: "t",
            ipa: "t",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: false,
        });
        phonemes.insert("d", Phoneme {
            symbol: "d",
            ipa: "d",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: true,
        });
        phonemes.insert("k", Phoneme {
            symbol: "k",
            ipa: "k",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: false,
        });
        phonemes.insert("g", Phoneme {
            symbol: "g",
            ipa: "g",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: true,
        });

        // Consonants - Fricatives
        phonemes.insert("f", Phoneme {
            symbol: "f",
            ipa: "f",
            category: PhonemeCategory::Fricative,
            duration_ms: 80,
            formants: None,
            voiced: false,
        });
        phonemes.insert("v", Phoneme {
            symbol: "v",
            ipa: "v",
            category: PhonemeCategory::Fricative,
            duration_ms: 80,
            formants: None,
            voiced: true,
        });
        phonemes.insert("T", Phoneme {
            symbol: "T",
            ipa: "θ",
            category: PhonemeCategory::Fricative,
            duration_ms: 80,
            formants: None,
            voiced: false,
        });
        phonemes.insert("D", Phoneme {
            symbol: "D",
            ipa: "ð",
            category: PhonemeCategory::Fricative,
            duration_ms: 80,
            formants: None,
            voiced: true,
        });
        phonemes.insert("s", Phoneme {
            symbol: "s",
            ipa: "s",
            category: PhonemeCategory::Fricative,
            duration_ms: 90,
            formants: None,
            voiced: false,
        });
        phonemes.insert("z", Phoneme {
            symbol: "z",
            ipa: "z",
            category: PhonemeCategory::Fricative,
            duration_ms: 90,
            formants: None,
            voiced: true,
        });
        phonemes.insert("S", Phoneme {
            symbol: "S",
            ipa: "ʃ",
            category: PhonemeCategory::Fricative,
            duration_ms: 100,
            formants: None,
            voiced: false,
        });
        phonemes.insert("Z", Phoneme {
            symbol: "Z",
            ipa: "ʒ",
            category: PhonemeCategory::Fricative,
            duration_ms: 100,
            formants: None,
            voiced: true,
        });
        phonemes.insert("h", Phoneme {
            symbol: "h",
            ipa: "h",
            category: PhonemeCategory::Fricative,
            duration_ms: 60,
            formants: None,
            voiced: false,
        });

        // Consonants - Affricates
        phonemes.insert("tS", Phoneme {
            symbol: "tS",
            ipa: "tʃ",
            category: PhonemeCategory::Affricate,
            duration_ms: 110,
            formants: None,
            voiced: false,
        });
        phonemes.insert("dZ", Phoneme {
            symbol: "dZ",
            ipa: "dʒ",
            category: PhonemeCategory::Affricate,
            duration_ms: 110,
            formants: None,
            voiced: true,
        });

        // Consonants - Nasals
        phonemes.insert("m", Phoneme {
            symbol: "m",
            ipa: "m",
            category: PhonemeCategory::Nasal,
            duration_ms: 80,
            formants: Some(FormantValues::new(300.0, 1000.0, 2500.0)),
            voiced: true,
        });
        phonemes.insert("n", Phoneme {
            symbol: "n",
            ipa: "n",
            category: PhonemeCategory::Nasal,
            duration_ms: 80,
            formants: Some(FormantValues::new(300.0, 1500.0, 2500.0)),
            voiced: true,
        });
        phonemes.insert("N", Phoneme {
            symbol: "N",
            ipa: "ŋ",
            category: PhonemeCategory::Nasal,
            duration_ms: 80,
            formants: Some(FormantValues::new(300.0, 2000.0, 2500.0)),
            voiced: true,
        });

        // Consonants - Liquids/Approximants
        phonemes.insert("l", Phoneme {
            symbol: "l",
            ipa: "l",
            category: PhonemeCategory::Lateral,
            duration_ms: 70,
            formants: Some(FormantValues::new(350.0, 1100.0, 2700.0)),
            voiced: true,
        });
        phonemes.insert("r", Phoneme {
            symbol: "r",
            ipa: "ɹ",
            category: PhonemeCategory::Rhotic,
            duration_ms: 70,
            formants: Some(FormantValues::new(350.0, 1300.0, 1700.0)),
            voiced: true,
        });
        phonemes.insert("w", Phoneme {
            symbol: "w",
            ipa: "w",
            category: PhonemeCategory::Approximant,
            duration_ms: 60,
            formants: Some(FormantValues::new(300.0, 700.0, 2400.0)),
            voiced: true,
        });
        phonemes.insert("j", Phoneme {
            symbol: "j",
            ipa: "j",
            category: PhonemeCategory::Approximant,
            duration_ms: 60,
            formants: Some(FormantValues::new(280.0, 2300.0, 3000.0)),
            voiced: true,
        });

        // Silence
        phonemes.insert("_", Phoneme {
            symbol: "_",
            ipa: "",
            category: PhonemeCategory::Silence,
            duration_ms: 100,
            formants: None,
            voiced: false,
        });

        Self {
            phonemes,
            language: "en",
        }
    }

    /// Creates a new phoneme inventory for Spanish.
    pub fn spanish() -> Self {
        let mut phonemes = HashMap::new();

        // Spanish Vowels (5 vowel system)
        phonemes.insert("a", Phoneme {
            symbol: "a",
            ipa: "a",
            category: PhonemeCategory::Vowel,
            duration_ms: 100,
            formants: Some(FormantValues::new(750.0, 1200.0, 2600.0)),
            voiced: true,
        });
        phonemes.insert("e", Phoneme {
            symbol: "e",
            ipa: "e",
            category: PhonemeCategory::Vowel,
            duration_ms: 100,
            formants: Some(FormantValues::new(450.0, 1900.0, 2500.0)),
            voiced: true,
        });
        phonemes.insert("i", Phoneme {
            symbol: "i",
            ipa: "i",
            category: PhonemeCategory::Vowel,
            duration_ms: 100,
            formants: Some(FormantValues::new(270.0, 2300.0, 3000.0)),
            voiced: true,
        });
        phonemes.insert("o", Phoneme {
            symbol: "o",
            ipa: "o",
            category: PhonemeCategory::Vowel,
            duration_ms: 100,
            formants: Some(FormantValues::new(500.0, 900.0, 2500.0)),
            voiced: true,
        });
        phonemes.insert("u", Phoneme {
            symbol: "u",
            ipa: "u",
            category: PhonemeCategory::Vowel,
            duration_ms: 100,
            formants: Some(FormantValues::new(300.0, 800.0, 2300.0)),
            voiced: true,
        });

        // Spanish Consonants - Plosives
        phonemes.insert("p", Phoneme {
            symbol: "p",
            ipa: "p",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: false,
        });
        phonemes.insert("b", Phoneme {
            symbol: "b",
            ipa: "b",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: true,
        });
        phonemes.insert("t", Phoneme {
            symbol: "t",
            ipa: "t",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: false,
        });
        phonemes.insert("d", Phoneme {
            symbol: "d",
            ipa: "d",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: true,
        });
        phonemes.insert("k", Phoneme {
            symbol: "k",
            ipa: "k",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: false,
        });
        phonemes.insert("g", Phoneme {
            symbol: "g",
            ipa: "g",
            category: PhonemeCategory::Plosive,
            duration_ms: 60,
            formants: None,
            voiced: true,
        });

        // Spanish Consonants - Fricatives
        phonemes.insert("f", Phoneme {
            symbol: "f",
            ipa: "f",
            category: PhonemeCategory::Fricative,
            duration_ms: 80,
            formants: None,
            voiced: false,
        });
        phonemes.insert("s", Phoneme {
            symbol: "s",
            ipa: "s",
            category: PhonemeCategory::Fricative,
            duration_ms: 90,
            formants: None,
            voiced: false,
        });
        phonemes.insert("x", Phoneme {
            symbol: "x",
            ipa: "x",
            category: PhonemeCategory::Fricative,
            duration_ms: 80,
            formants: None,
            voiced: false,
        });
        phonemes.insert("T", Phoneme {
            symbol: "T",
            ipa: "θ",
            category: PhonemeCategory::Fricative,
            duration_ms: 80,
            formants: None,
            voiced: false,
        });

        // Spanish Consonants - Affricates
        phonemes.insert("tS", Phoneme {
            symbol: "tS",
            ipa: "tʃ",
            category: PhonemeCategory::Affricate,
            duration_ms: 100,
            formants: None,
            voiced: false,
        });

        // Spanish Consonants - Nasals
        phonemes.insert("m", Phoneme {
            symbol: "m",
            ipa: "m",
            category: PhonemeCategory::Nasal,
            duration_ms: 80,
            formants: Some(FormantValues::new(300.0, 1000.0, 2500.0)),
            voiced: true,
        });
        phonemes.insert("n", Phoneme {
            symbol: "n",
            ipa: "n",
            category: PhonemeCategory::Nasal,
            duration_ms: 80,
            formants: Some(FormantValues::new(300.0, 1500.0, 2500.0)),
            voiced: true,
        });
        phonemes.insert("J", Phoneme {
            symbol: "J",
            ipa: "ɲ",
            category: PhonemeCategory::Nasal,
            duration_ms: 80,
            formants: Some(FormantValues::new(300.0, 1900.0, 2700.0)),
            voiced: true,
        });

        // Spanish Consonants - Liquids
        phonemes.insert("l", Phoneme {
            symbol: "l",
            ipa: "l",
            category: PhonemeCategory::Lateral,
            duration_ms: 70,
            formants: Some(FormantValues::new(350.0, 1100.0, 2700.0)),
            voiced: true,
        });
        phonemes.insert("L", Phoneme {
            symbol: "L",
            ipa: "ʎ",
            category: PhonemeCategory::Lateral,
            duration_ms: 80,
            formants: Some(FormantValues::new(300.0, 1900.0, 2700.0)),
            voiced: true,
        });
        phonemes.insert("r", Phoneme {
            symbol: "r",
            ipa: "ɾ",
            category: PhonemeCategory::Rhotic,
            duration_ms: 40,
            formants: Some(FormantValues::new(400.0, 1400.0, 2200.0)),
            voiced: true,
        });
        phonemes.insert("rr", Phoneme {
            symbol: "rr",
            ipa: "r",
            category: PhonemeCategory::Rhotic,
            duration_ms: 120,
            formants: Some(FormantValues::new(400.0, 1400.0, 2200.0)),
            voiced: true,
        });

        // Spanish Consonants - Approximants
        phonemes.insert("j", Phoneme {
            symbol: "j",
            ipa: "j",
            category: PhonemeCategory::Approximant,
            duration_ms: 60,
            formants: Some(FormantValues::new(280.0, 2300.0, 3000.0)),
            voiced: true,
        });
        phonemes.insert("w", Phoneme {
            symbol: "w",
            ipa: "w",
            category: PhonemeCategory::Approximant,
            duration_ms: 60,
            formants: Some(FormantValues::new(300.0, 700.0, 2400.0)),
            voiced: true,
        });

        // Silence
        phonemes.insert("_", Phoneme {
            symbol: "_",
            ipa: "",
            category: PhonemeCategory::Silence,
            duration_ms: 100,
            formants: None,
            voiced: false,
        });

        Self {
            phonemes,
            language: "es",
        }
    }

    /// Gets a phoneme by its symbol.
    pub fn get(&self, symbol: &str) -> Option<&Phoneme> {
        self.phonemes.get(symbol)
    }

    /// Returns all phonemes in the inventory.
    pub fn all(&self) -> impl Iterator<Item = &Phoneme> {
        self.phonemes.values()
    }

    /// Returns the number of phonemes in the inventory.
    pub fn len(&self) -> usize {
        self.phonemes.len()
    }

    /// Returns true if the inventory is empty.
    pub fn is_empty(&self) -> bool {
        self.phonemes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_inventory() {
        let inv = PhonemeInventory::english();
        assert!(!inv.is_empty());
        assert_eq!(inv.language, "en");
        
        // Test vowel retrieval
        let vowel = inv.get("i").unwrap();
        assert_eq!(vowel.ipa, "iː");
        assert!(vowel.voiced);
        assert!(vowel.formants.is_some());
    }

    #[test]
    fn test_spanish_inventory() {
        let inv = PhonemeInventory::spanish();
        assert!(!inv.is_empty());
        assert_eq!(inv.language, "es");
        
        // Test vowel retrieval
        let vowel = inv.get("a").unwrap();
        assert_eq!(vowel.ipa, "a");
        assert!(vowel.voiced);
    }

    #[test]
    fn test_formant_values() {
        let formants = FormantValues::new(270.0, 2290.0, 3010.0);
        assert_eq!(formants.f1, 270.0);
        assert_eq!(formants.f2, 2290.0);
        assert_eq!(formants.f3, 3010.0);
    }
}
