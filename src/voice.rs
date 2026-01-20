//! Language and voice configuration for the speech synthesizer.

use std::fmt;

/// Supported languages for speech synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum Language {
    /// English language.
    #[default]
    English,
    /// Spanish language.
    Spanish,
}

impl Language {
    /// Returns the language code.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
        }
    }

    /// Returns the full language name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Spanish",
        }
    }

    /// Creates a Language from a language code string.
    ///
    /// Accepts common language codes like "en", "eng", "english", "es", "spa", "spanish".
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "en" | "eng" | "english" | "en-us" | "en-gb" => Some(Language::English),
            "es" | "spa" | "spanish" | "es-es" | "es-mx" => Some(Language::Spanish),
            _ => None,
        }
    }
}


impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Voice variant configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum VoiceVariant {
    /// Default voice.
    #[default]
    Default,
    /// Male voice variant 1.
    Male1,
    /// Male voice variant 2.
    Male2,
    /// Male voice variant 3.
    Male3,
    /// Female voice variant 1.
    Female1,
    /// Female voice variant 2.
    Female2,
    /// Female voice variant 3.
    Female3,
}

impl VoiceVariant {
    /// Returns the base pitch frequency for this variant.
    #[must_use]
    pub fn base_pitch_hz(&self) -> f32 {
        match self {
            VoiceVariant::Default => 130.0,
            VoiceVariant::Male1 => 100.0,
            VoiceVariant::Male2 => 120.0,
            VoiceVariant::Male3 => 140.0,
            VoiceVariant::Female1 => 180.0,
            VoiceVariant::Female2 => 200.0,
            VoiceVariant::Female3 => 220.0,
        }
    }

    /// Returns a human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            VoiceVariant::Default => "Default",
            VoiceVariant::Male1 => "Male 1",
            VoiceVariant::Male2 => "Male 2",
            VoiceVariant::Male3 => "Male 3",
            VoiceVariant::Female1 => "Female 1",
            VoiceVariant::Female2 => "Female 2",
            VoiceVariant::Female3 => "Female 3",
        }
    }
}


/// Configuration for a voice used in speech synthesis.
#[derive(Debug, Clone, PartialEq)]
pub struct VoiceConfig {
    /// The language for the voice.
    pub language: Language,
    /// The voice variant.
    pub variant: VoiceVariant,
    /// Speech rate (words per minute). Default is 175.
    pub rate: u32,
    /// Pitch adjustment (-100 to 100, 0 = default).
    pub pitch: i8,
    /// Volume (0-200, with 100 being normal). Default is 100.
    pub volume: u8,
}

impl VoiceConfig {
    /// Creates a new voice configuration with default settings.
    #[must_use]
    pub fn new(language: Language) -> Self {
        Self {
            language,
            variant: VoiceVariant::Default,
            rate: 175,
            pitch: 0,
            volume: 100,
        }
    }

    /// Sets the voice variant.
    #[must_use]
    pub fn with_variant(mut self, variant: VoiceVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the speech rate in words per minute.
    #[must_use]
    pub fn with_rate(mut self, rate: u32) -> Self {
        self.rate = rate.clamp(50, 500);
        self
    }

    /// Sets the pitch adjustment (-100 to 100).
    #[must_use]
    pub fn with_pitch(mut self, pitch: i8) -> Self {
        self.pitch = pitch.clamp(-100, 100);
        self
    }

    /// Sets the volume (0-200).
    #[must_use]
    pub fn with_volume(mut self, volume: u8) -> Self {
        self.volume = volume.min(200);
        self
    }

    /// Calculates the effective pitch frequency.
    pub fn effective_pitch_hz(&self) -> f32 {
        let base = self.variant.base_pitch_hz();
        let adjustment = 1.0 + (self.pitch as f32 / 100.0) * 0.5;
        base * adjustment
    }

    /// Calculates the rate multiplier.
    pub fn rate_multiplier(&self) -> f32 {
        self.rate as f32 / 175.0
    }

    /// Calculates the volume level (0.0 to 1.0).
    pub fn volume_level(&self) -> f32 {
        self.volume as f32 / 100.0
    }
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self::new(Language::English)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_codes() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Spanish.code(), "es");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::from_code("english"), Some(Language::English));
        assert_eq!(Language::from_code("es"), Some(Language::Spanish));
        assert_eq!(Language::from_code("spanish"), Some(Language::Spanish));
        assert_eq!(Language::from_code("fr"), None);
    }

    #[test]
    fn test_voice_config_builder() {
        let config = VoiceConfig::new(Language::Spanish)
            .with_variant(VoiceVariant::Female1)
            .with_rate(200)
            .with_pitch(20)
            .with_volume(150);

        assert_eq!(config.language, Language::Spanish);
        assert_eq!(config.variant, VoiceVariant::Female1);
        assert_eq!(config.rate, 200);
        assert_eq!(config.pitch, 20);
        assert_eq!(config.volume, 150);
    }

    #[test]
    fn test_effective_pitch() {
        let config = VoiceConfig::new(Language::English)
            .with_variant(VoiceVariant::Female1)
            .with_pitch(50);
        
        let base = VoiceVariant::Female1.base_pitch_hz();
        let expected = base * 1.25; // 50% adjustment = 1.25x
        assert!((config.effective_pitch_hz() - expected).abs() < 0.1);
    }

    #[test]
    fn test_rate_clamping() {
        let config = VoiceConfig::new(Language::English).with_rate(1000);
        assert_eq!(config.rate, 500);

        let config = VoiceConfig::new(Language::English).with_rate(10);
        assert_eq!(config.rate, 50);
    }
}
