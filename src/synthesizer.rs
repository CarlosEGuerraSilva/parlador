//! Main speech synthesizer implementation.
//!
//! This module provides the main `Synthesizer` struct that combines
//! text-to-phoneme conversion and formant synthesis to generate speech.

use crate::error::{Result, SynthesizerError};
use crate::formant::{AudioOutput, FormantSynthesizer, SynthesisConfig, SAMPLE_RATE};
use crate::g2p::{text_to_ipa, G2PConverter};
use crate::phoneme::PhonemeInventory;
use crate::voice::{Language, VoiceConfig};

/// Phoneme output format for TTS model compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum PhonemeFormat {
    /// International Phonetic Alphabet (IPA).
    #[default]
    Ipa,
    /// ASCII phoneme representation (internal format).
    Ascii,
}


/// Result of phoneme generation.
#[derive(Debug, Clone)]
pub struct PhonemeResult {
    /// The original input text.
    pub text: String,
    /// Generated phonemes.
    pub phonemes: String,
    /// The format of the phonemes.
    pub format: PhonemeFormat,
    /// The language used for phoneme generation.
    pub language: Language,
}

/// The main speech synthesizer.
///
/// This synthesizer uses formant synthesis to generate speech audio
/// from text, supporting English and Spanish languages.
///
/// # Example
///
/// ```
/// use parlador::{Synthesizer, VoiceConfig, Language};
///
/// // Create a synthesizer with default English voice
/// let synth = Synthesizer::new()?;
///
/// // Synthesize speech and get audio data
/// let audio = synth.synthesize("Hello, world!")?;
/// println!("Generated {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
/// # Ok::<(), parlador::SynthesizerError>(())
/// ```
pub struct Synthesizer {
    config: VoiceConfig,
    g2p_en: G2PConverter,
    g2p_es: G2PConverter,
    inventory_en: PhonemeInventory,
    inventory_es: PhonemeInventory,
}

impl Synthesizer {
    /// Creates a new synthesizer with default settings.
    ///
    /// Uses English as the default language with standard voice parameters.
    pub fn new() -> Result<Self> {
        Self::with_config(VoiceConfig::default())
    }

    /// Creates a new synthesizer with the specified voice configuration.
    pub fn with_config(config: VoiceConfig) -> Result<Self> {
        Ok(Self {
            config,
            g2p_en: G2PConverter::english(),
            g2p_es: G2PConverter::spanish(),
            inventory_en: PhonemeInventory::english(),
            inventory_es: PhonemeInventory::spanish(),
        })
    }

    /// Returns the current voice configuration.
    #[must_use]
    pub fn config(&self) -> &VoiceConfig {
        &self.config
    }

    /// Sets a new voice configuration.
    pub fn set_config(&mut self, config: VoiceConfig) {
        self.config = config;
    }

    /// Sets the language.
    pub fn set_language(&mut self, language: Language) {
        self.config.language = language;
    }

    /// Sets the speech rate in words per minute.
    pub fn set_rate(&mut self, rate: u32) {
        self.config.rate = rate.clamp(50, 500);
    }

    /// Sets the pitch adjustment (-100 to 100).
    pub fn set_pitch(&mut self, pitch: i8) {
        self.config.pitch = pitch.clamp(-100, 100);
    }

    /// Sets the volume (0-200).
    pub fn set_volume(&mut self, volume: u8) {
        self.config.volume = volume.min(200);
    }

    /// Gets the appropriate G2P converter for the current language.
    fn get_g2p(&self) -> &G2PConverter {
        match self.config.language {
            Language::English => &self.g2p_en,
            Language::Spanish => &self.g2p_es,
        }
    }

    /// Gets the appropriate phoneme inventory for the current language.
    fn get_inventory(&self) -> &PhonemeInventory {
        match self.config.language {
            Language::English => &self.inventory_en,
            Language::Spanish => &self.inventory_es,
        }
    }

    /// Creates a formant synthesizer with current configuration.
    fn create_formant_synthesizer(&self) -> FormantSynthesizer {
        let synth_config = SynthesisConfig {
            pitch_hz: self.config.effective_pitch_hz(),
            rate: self.config.rate_multiplier(),
            volume: self.config.volume_level().min(1.0),
            sample_rate: SAMPLE_RATE,
        };
        FormantSynthesizer::new(synth_config)
    }

    /// Synthesizes speech from text and returns the audio data.
    ///
    /// The returned audio is 16-bit signed PCM at 22050 Hz, mono.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to synthesize.
    ///
    /// # Returns
    ///
    /// Audio data containing the synthesized speech.
    pub fn synthesize(&self, text: &str) -> Result<AudioOutput> {
        // Convert text to phonemes
        let g2p = self.get_g2p();
        let phonemes = g2p.convert(text)?;

        if phonemes.is_empty() {
            return Ok(AudioOutput::new(vec![], SAMPLE_RATE, 1));
        }

        // Synthesize phonemes to audio
        let inventory = self.get_inventory();
        let mut formant_synth = self.create_formant_synthesizer();
        let float_samples = formant_synth.synthesize_phonemes(&phonemes, inventory)?;
        
        // Convert to PCM16
        let pcm_samples = formant_synth.to_pcm16(&float_samples);

        Ok(AudioOutput::new(pcm_samples, SAMPLE_RATE, 1))
    }

    /// Converts text to phonemes without synthesizing audio.
    ///
    /// This is useful for integration with external TTS models like Kokoro
    /// that require phoneme input.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to convert.
    /// * `format` - The desired phoneme output format.
    ///
    /// # Returns
    ///
    /// A `PhonemeResult` containing the phonemes.
    pub fn text_to_phonemes(&self, text: &str, format: PhonemeFormat) -> Result<PhonemeResult> {
        let phonemes = match format {
            PhonemeFormat::Ascii => {
                let g2p = self.get_g2p();
                g2p.convert(text)?
            }
            PhonemeFormat::Ipa => {
                text_to_ipa(text, self.config.language.code())?
            }
        };

        Ok(PhonemeResult {
            text: text.to_string(),
            phonemes,
            format,
            language: self.config.language,
        })
    }

    /// Gets the sample rate used for audio output.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    /// Gets the supported languages.
    #[must_use]
    pub fn supported_languages() -> &'static [Language] {
        &[Language::English, Language::Spanish]
    }
}

impl Default for Synthesizer {
    /// Creates a default synthesizer with English language.
    ///
    /// # Panics
    ///
    /// This implementation should never panic as the synthesizer
    /// initialization cannot fail with default parameters.
    fn default() -> Self {
        Self::new().expect("Default synthesizer initialization cannot fail")
    }
}

// ============================================================================
// espeak-ng compatible API
// ============================================================================
// These functions provide an API similar to espeak-ng for easy integration
// with projects that previously used espeak-ng.

/// Audio output type (compatible with espeak-ng).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AudioOutputType {
    /// Playback through audio device (not implemented - returns buffer).
    Playback = 0,
    /// Return audio in buffer.
    Retrieval = 1,
    /// Synchronous playback (not implemented - returns buffer).
    SynchronousPlayback = 2,
}

/// Initialize the synthesizer (espeak-ng compatible).
///
/// # Arguments
///
/// * `output` - Audio output type.
/// * `_buffer_length` - Buffer length (ignored, for compatibility).
/// * `_path` - Data path (ignored, for compatibility).
/// * `_options` - Options (ignored, for compatibility).
///
/// # Returns
///
/// Sample rate on success.
pub fn espeak_initialize(
    _output: AudioOutputType,
    _buffer_length: i32,
    _path: Option<&str>,
    _options: i32,
) -> Result<i32> {
    Ok(SAMPLE_RATE as i32)
}

/// Set the voice by name (espeak-ng compatible).
///
/// # Arguments
///
/// * `name` - Voice name (language code like "en" or "es").
pub fn espeak_set_voice_by_name(name: &str) -> Result<()> {
    if Language::from_code(name).is_some() {
        Ok(())
    } else {
        Err(SynthesizerError::UnsupportedLanguage(name.to_string()))
    }
}

/// Synthesize text to audio (espeak-ng compatible).
///
/// # Arguments
///
/// * `text` - Text to synthesize.
/// * `language` - Language code.
///
/// # Returns
///
/// Audio samples as i16 PCM.
pub fn espeak_synth(text: &str, language: &str) -> Result<Vec<i16>> {
    let lang = Language::from_code(language)
        .ok_or_else(|| SynthesizerError::UnsupportedLanguage(language.to_string()))?;
    
    let config = VoiceConfig::new(lang);
    let synth = Synthesizer::with_config(config)?;
    let audio = synth.synthesize(text)?;
    
    Ok(audio.samples)
}

/// Convert text to phonemes (espeak-ng compatible).
///
/// # Arguments
///
/// * `text` - Text to convert.
/// * `language` - Language code.
/// * `ipa` - If true, return IPA format; otherwise ASCII.
///
/// # Returns
///
/// Phoneme string.
pub fn espeak_text_to_phonemes(text: &str, language: &str, ipa: bool) -> Result<String> {
    let lang = Language::from_code(language)
        .ok_or_else(|| SynthesizerError::UnsupportedLanguage(language.to_string()))?;
    
    let format = if ipa {
        PhonemeFormat::Ipa
    } else {
        PhonemeFormat::Ascii
    };
    
    let config = VoiceConfig::new(lang);
    let synth = Synthesizer::with_config(config)?;
    let result = synth.text_to_phonemes(text, format)?;
    
    Ok(result.phonemes)
}

/// Terminate the synthesizer (espeak-ng compatible).
/// This is a no-op since we don't have global state.
pub fn espeak_terminate() {
    // No-op - no global state to clean up
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesizer_creation() {
        let synth = Synthesizer::new();
        assert!(synth.is_ok());
    }

    #[test]
    fn test_synthesize_english() {
        let synth = Synthesizer::new().unwrap();
        let audio = synth.synthesize("hello");
        assert!(audio.is_ok());
        assert!(!audio.unwrap().is_empty());
    }

    #[test]
    fn test_synthesize_spanish() {
        let config = VoiceConfig::new(Language::Spanish);
        let synth = Synthesizer::with_config(config).unwrap();
        let audio = synth.synthesize("hola");
        assert!(audio.is_ok());
        assert!(!audio.unwrap().is_empty());
    }

    #[test]
    fn test_text_to_phonemes_ipa() {
        let synth = Synthesizer::new().unwrap();
        let result = synth.text_to_phonemes("hello", PhonemeFormat::Ipa);
        assert!(result.is_ok());
        assert!(!result.unwrap().phonemes.is_empty());
    }

    #[test]
    fn test_text_to_phonemes_ascii() {
        let synth = Synthesizer::new().unwrap();
        let result = synth.text_to_phonemes("hello", PhonemeFormat::Ascii);
        assert!(result.is_ok());
        assert!(!result.unwrap().phonemes.is_empty());
    }

    #[test]
    fn test_espeak_compatible_api() {
        let result = espeak_initialize(AudioOutputType::Retrieval, 500, None, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SAMPLE_RATE as i32);

        let result = espeak_set_voice_by_name("en");
        assert!(result.is_ok());

        let result = espeak_synth("hello", "en");
        assert!(result.is_ok());

        let result = espeak_text_to_phonemes("hello", "en", true);
        assert!(result.is_ok());

        espeak_terminate();
    }
}
