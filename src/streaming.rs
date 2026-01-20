//! Real-time audio streaming for speech synthesis.
//!
//! This module provides streaming synthesis capabilities, allowing
//! audio to be consumed incrementally as it is generated.

use crate::error::Result;
use crate::formant::{AudioOutput, FormantSynthesizer, SynthesisConfig, SAMPLE_RATE};
use crate::g2p::G2PConverter;
use crate::phoneme::PhonemeInventory;
use crate::prosody::{PhraseAnalyzer, ProsodyConfig};
use crate::voice::{Language, VoiceConfig};

/// Default chunk size in samples (about 50ms at 22050 Hz).
pub const DEFAULT_CHUNK_SIZE: usize = 1024;

/// A chunk of audio data from streaming synthesis.
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// The audio samples (16-bit PCM).
    pub samples: Vec<i16>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Whether this is the final chunk.
    pub is_final: bool,
    /// Progress through synthesis (0.0 to 1.0).
    pub progress: f32,
}

impl AudioChunk {
    /// Creates a new audio chunk.
    #[must_use]
    pub fn new(samples: Vec<i16>, is_final: bool, progress: f32) -> Self {
        Self {
            samples,
            sample_rate: SAMPLE_RATE,
            is_final,
            progress,
        }
    }

    /// Returns the duration of this chunk in seconds.
    #[must_use]
    pub fn duration_secs(&self) -> f64 {
        self.samples.len() as f64 / self.sample_rate as f64
    }

    /// Returns true if the chunk is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

/// Streaming synthesis configuration.
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Chunk size in samples.
    pub chunk_size: usize,
    /// Voice configuration.
    pub voice: VoiceConfig,
    /// Enable prosody analysis.
    pub enable_prosody: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            voice: VoiceConfig::default(),
            enable_prosody: true,
        }
    }
}

impl StreamingConfig {
    /// Creates a new streaming configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the chunk size.
    #[must_use]
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size.max(256); // Minimum 256 samples
        self
    }

    /// Sets the voice configuration.
    #[must_use]
    pub fn with_voice(mut self, voice: VoiceConfig) -> Self {
        self.voice = voice;
        self
    }

    /// Sets whether to enable prosody analysis.
    #[must_use]
    pub fn with_prosody(mut self, enable: bool) -> Self {
        self.enable_prosody = enable;
        self
    }
}

/// A streaming speech synthesizer that generates audio incrementally.
pub struct StreamingSynthesizer {
    config: StreamingConfig,
    g2p_en: G2PConverter,
    g2p_es: G2PConverter,
    inventory_en: PhonemeInventory,
    inventory_es: PhonemeInventory,
}

impl StreamingSynthesizer {
    /// Creates a new streaming synthesizer with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(StreamingConfig::default())
    }

    /// Creates a new streaming synthesizer with the specified configuration.
    #[must_use]
    pub fn with_config(config: StreamingConfig) -> Self {
        Self {
            config,
            g2p_en: G2PConverter::english(),
            g2p_es: G2PConverter::spanish(),
            inventory_en: PhonemeInventory::english(),
            inventory_es: PhonemeInventory::spanish(),
        }
    }

    /// Returns the current configuration.
    #[must_use]
    pub fn config(&self) -> &StreamingConfig {
        &self.config
    }

    /// Gets the G2P converter for the configured language.
    fn get_g2p(&self) -> &G2PConverter {
        match self.config.voice.language {
            Language::English => &self.g2p_en,
            Language::Spanish => &self.g2p_es,
        }
    }

    /// Gets the phoneme inventory for the configured language.
    fn get_inventory(&self) -> &PhonemeInventory {
        match self.config.voice.language {
            Language::English => &self.inventory_en,
            Language::Spanish => &self.inventory_es,
        }
    }

    /// Creates an iterator that yields audio chunks.
    pub fn synthesize_stream(&self, text: &str) -> Result<AudioStream> {
        let g2p = self.get_g2p();
        let phonemes = g2p.convert(text)?;
        let inventory = self.get_inventory().clone();

        // Analyze prosody if enabled
        let prosody = if self.config.enable_prosody {
            let segments = PhraseAnalyzer::analyze(text);
            if segments.is_empty() {
                ProsodyConfig::default()
            } else {
                segments[0].prosody.clone()
            }
        } else {
            ProsodyConfig::default()
        };

        // Create formant synthesizer
        let synth_config = SynthesisConfig {
            pitch_hz: self.config.voice.effective_pitch_hz() * prosody.pitch_multiplier,
            rate: self.config.voice.rate_multiplier() * prosody.rate_multiplier,
            volume: self.config.voice.volume_level().min(1.0) * prosody.volume_multiplier,
            sample_rate: SAMPLE_RATE,
        };

        Ok(AudioStream::new(
            phonemes,
            inventory,
            synth_config,
            self.config.chunk_size,
            prosody,
        ))
    }

    /// Synthesizes text and calls the callback for each chunk.
    pub fn synthesize_with_callback<F>(&self, text: &str, mut callback: F) -> Result<()>
    where
        F: FnMut(AudioChunk) -> bool,
    {
        let stream = self.synthesize_stream(text)?;
        
        for chunk in stream {
            if !callback(chunk) {
                break;
            }
        }
        
        Ok(())
    }

    /// Synthesizes text and collects all chunks into a single `AudioOutput`.
    pub fn synthesize_complete(&self, text: &str) -> Result<AudioOutput> {
        let stream = self.synthesize_stream(text)?;
        let mut all_samples = Vec::new();
        
        for chunk in stream {
            all_samples.extend(chunk.samples);
        }
        
        Ok(AudioOutput::new(all_samples, SAMPLE_RATE, 1))
    }
}

impl Default for StreamingSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

/// An iterator that yields audio chunks from streaming synthesis.
pub struct AudioStream {
    /// Phoneme tokens to synthesize.
    phoneme_tokens: Vec<String>,
    /// Current phoneme index.
    current_index: usize,
    /// Phoneme inventory.
    inventory: PhonemeInventory,
    /// Formant synthesizer.
    formant_synth: FormantSynthesizer,
    /// Chunk size.
    chunk_size: usize,
    /// Prosody configuration.
    prosody: ProsodyConfig,
    /// Buffer of pending samples.
    buffer: Vec<f32>,
    /// Whether synthesis is complete.
    is_complete: bool,
}

impl AudioStream {
    /// Creates a new audio stream.
    fn new(
        phonemes: String,
        inventory: PhonemeInventory,
        synth_config: SynthesisConfig,
        chunk_size: usize,
        prosody: ProsodyConfig,
    ) -> Self {
        let phoneme_tokens: Vec<String> = phonemes
            .split_whitespace()
            .map(String::from)
            .collect();

        Self {
            phoneme_tokens,
            current_index: 0,
            inventory,
            formant_synth: FormantSynthesizer::new(synth_config),
            chunk_size,
            prosody,
            buffer: Vec::new(),
            is_complete: false,
        }
    }

    /// Returns the total number of phonemes.
    #[must_use]
    pub fn total_phonemes(&self) -> usize {
        self.phoneme_tokens.len()
    }

    /// Returns the current progress (0.0 to 1.0).
    #[must_use]
    pub fn progress(&self) -> f32 {
        if self.phoneme_tokens.is_empty() {
            1.0
        } else {
            self.current_index as f32 / self.phoneme_tokens.len() as f32
        }
    }

    /// Synthesize the next phoneme into the buffer.
    fn synthesize_next_phoneme(&mut self) {
        if self.current_index >= self.phoneme_tokens.len() {
            self.is_complete = true;
            return;
        }

        let phoneme_sym = &self.phoneme_tokens[self.current_index];
        
        if phoneme_sym == "_" {
            // Pause between words
            let pause_samples = (0.1 * SAMPLE_RATE as f32 / self.formant_synth.config().rate) as usize;
            self.buffer.extend(std::iter::repeat_n(0.0, pause_samples));
        } else if let Some(phoneme) = self.inventory.get(phoneme_sym) {
            // Calculate position for prosody
            let position = self.progress();
            let pitch_mod = self.prosody.pitch_at_position(position);
            
            // Temporarily adjust pitch for this phoneme
            let duration = (phoneme.duration_ms as f32 / self.formant_synth.config().rate) as u32;
            
            // Synthesize with prosody-adjusted parameters
            let samples = self.formant_synth.synthesize_phoneme_with_pitch_mod(
                phoneme,
                duration,
                pitch_mod,
            );
            self.buffer.extend(samples);
        }

        self.current_index += 1;
    }

    /// Extract a chunk from the buffer.
    fn extract_chunk(&mut self, is_final: bool) -> AudioChunk {
        let samples_to_take = self.chunk_size.min(self.buffer.len());
        let float_samples: Vec<f32> = self.buffer.drain(..samples_to_take).collect();
        
        // Convert to PCM16
        let pcm_samples: Vec<i16> = float_samples
            .iter()
            .map(|&s| {
                let clamped = s.clamp(-1.0, 1.0);
                (clamped * 32767.0) as i16
            })
            .collect();

        AudioChunk::new(pcm_samples, is_final && self.buffer.is_empty(), self.progress())
    }
}

impl Iterator for AudioStream {
    type Item = AudioChunk;

    fn next(&mut self) -> Option<Self::Item> {
        // Fill buffer until we have enough samples or synthesis is complete
        while self.buffer.len() < self.chunk_size && !self.is_complete {
            self.synthesize_next_phoneme();
        }

        if self.buffer.is_empty() {
            return None;
        }

        let is_final = self.is_complete;
        Some(self.extract_chunk(is_final))
    }
}

// Extension trait for FormantSynthesizer to support pitch modification
impl FormantSynthesizer {
    /// Returns a reference to the configuration.
    pub fn config(&self) -> &SynthesisConfig {
        &self.config
    }

    /// Synthesize a phoneme with a pitch modification factor.
    pub fn synthesize_phoneme_with_pitch_mod(
        &mut self,
        phoneme: &crate::phoneme::Phoneme,
        duration_ms: u32,
        pitch_mod: f32,
    ) -> Vec<f32> {
        // Store original pitch
        let original_pitch = self.config.pitch_hz;
        
        // Apply pitch modification
        self.config.pitch_hz = original_pitch * pitch_mod;
        
        // Synthesize
        let samples = self.synthesize_phoneme(phoneme, duration_ms);
        
        // Restore pitch
        self.config.pitch_hz = original_pitch;
        
        samples
    }
}

/// Callback type for streaming synthesis.
#[allow(dead_code)]
pub type StreamCallback = Box<dyn FnMut(AudioChunk) -> bool + Send>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_synthesizer_creation() {
        let synth = StreamingSynthesizer::new();
        assert_eq!(synth.config().chunk_size, DEFAULT_CHUNK_SIZE);
    }

    #[test]
    fn test_audio_chunk_creation() {
        let chunk = AudioChunk::new(vec![0i16; 1024], false, 0.5);
        assert!(!chunk.is_final);
        assert!((chunk.progress - 0.5).abs() < f32::EPSILON);
        assert!(!chunk.is_empty());
    }

    #[test]
    fn test_streaming_synthesis() {
        let synth = StreamingSynthesizer::new();
        let stream = synth.synthesize_stream("hello");
        assert!(stream.is_ok());
        
        let chunks: Vec<AudioChunk> = stream.unwrap().collect();
        assert!(!chunks.is_empty());
        
        // Last chunk should be marked as final
        assert!(chunks.last().unwrap().is_final);
    }

    #[test]
    fn test_callback_synthesis() {
        let synth = StreamingSynthesizer::new();
        let mut chunk_count = 0;
        
        let result = synth.synthesize_with_callback("hello", |_chunk| {
            chunk_count += 1;
            true // Continue processing
        });
        
        assert!(result.is_ok());
        assert!(chunk_count > 0);
    }

    #[test]
    fn test_complete_synthesis() {
        let synth = StreamingSynthesizer::new();
        let audio = synth.synthesize_complete("hello");
        assert!(audio.is_ok());
        assert!(!audio.unwrap().is_empty());
    }

    #[test]
    fn test_streaming_config_builder() {
        let config = StreamingConfig::new()
            .with_chunk_size(2048)
            .with_prosody(false);
        
        assert_eq!(config.chunk_size, 2048);
        assert!(!config.enable_prosody);
    }

    #[test]
    fn test_stream_progress() {
        let synth = StreamingSynthesizer::new();
        let stream = synth.synthesize_stream("hello world").unwrap();
        
        let mut last_progress = 0.0;
        for chunk in stream {
            assert!(chunk.progress >= last_progress || chunk.progress == 0.0);
            last_progress = chunk.progress;
        }
    }
}
