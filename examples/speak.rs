//! Example: Basic speech synthesis with parlador.
//!
//! This example demonstrates how to use parlador to synthesize speech
//! in different languages with various voice configurations.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example speak
//! cargo run --example speak -- "Custom text to speak"
//! cargo run --example speak -- --language es "Texto en español"
//! cargo run --example speak -- --rate 200 --pitch 20 "Fast and high pitch"
//! ```

use parlador::{Language, Synthesizer, SynthesizerError, VoiceConfig, VoiceVariant};
use std::env;
use std::fs::File;
use std::io::Write;

fn print_usage() {
    println!("Usage: speak [OPTIONS] [TEXT]");
    println!();
    println!("Options:");
    println!("  --language, -l <LANG>   Language: en (English) or es (Spanish). Default: en");
    println!("  --rate, -r <WPM>        Speech rate in words per minute. Default: 175");
    println!("  --pitch, -p <PITCH>     Pitch adjustment (-100 to 100). Default: 0");
    println!("  --volume, -v <VOLUME>   Volume (0-200). Default: 100");
    println!("  --voice <VARIANT>       Voice variant: m1, m2, m3, f1, f2, f3. Default: default");
    println!("  --output, -o <FILE>     Output file (raw PCM). If not specified, prints info only.");
    println!("  --help, -h              Show this help message");
    println!();
    println!("Examples:");
    println!("  speak \"Hello, world!\"");
    println!("  speak --language es \"¡Hola, mundo!\"");
    println!("  speak --rate 200 --pitch 20 \"Fast speech\"");
    println!("  speak --voice f1 \"Female voice\"");
    println!("  speak --output output.raw \"Hello world\"");
}

fn parse_variant(s: &str) -> Option<VoiceVariant> {
    match s.to_lowercase().as_str() {
        "default" | "d" => Some(VoiceVariant::Default),
        "m1" | "male1" => Some(VoiceVariant::Male1),
        "m2" | "male2" => Some(VoiceVariant::Male2),
        "m3" | "male3" => Some(VoiceVariant::Male3),
        "f1" | "female1" => Some(VoiceVariant::Female1),
        "f2" | "female2" => Some(VoiceVariant::Female2),
        "f3" | "female3" => Some(VoiceVariant::Female3),
        _ => None,
    }
}

fn main() -> Result<(), SynthesizerError> {
    let args: Vec<String> = env::args().collect();

    let mut language = Language::English;
    let mut rate: u32 = 175;
    let mut pitch: i8 = 0;
    let mut volume: u8 = 100;
    let mut variant = VoiceVariant::Default;
    let mut output_file: Option<String> = None;
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
            "--rate" | "-r" => {
                i += 1;
                if i < args.len() {
                    rate = args[i].parse().unwrap_or(175);
                }
            }
            "--pitch" | "-p" => {
                i += 1;
                if i < args.len() {
                    pitch = args[i].parse().unwrap_or(0);
                }
            }
            "--volume" | "-v" => {
                i += 1;
                if i < args.len() {
                    volume = args[i].parse().unwrap_or(100);
                }
            }
            "--voice" => {
                i += 1;
                if i < args.len() {
                    variant = parse_variant(&args[i]).unwrap_or(VoiceVariant::Default);
                }
            }
            "--output" | "-o" => {
                i += 1;
                if i < args.len() {
                    output_file = Some(args[i].clone());
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
            Language::English => {
                "Hello! This is a demonstration of the Parlador speech synthesizer.".to_string()
            }
            Language::Spanish => {
                "¡Hola! Esta es una demostración del sintetizador de voz Parlador.".to_string()
            }
        };
    }

    // Create voice configuration
    let config = VoiceConfig::new(language)
        .with_variant(variant)
        .with_rate(rate)
        .with_pitch(pitch)
        .with_volume(volume);

    println!("Parlador Speech Synthesizer");
    println!("===========================");
    println!("Language: {}", language);
    println!("Voice: {}", variant.name());
    println!("Rate: {} WPM", rate);
    println!("Pitch: {}", pitch);
    println!("Volume: {}", volume);
    println!();
    println!("Text: \"{}\"", text);
    println!();

    // Create synthesizer and synthesize
    let synth = Synthesizer::with_config(config)?;
    let audio = synth.synthesize(&text)?;

    println!("Generated {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
    println!("Duration: {:.2} seconds", audio.duration_secs());

    // Save to file if requested
    if let Some(filename) = output_file {
        let mut file = File::create(&filename).map_err(|e| {
            SynthesizerError::SystemError(format!("Failed to create file: {}", e))
        })?;

        for sample in &audio.samples {
            file.write_all(&sample.to_le_bytes()).map_err(|e| {
                SynthesizerError::SystemError(format!("Failed to write to file: {}", e))
            })?;
        }

        println!("\nAudio saved to: {}", filename);
        println!("To convert to WAV, use:");
        println!(
            "  sox -r {} -b 16 -e signed -c 1 {} output.wav",
            audio.sample_rate, filename
        );
    }

    println!("\nDone!");
    Ok(())
}
