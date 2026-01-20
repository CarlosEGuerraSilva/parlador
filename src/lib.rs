//! # Parlador
//!
//! Rust-powered multiplatform speech synthesizer engine with formant synthesis.
//!
//! Parlador is a custom speech synthesis library that uses formant synthesis
//! to generate speech audio from text. It provides a high-level API for
//! text-to-speech synthesis with support for English and Spanish, and is
//! designed to be compatible with external TTS models like Kokoro through
//! phoneme generation.
//!
//! ## Features
//!
//! - **Cross-platform**: Pure Rust implementation, works on Linux, macOS, and Windows
//! - **No external dependencies**: Self-contained formant synthesis engine
//! - **Multiple languages**: Support for English and Spanish
//! - **Voice customization**: Adjust rate, pitch, volume, and voice variants
//! - **Phoneme generation**: Compatible with TTS models like Kokoro
//! - **Audio synthesis**: Get raw audio data (16-bit PCM) for further processing
//! - **espeak-ng compatible API**: Easy migration from espeak-ng
//! - **SSML support**: Parse and synthesize SSML documents
//! - **Real-time streaming**: Generate audio incrementally for low-latency applications
//! - **Improved prosody**: Natural intonation patterns for different sentence types
//!
//! ## Quick Start
//!
//! ```
//! use parlador::{Synthesizer, VoiceConfig, Language};
//!
//! // Create a synthesizer with default English voice
//! let synth = Synthesizer::new()?;
//!
//! // Synthesize speech and get audio data
//! let audio = synth.synthesize("Hello, world!")?;
//! println!("Generated {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
//!
//! // Create a synthesizer with Spanish voice
//! let spanish_synth = Synthesizer::with_config(
//!     VoiceConfig::new(Language::Spanish)
//!         .with_rate(150)
//!         .with_pitch(20)
//! )?;
//! let audio = spanish_synth.synthesize("¡Hola, mundo!")?;
//! # Ok::<(), parlador::SynthesizerError>(())
//! ```
//!
//! ## Phoneme Generation for TTS Models
//!
//! Parlador can generate phonemes for use with external TTS models:
//!
//! ```
//! use parlador::{Synthesizer, PhonemeFormat};
//!
//! let synth = Synthesizer::new()?;
//! let result = synth.text_to_phonemes("Hello, world!", PhonemeFormat::Ipa)?;
//! println!("Phonemes: {}", result.phonemes);
//! # Ok::<(), parlador::SynthesizerError>(())
//! ```
//!
//! ## SSML Support
//!
//! Synthesize speech from SSML documents for fine-grained control:
//!
//! ```
//! use parlador::Synthesizer;
//!
//! let synth = Synthesizer::new()?;
//! let ssml = r#"<speak>
//!     Hello <break time="500ms"/> world!
//!     <prosody rate="fast" pitch="high">This is fast and high.</prosody>
//! </speak>"#;
//! let audio = synth.synthesize_ssml(ssml)?;
//! # Ok::<(), parlador::SynthesizerError>(())
//! ```
//!
//! ## Real-time Streaming
//!
//! Generate audio incrementally for low-latency applications:
//!
//! ```
//! use parlador::StreamingSynthesizer;
//!
//! let synth = StreamingSynthesizer::new();
//! for chunk in synth.synthesize_stream("Hello, world!")? {
//!     // Process each audio chunk as it's generated
//!     println!("Got {} samples, progress: {:.1}%", 
//!              chunk.samples.len(), 
//!              chunk.progress * 100.0);
//! }
//! # Ok::<(), parlador::SynthesizerError>(())
//! ```
//!
//! ## espeak-ng Compatible API
//!
//! For projects migrating from espeak-ng, a compatible API is provided:
//!
//! ```
//! use parlador::{espeak_initialize, espeak_synth, espeak_text_to_phonemes, AudioOutputType};
//!
//! // Initialize (returns sample rate)
//! let sample_rate = espeak_initialize(AudioOutputType::Retrieval, 500, None, 0)?;
//!
//! // Synthesize speech
//! let audio = espeak_synth("Hello", "en")?;
//!
//! // Get phonemes
//! let phonemes = espeak_text_to_phonemes("Hello", "en", true)?;
//! # Ok::<(), parlador::SynthesizerError>(())
//! ```

mod error;
mod formant;
mod g2p;
mod phoneme;
mod prosody;
mod ssml;
mod streaming;
mod synthesizer;
mod voice;

pub use error::{Result, SynthesizerError};
pub use formant::{AudioOutput, SynthesisConfig, SAMPLE_RATE};
pub use g2p::{text_to_ipa, G2PConverter};
pub use phoneme::{FormantValues, Phoneme, PhonemeCategory, PhonemeInventory};
pub use prosody::{
    PhraseAnalyzer, PhraseSegment, PitchContour, ProsodyConfig, SentenceType, StressLevel,
};
pub use ssml::{
    BreakSpec, BreakStrength, EmphasisLevel, SsmlDocument, SsmlElement, SsmlParser,
    SynthesisSegment,
};
pub use streaming::{AudioChunk, AudioStream, StreamingConfig, StreamingSynthesizer};
pub use synthesizer::{
    espeak_initialize, espeak_set_voice_by_name, espeak_synth, espeak_terminate,
    espeak_text_to_phonemes, AudioOutputType, PhonemeFormat, PhonemeResult, Synthesizer,
};
pub use voice::{Language, VoiceConfig, VoiceVariant};
