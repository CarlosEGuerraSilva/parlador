//! Formant synthesis engine for generating speech audio.
//!
//! This module implements a Klatt-style formant synthesizer that generates
//! speech audio from phoneme sequences with formant specifications.

use crate::error::Result;
use crate::phoneme::{FormantValues, Phoneme, PhonemeCategory, PhonemeInventory};
use std::f32::consts::PI;

/// Sample rate for audio generation (Hz).
pub const SAMPLE_RATE: u32 = 22050;

/// Audio output from speech synthesis.
#[derive(Debug, Clone)]
pub struct AudioOutput {
    /// Raw audio samples (16-bit signed PCM).
    pub samples: Vec<i16>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of audio channels (typically 1 for mono).
    pub channels: u16,
}

impl AudioOutput {
    /// Creates a new audio output.
    pub fn new(samples: Vec<i16>, sample_rate: u32, channels: u16) -> Self {
        Self {
            samples,
            sample_rate,
            channels,
        }
    }

    /// Returns the duration of the audio in seconds.
    pub fn duration_secs(&self) -> f64 {
        self.samples.len() as f64 / (self.sample_rate as f64 * self.channels as f64)
    }

    /// Returns true if the audio is empty.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

/// Configuration for the formant synthesizer.
#[derive(Debug, Clone)]
pub struct SynthesisConfig {
    /// Base pitch frequency (F0) in Hz.
    pub pitch_hz: f32,
    /// Speech rate multiplier (1.0 = normal).
    pub rate: f32,
    /// Volume (0.0 to 1.0).
    pub volume: f32,
    /// Sample rate in Hz.
    pub sample_rate: u32,
}

impl Default for SynthesisConfig {
    fn default() -> Self {
        Self {
            pitch_hz: 120.0,  // Default male voice pitch
            rate: 1.0,
            volume: 0.8,
            sample_rate: SAMPLE_RATE,
        }
    }
}

impl SynthesisConfig {
    /// Creates a configuration for a male voice.
    pub fn male() -> Self {
        Self {
            pitch_hz: 120.0,
            ..Default::default()
        }
    }

    /// Creates a configuration for a female voice.
    pub fn female() -> Self {
        Self {
            pitch_hz: 200.0,
            ..Default::default()
        }
    }
}

/// A resonator (second-order bandpass filter) for formant synthesis.
#[derive(Debug, Clone)]
struct Resonator {
    a: f32,
    b: f32,
    c: f32,
    y1: f32,
    y2: f32,
}

impl Resonator {
    /// Creates a new resonator with the given frequency and bandwidth.
    fn new(freq: f32, bandwidth: f32, sample_rate: f32) -> Self {
        let c = -f32::exp(-2.0 * PI * bandwidth / sample_rate);
        let b = 2.0 * f32::exp(-PI * bandwidth / sample_rate) * f32::cos(2.0 * PI * freq / sample_rate);
        let a = 1.0 - b - c;

        Self {
            a,
            b,
            c,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Process a sample through the resonator.
    fn process(&mut self, x: f32) -> f32 {
        let y = self.a * x + self.b * self.y1 + self.c * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    /// Reset the resonator state.
    #[allow(dead_code)]
    fn reset(&mut self) {
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Update the resonator parameters.
    fn set_params(&mut self, freq: f32, bandwidth: f32, sample_rate: f32) {
        self.c = -f32::exp(-2.0 * PI * bandwidth / sample_rate);
        self.b = 2.0 * f32::exp(-PI * bandwidth / sample_rate) * f32::cos(2.0 * PI * freq / sample_rate);
        self.a = 1.0 - self.b - self.c;
    }
}

/// Formant synthesizer engine.
pub struct FormantSynthesizer {
    /// Synthesis configuration.
    pub config: SynthesisConfig,
    /// Formant resonators (F1, F2, F3).
    formants: [Resonator; 3],
    /// Nasal resonator.
    nasal: Resonator,
    /// Current pitch phase.
    pitch_phase: f32,
    /// Noise generator state.
    noise_state: u32,
}

impl FormantSynthesizer {
    /// Creates a new formant synthesizer.
    pub fn new(config: SynthesisConfig) -> Self {
        let sr = config.sample_rate as f32;
        Self {
            config,
            formants: [
                Resonator::new(500.0, 60.0, sr),
                Resonator::new(1500.0, 90.0, sr),
                Resonator::new(2500.0, 150.0, sr),
            ],
            nasal: Resonator::new(300.0, 100.0, sr),
            pitch_phase: 0.0,
            noise_state: 12345,
        }
    }

    /// Reset the synthesizer state.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        for f in &mut self.formants {
            f.reset();
        }
        self.nasal.reset();
        self.pitch_phase = 0.0;
    }

    /// Generate white noise sample.
    fn noise(&mut self) -> f32 {
        // Simple linear congruential generator
        self.noise_state = self.noise_state.wrapping_mul(1103515245).wrapping_add(12345);
        let val = ((self.noise_state >> 16) & 0x7FFF) as f32 / 32767.0;
        val * 2.0 - 1.0
    }

    /// Generate a glottal pulse waveform.
    fn glottal_pulse(&mut self, f0: f32) -> f32 {
        let sample_rate = self.config.sample_rate as f32;
        let phase_inc = f0 / sample_rate;
        
        self.pitch_phase += phase_inc;
        if self.pitch_phase >= 1.0 {
            self.pitch_phase -= 1.0;
        }

        // Modified Liljencrants-Fant glottal pulse model (simplified)
        let t = self.pitch_phase;
        if t < 0.4 {
            // Opening phase
            let x = t / 0.4;
            3.0 * x * x - 2.0 * x * x * x
        } else if t < 0.6 {
            // Closing phase
            let x = (t - 0.4) / 0.2;
            1.0 - x * x
        } else {
            // Closed phase
            0.0
        }
    }

    /// Synthesize audio for a single phoneme.
    pub fn synthesize_phoneme(&mut self, phoneme: &Phoneme, duration_ms: u32) -> Vec<f32> {
        let sample_rate = self.config.sample_rate as f32;
        let duration_samples = ((duration_ms as f32 / 1000.0) * sample_rate / self.config.rate) as usize;
        let mut output = Vec::with_capacity(duration_samples);

        match phoneme.category {
            PhonemeCategory::Silence => {
                // Generate silence
                output.extend(std::iter::repeat_n(0.0, duration_samples));
            }
            PhonemeCategory::Vowel | PhonemeCategory::Diphthong => {
                if let Some(formants) = &phoneme.formants {
                    self.synthesize_vowel(formants, duration_samples, &mut output);
                }
            }
            PhonemeCategory::Nasal => {
                if let Some(formants) = &phoneme.formants {
                    self.synthesize_nasal(formants, duration_samples, &mut output);
                }
            }
            PhonemeCategory::Plosive => {
                self.synthesize_plosive(phoneme.voiced, duration_samples, &mut output);
            }
            PhonemeCategory::Fricative => {
                self.synthesize_fricative(phoneme.voiced, duration_samples, &mut output);
            }
            PhonemeCategory::Affricate => {
                self.synthesize_affricate(phoneme.voiced, duration_samples, &mut output);
            }
            PhonemeCategory::Lateral | PhonemeCategory::Rhotic | PhonemeCategory::Approximant => {
                if let Some(formants) = &phoneme.formants {
                    self.synthesize_approximant(formants, phoneme.voiced, duration_samples, &mut output);
                }
            }
        }

        output
    }

    /// Synthesize a vowel sound.
    fn synthesize_vowel(&mut self, formants: &FormantValues, samples: usize, output: &mut Vec<f32>) {
        let sr = self.config.sample_rate as f32;
        
        // Update formant resonators
        self.formants[0].set_params(formants.f1, formants.b1, sr);
        self.formants[1].set_params(formants.f2, formants.b2, sr);
        self.formants[2].set_params(formants.f3, formants.b3, sr);

        for i in 0..samples {
            // Apply amplitude envelope for natural sound
            let env = self.amplitude_envelope(i, samples);
            
            // Generate glottal source
            let source = self.glottal_pulse(self.config.pitch_hz);
            
            // Apply formant filtering (parallel configuration)
            let f1_out = self.formants[0].process(source);
            let f2_out = self.formants[1].process(source);
            let f3_out = self.formants[2].process(source);
            
            // Mix formants with decreasing weights
            let sample = (f1_out * 1.0 + f2_out * 0.5 + f3_out * 0.25) * env * self.config.volume;
            output.push(sample);
        }
    }

    /// Synthesize a nasal sound.
    fn synthesize_nasal(&mut self, formants: &FormantValues, samples: usize, output: &mut Vec<f32>) {
        let sr = self.config.sample_rate as f32;
        
        self.formants[0].set_params(formants.f1, formants.b1 * 1.5, sr);
        self.nasal.set_params(250.0, 100.0, sr);

        for i in 0..samples {
            let env = self.amplitude_envelope(i, samples);
            let source = self.glottal_pulse(self.config.pitch_hz);
            
            let formant_out = self.formants[0].process(source);
            let nasal_out = self.nasal.process(source);
            
            let sample = (formant_out * 0.3 + nasal_out * 0.7) * env * self.config.volume;
            output.push(sample);
        }
    }

    /// Synthesize a plosive sound.
    fn synthesize_plosive(&mut self, voiced: bool, samples: usize, output: &mut Vec<f32>) {
        // Closure phase (silence)
        let closure_samples = samples * 2 / 3;
        output.extend(std::iter::repeat_n(0.0, closure_samples));

        // Burst phase
        let burst_samples = samples - closure_samples;
        for i in 0..burst_samples {
            let env = (1.0 - i as f32 / burst_samples as f32).powi(2);
            let noise = self.noise();
            let voicing = if voiced {
                self.glottal_pulse(self.config.pitch_hz) * 0.3
            } else {
                0.0
            };
            
            let sample = (noise * 0.4 + voicing) * env * self.config.volume;
            output.push(sample);
        }
    }

    /// Synthesize a fricative sound.
    fn synthesize_fricative(&mut self, voiced: bool, samples: usize, output: &mut Vec<f32>) {
        for i in 0..samples {
            let env = self.amplitude_envelope(i, samples);
            let noise = self.noise();
            let voicing = if voiced {
                self.glottal_pulse(self.config.pitch_hz) * 0.4
            } else {
                0.0
            };
            
            let sample = (noise * 0.6 + voicing) * env * self.config.volume * 0.5;
            output.push(sample);
        }
    }

    /// Synthesize an affricate sound.
    fn synthesize_affricate(&mut self, voiced: bool, samples: usize, output: &mut Vec<f32>) {
        // Plosive portion
        let plosive_samples = samples / 3;
        self.synthesize_plosive(voiced, plosive_samples, output);
        
        // Fricative portion
        let fricative_samples = samples - plosive_samples;
        self.synthesize_fricative(voiced, fricative_samples, output);
    }

    /// Synthesize an approximant sound.
    fn synthesize_approximant(&mut self, formants: &FormantValues, voiced: bool, samples: usize, output: &mut Vec<f32>) {
        let sr = self.config.sample_rate as f32;
        
        self.formants[0].set_params(formants.f1, formants.b1 * 1.2, sr);
        self.formants[1].set_params(formants.f2, formants.b2 * 1.2, sr);

        for i in 0..samples {
            let env = self.amplitude_envelope(i, samples);
            
            let source = if voiced {
                self.glottal_pulse(self.config.pitch_hz)
            } else {
                self.noise() * 0.3
            };
            
            let f1_out = self.formants[0].process(source);
            let f2_out = self.formants[1].process(source);
            
            let sample = (f1_out * 0.7 + f2_out * 0.3) * env * self.config.volume * 0.7;
            output.push(sample);
        }
    }

    /// Generate an amplitude envelope for natural attack/decay.
    fn amplitude_envelope(&self, sample: usize, total: usize) -> f32 {
        let attack_len = (total as f32 * 0.1) as usize;
        let decay_len = (total as f32 * 0.15) as usize;
        
        if sample < attack_len {
            sample as f32 / attack_len as f32
        } else if sample > total - decay_len {
            (total - sample) as f32 / decay_len as f32
        } else {
            1.0
        }
    }

    /// Synthesize a sequence of phonemes to audio.
    pub fn synthesize_phonemes(&mut self, phoneme_str: &str, inventory: &PhonemeInventory) -> Result<Vec<f32>> {
        let mut output = Vec::new();
        
        for phoneme_sym in phoneme_str.split_whitespace() {
            if phoneme_sym == "_" {
                // Pause between words
                let pause_samples = (0.1 * self.config.sample_rate as f32 / self.config.rate) as usize;
                output.extend(std::iter::repeat_n(0.0, pause_samples));
                continue;
            }

            if let Some(phoneme) = inventory.get(phoneme_sym) {
                let duration = (phoneme.duration_ms as f32 / self.config.rate) as u32;
                let samples = self.synthesize_phoneme(phoneme, duration);
                output.extend(samples);
            }
        }

        Ok(output)
    }

    /// Convert float samples to 16-bit PCM.
    pub fn to_pcm16(&self, samples: &[f32]) -> Vec<i16> {
        samples
            .iter()
            .map(|&s| {
                let clamped = s.clamp(-1.0, 1.0);
                (clamped * 32767.0) as i16
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_output_duration() {
        let audio = AudioOutput::new(vec![0i16; 22050], 22050, 1);
        assert!((audio.duration_secs() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_synthesizer_creation() {
        let config = SynthesisConfig::default();
        let synth = FormantSynthesizer::new(config);
        assert_eq!(synth.config.sample_rate, SAMPLE_RATE);
    }

    #[test]
    fn test_vowel_synthesis() {
        let config = SynthesisConfig::default();
        let mut synth = FormantSynthesizer::new(config);
        
        let inventory = PhonemeInventory::english();
        let vowel = inventory.get("i").unwrap();
        
        let samples = synth.synthesize_phoneme(vowel, 100);
        assert!(!samples.is_empty());
    }

    #[test]
    fn test_noise_generation() {
        let config = SynthesisConfig::default();
        let mut synth = FormantSynthesizer::new(config);
        
        let noise1 = synth.noise();
        let noise2 = synth.noise();
        
        assert!(noise1 >= -1.0 && noise1 <= 1.0);
        assert!(noise2 >= -1.0 && noise2 <= 1.0);
        assert_ne!(noise1, noise2);
    }
}
