//! Prosody and intonation control for speech synthesis.
//!
//! This module provides prosodic features like pitch contours,
//! stress patterns, and natural intonation for different sentence types.

/// Sentence type for prosody determination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SentenceType {
    /// Declarative statement (falling intonation).
    #[default]
    Statement,
    /// Yes/no question (rising intonation).
    Question,
    /// Wh-question (falling intonation).
    WhQuestion,
    /// Exclamation (emphasized).
    Exclamation,
    /// Command/imperative.
    Command,
}

impl SentenceType {
    /// Detect sentence type from text.
    #[must_use]
    pub fn detect(text: &str) -> Self {
        let trimmed = text.trim();
        
        // Check for question mark
        if trimmed.ends_with('?') || trimmed.starts_with('¿') {
            // Check if it's a wh-question (starts with question word)
            // Skip leading punctuation for Spanish
            let lower = trimmed.to_lowercase();
            let content = lower.trim_start_matches(|c: char| !c.is_alphabetic());
            
            let wh_words = ["what", "who", "where", "when", "why", "how", "which", "whose",
                           "qué", "quién", "dónde", "cuándo", "por qué", "cómo", "cuál"];
            for word in wh_words {
                if content.starts_with(word) {
                    return SentenceType::WhQuestion;
                }
            }
            return SentenceType::Question;
        }
        
        // Check for exclamation
        if trimmed.ends_with('!') || trimmed.starts_with('¡') {
            return SentenceType::Exclamation;
        }
        
        // Default to statement
        SentenceType::Statement
    }

    /// Get the pitch contour pattern for this sentence type.
    #[must_use]
    pub fn pitch_contour(&self) -> PitchContour {
        match self {
            SentenceType::Statement => PitchContour::Falling,
            SentenceType::Question => PitchContour::Rising,
            SentenceType::WhQuestion => PitchContour::FallingRising,
            SentenceType::Exclamation => PitchContour::Emphasized,
            SentenceType::Command => PitchContour::Flat,
        }
    }
}

/// Pitch contour patterns for intonation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PitchContour {
    /// Flat contour (no change).
    #[default]
    Flat,
    /// Rising contour (questions).
    Rising,
    /// Falling contour (statements).
    Falling,
    /// Falling-rising contour (wh-questions, continuation).
    FallingRising,
    /// Emphasized contour (exclamations).
    Emphasized,
}

/// Prosody configuration for speech synthesis.
#[derive(Debug, Clone)]
pub struct ProsodyConfig {
    /// Pitch multiplier (1.0 = normal).
    pub pitch_multiplier: f32,
    /// Rate multiplier (1.0 = normal).
    pub rate_multiplier: f32,
    /// Volume multiplier (1.0 = normal).
    pub volume_multiplier: f32,
    /// Pitch contour pattern.
    pub contour: PitchContour,
    /// Emphasis level (0.0 = none, 1.0 = strong).
    pub emphasis: f32,
}

impl Default for ProsodyConfig {
    fn default() -> Self {
        Self {
            pitch_multiplier: 1.0,
            rate_multiplier: 1.0,
            volume_multiplier: 1.0,
            contour: PitchContour::Flat,
            emphasis: 0.0,
        }
    }
}

impl ProsodyConfig {
    /// Creates a new prosody configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pitch multiplier.
    #[must_use]
    pub fn with_pitch(mut self, multiplier: f32) -> Self {
        self.pitch_multiplier = multiplier.clamp(0.5, 2.0);
        self
    }

    /// Sets the rate multiplier.
    #[must_use]
    pub fn with_rate(mut self, multiplier: f32) -> Self {
        self.rate_multiplier = multiplier.clamp(0.25, 4.0);
        self
    }

    /// Sets the volume multiplier.
    #[must_use]
    pub fn with_volume(mut self, multiplier: f32) -> Self {
        self.volume_multiplier = multiplier.clamp(0.0, 2.0);
        self
    }

    /// Sets the pitch contour.
    #[must_use]
    pub fn with_contour(mut self, contour: PitchContour) -> Self {
        self.contour = contour;
        self
    }

    /// Sets the emphasis level.
    #[must_use]
    pub fn with_emphasis(mut self, emphasis: f32) -> Self {
        self.emphasis = emphasis.clamp(0.0, 1.0);
        self
    }

    /// Creates prosody config for a sentence based on its type.
    #[must_use]
    pub fn from_sentence_type(sentence_type: SentenceType) -> Self {
        match sentence_type {
            SentenceType::Statement => Self::default().with_contour(PitchContour::Falling),
            SentenceType::Question => Self::default()
                .with_contour(PitchContour::Rising)
                .with_pitch(1.1),
            SentenceType::WhQuestion => Self::default()
                .with_contour(PitchContour::FallingRising),
            SentenceType::Exclamation => Self::default()
                .with_contour(PitchContour::Emphasized)
                .with_emphasis(0.5)
                .with_volume(1.2),
            SentenceType::Command => Self::default()
                .with_contour(PitchContour::Flat)
                .with_emphasis(0.3),
        }
    }

    /// Calculate pitch multiplier at a given position in the phrase.
    ///
    /// # Arguments
    /// * `position` - Position in the phrase (0.0 to 1.0)
    ///
    /// # Returns
    /// Pitch multiplier for the given position.
    #[must_use]
    pub fn pitch_at_position(&self, position: f32) -> f32 {
        let pos = position.clamp(0.0, 1.0);
        
        let contour_modifier = match self.contour {
            PitchContour::Flat => 1.0,
            PitchContour::Rising => {
                // Gradual rise, more pronounced at the end
                1.0 + 0.3 * pos * pos
            }
            PitchContour::Falling => {
                // Start slightly high, fall toward the end
                1.1 - 0.15 * pos
            }
            PitchContour::FallingRising => {
                // Fall then rise (U-shape)
                let mid = 0.5;
                if pos < mid {
                    1.05 - 0.15 * (pos / mid)
                } else {
                    0.9 + 0.2 * ((pos - mid) / mid)
                }
            }
            PitchContour::Emphasized => {
                // High start, strong fall, slight rise at end
                if pos < 0.3 {
                    1.2 - 0.2 * (pos / 0.3)
                } else if pos < 0.8 {
                    1.0 - 0.1 * ((pos - 0.3) / 0.5)
                } else {
                    0.9 + 0.1 * ((pos - 0.8) / 0.2)
                }
            }
        };

        // Apply emphasis (increases pitch variation)
        let emphasis_boost = 1.0 + self.emphasis * 0.2 * (1.0 - pos);
        
        self.pitch_multiplier * contour_modifier * emphasis_boost
    }
}

/// Stress pattern for words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StressLevel {
    /// Unstressed syllable.
    Unstressed,
    /// Primary stress.
    #[default]
    Primary,
    /// Secondary stress.
    Secondary,
}

/// A prosodic phrase segment.
#[derive(Debug, Clone)]
pub struct PhraseSegment {
    /// The text content.
    pub text: String,
    /// Prosody configuration for this segment.
    pub prosody: ProsodyConfig,
    /// Start position in the full phrase (0.0 to 1.0).
    pub start_position: f32,
    /// End position in the full phrase (0.0 to 1.0).
    pub end_position: f32,
}

impl PhraseSegment {
    /// Creates a new phrase segment.
    #[must_use]
    pub fn new(text: String, start: f32, end: f32) -> Self {
        Self {
            text,
            prosody: ProsodyConfig::default(),
            start_position: start,
            end_position: end,
        }
    }

    /// Sets the prosody for this segment.
    #[must_use]
    pub fn with_prosody(mut self, prosody: ProsodyConfig) -> Self {
        self.prosody = prosody;
        self
    }
}

/// Phrase analyzer for prosody.
pub struct PhraseAnalyzer;

impl PhraseAnalyzer {
    /// Analyze text and split into prosodic segments.
    #[must_use]
    pub fn analyze(text: &str) -> Vec<PhraseSegment> {
        let sentences: Vec<&str> = Self::split_sentences(text);
        let total_len: usize = sentences.iter().map(|s| s.len()).sum();
        
        if total_len == 0 {
            return vec![];
        }

        let mut segments = Vec::new();
        let mut current_pos = 0usize;

        for sentence in sentences {
            let sentence_len = sentence.len();
            if sentence_len == 0 {
                continue;
            }

            let start = current_pos as f32 / total_len as f32;
            let end = (current_pos + sentence_len) as f32 / total_len as f32;
            
            let sentence_type = SentenceType::detect(sentence);
            let prosody = ProsodyConfig::from_sentence_type(sentence_type);
            
            segments.push(PhraseSegment::new(sentence.to_string(), start, end)
                .with_prosody(prosody));
            
            current_pos += sentence_len;
        }

        segments
    }

    /// Split text into sentences.
    fn split_sentences(text: &str) -> Vec<&str> {
        let mut sentences = Vec::new();
        let mut start = 0;
        let chars: Vec<char> = text.chars().collect();
        
        for (i, c) in chars.iter().enumerate() {
            if *c == '.' || *c == '!' || *c == '?' || *c == '¿' || *c == '¡' {
                // Check if it's not a decimal point or abbreviation
                let is_sentence_end = *c != '.' || 
                    i + 1 >= chars.len() || 
                    chars.get(i + 1).is_some_and(|next| next.is_whitespace() || next.is_uppercase());
                
                if is_sentence_end {
                    let byte_start = text.char_indices().nth(start).map(|(idx, _)| idx).unwrap_or(0);
                    let byte_end = text.char_indices().nth(i + 1).map(|(idx, _)| idx).unwrap_or(text.len());
                    let sentence = &text[byte_start..byte_end];
                    if !sentence.trim().is_empty() {
                        sentences.push(sentence.trim());
                    }
                    start = i + 1;
                }
            }
        }

        // Add remaining text as a sentence
        if start < chars.len() {
            let byte_start = text.char_indices().nth(start).map(|(idx, _)| idx).unwrap_or(0);
            let remaining = &text[byte_start..];
            if !remaining.trim().is_empty() {
                sentences.push(remaining.trim());
            }
        }

        if sentences.is_empty() && !text.trim().is_empty() {
            sentences.push(text.trim());
        }

        sentences
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_type_detection() {
        assert_eq!(SentenceType::detect("Hello."), SentenceType::Statement);
        assert_eq!(SentenceType::detect("Hello"), SentenceType::Statement);
        assert_eq!(SentenceType::detect("Are you there?"), SentenceType::Question);
        assert_eq!(SentenceType::detect("What time is it?"), SentenceType::WhQuestion);
        assert_eq!(SentenceType::detect("Wow!"), SentenceType::Exclamation);
    }

    #[test]
    fn test_spanish_sentence_detection() {
        assert_eq!(SentenceType::detect("¿Cómo estás?"), SentenceType::WhQuestion);
        assert_eq!(SentenceType::detect("¡Hola!"), SentenceType::Exclamation);
    }

    #[test]
    fn test_prosody_config_defaults() {
        let config = ProsodyConfig::default();
        assert!((config.pitch_multiplier - 1.0).abs() < f32::EPSILON);
        assert!((config.rate_multiplier - 1.0).abs() < f32::EPSILON);
        assert!((config.volume_multiplier - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pitch_contour_rising() {
        let config = ProsodyConfig::default().with_contour(PitchContour::Rising);
        
        let start_pitch = config.pitch_at_position(0.0);
        let end_pitch = config.pitch_at_position(1.0);
        
        assert!(end_pitch > start_pitch, "Rising contour should increase pitch");
    }

    #[test]
    fn test_pitch_contour_falling() {
        let config = ProsodyConfig::default().with_contour(PitchContour::Falling);
        
        let start_pitch = config.pitch_at_position(0.0);
        let end_pitch = config.pitch_at_position(1.0);
        
        assert!(start_pitch > end_pitch, "Falling contour should decrease pitch");
    }

    #[test]
    fn test_phrase_analyzer() {
        let segments = PhraseAnalyzer::analyze("Hello. How are you?");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].prosody.contour, PitchContour::Falling);
        // "How are you?" is a wh-question (starts with "how"), so it gets FallingRising
        assert_eq!(segments[1].prosody.contour, PitchContour::FallingRising);
    }

    #[test]
    fn test_phrase_analyzer_single_sentence() {
        let segments = PhraseAnalyzer::analyze("Hello world");
        assert_eq!(segments.len(), 1);
    }
}
