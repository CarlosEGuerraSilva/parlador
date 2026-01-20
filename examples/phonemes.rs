//! Example: Phoneme generation for TTS model integration.
//!
//! This example demonstrates how to use parlador to generate phonemes
//! from text, which can be used with external TTS models like Kokoro.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example phonemes
//! cargo run --example phonemes -- "Custom text"
//! cargo run --example phonemes -- --language es "Texto en español"
//! cargo run --example phonemes -- --format ascii "Hello world"
//! ```

use parlador::{Language, PhonemeFormat, Synthesizer, SynthesizerError, VoiceConfig};
use std::env;

fn print_usage() {
    println!("Usage: phonemes [OPTIONS] [TEXT]");
    println!();
    println!("Options:");
    println!("  --language, -l <LANG>   Language: en (English) or es (Spanish). Default: en");
    println!("  --format, -f <FORMAT>   Phoneme format: ipa or ascii. Default: ipa");
    println!("  --help, -h              Show this help message");
    println!();
    println!("Examples:");
    println!("  phonemes \"Hello, world!\"");
    println!("  phonemes --language es \"¡Hola, mundo!\"");
    println!("  phonemes --format ascii \"Hello\"");
    println!();
    println!("Output Format:");
    println!("  IPA:   International Phonetic Alphabet symbols");
    println!("  ASCII: Internal phoneme representation");
}

fn main() -> Result<(), SynthesizerError> {
    let args: Vec<String> = env::args().collect();

    let mut language = Language::English;
    let mut format = PhonemeFormat::Ipa;
    let mut text = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            "--language" | "-l" => {
                i += 1;
                if i < args.len() {
                    language = Language::from_code(&args[i]).unwrap_or_else(|| {
                        eprintln!("Warning: Unknown language '{}', using English", args[i]);
                        Language::English
                    });
                }
            }
            "--format" | "-f" => {
                i += 1;
                if i < args.len() {
                    format = match args[i].to_lowercase().as_str() {
                        "ipa" => PhonemeFormat::Ipa,
                        "ascii" => PhonemeFormat::Ascii,
                        _ => {
                            eprintln!("Warning: Unknown format '{}', using IPA", args[i]);
                            PhonemeFormat::Ipa
                        }
                    };
                }
            }
            s if !s.starts_with('-') => {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(s);
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
            }
        }
        i += 1;
    }

    // Default text if none provided
    if text.is_empty() {
        text = match language {
            Language::English => "Hello world. This is a test of phoneme generation.".to_string(),
            Language::Spanish => {
                "Hola mundo. Esta es una prueba de generación de fonemas.".to_string()
            }
        };
    }

    // Create synthesizer
    let config = VoiceConfig::new(language);
    let synth = Synthesizer::with_config(config)?;

    println!("Parlador Phoneme Generator");
    println!("==========================");
    println!("Language: {}", language);
    println!("Format: {:?}", format);
    println!();
    println!("Input text: \"{}\"", text);
    println!();

    // Generate phonemes
    let result = synth.text_to_phonemes(&text, format)?;

    println!("Phonemes:");
    println!("{}", result.phonemes);
    println!();

    // Show usage example for Kokoro
    println!("Integration Example (Kokoro TTS):");
    println!("----------------------------------");
    println!("The generated phonemes can be passed to Kokoro or similar");
    println!("TTS models that expect phoneme input instead of raw text.");
    println!();
    println!("For IPA format, use with Kokoro models that support IPA input.");
    println!("For ASCII format, use with parlador-compatible TTS pipelines.");

    Ok(())
}
