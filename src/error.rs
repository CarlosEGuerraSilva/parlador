//! Error types for the Parlador speech synthesizer.

use thiserror::Error;

/// Errors that can occur during speech synthesis operations.
#[derive(Debug, Error)]
pub enum SynthesizerError {
    /// Failed to initialize the speech synthesizer.
    #[error("failed to initialize synthesizer: {0}")]
    InitializationError(String),

    /// Failed to set or get a voice configuration.
    #[error("voice configuration error: {0}")]
    VoiceError(String),

    /// Failed to synthesize speech from text.
    #[error("synthesis error: {0}")]
    SynthesisError(String),

    /// The requested language is not supported.
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// Failed to generate phonemes.
    #[error("phoneme generation error: {0}")]
    PhonemeError(String),

    /// Invalid phoneme input.
    #[error("invalid phoneme: {0}")]
    InvalidPhoneme(String),

    /// A system or platform-specific error occurred.
    #[error("system error: {0}")]
    SystemError(String),

    /// Audio output error.
    #[error("audio error: {0}")]
    AudioError(String),
}

/// Result type for speech synthesizer operations.
pub type Result<T> = std::result::Result<T, SynthesizerError>;
