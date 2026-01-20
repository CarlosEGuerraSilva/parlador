//! Example: Real-time audio streaming with parlador.
//!
//! This example demonstrates how to use the streaming API to generate
//! audio incrementally, which is useful for low-latency applications.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example streaming
//! cargo run --example streaming -- "Custom text to stream"
//! cargo run --example streaming -- --chunk-size 2048 "Larger chunks"
//! ```

use parlador::{
    AudioChunk, Language, StreamingConfig, StreamingSynthesizer, SynthesizerError, VoiceConfig,
};
use std::env;
use std::fs::File;
use std::io::Write;

fn print_usage() {
    println!("Usage: streaming [OPTIONS] [TEXT]");
    println!();
    println!("Options:");
    println!("  --language, -l <LANG>      Language: en (English) or es (Spanish). Default: en");
    println!("  --chunk-size, -c <SIZE>    Chunk size in samples. Default: 1024");
    println!("  --output, -o <FILE>        Output file (raw PCM). If not specified, prints info only.");
    println!("  --help, -h                 Show this help message");
    println!();
    println!("Examples:");
    println!("  streaming \"Hello, world!\"");
    println!("  streaming --chunk-size 2048 \"Larger chunks for less overhead\"");
    println!("  streaming --output output.raw \"Save to file\"");
}

fn main() -> Result<(), SynthesizerError> {
    let args: Vec<String> = env::args().collect();

    let mut language = Language::English;
    let mut chunk_size: usize = 1024;
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
            "--chunk-size" | "-c" => {
                i += 1;
                if i < args.len() {
                    chunk_size = args[i].parse().unwrap_or(1024);
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
                "Hello! This is a demonstration of real-time audio streaming with Parlador. \
                 The audio is generated in chunks as you listen.".to_string()
            }
            Language::Spanish => {
                "¡Hola! Esta es una demostración de transmisión de audio en tiempo real con Parlador. \
                 El audio se genera en fragmentos mientras escuchas.".to_string()
            }
        };
    }

    println!("Parlador Streaming Synthesizer");
    println!("==============================");
    println!("Language: {}", language);
    println!("Chunk size: {} samples", chunk_size);
    println!();
    println!("Text: \"{}\"", text);
    println!();

    // Create streaming synthesizer
    let config = StreamingConfig::new()
        .with_chunk_size(chunk_size)
        .with_voice(VoiceConfig::new(language))
        .with_prosody(true);

    let synth = StreamingSynthesizer::with_config(config);

    // Create output file if requested
    let mut file = output_file.as_ref().map(|filename| {
        File::create(filename).expect("Failed to create output file")
    });

    println!("Streaming audio...");
    println!();

    // Stream audio chunks
    let stream = synth.synthesize_stream(&text)?;
    let mut total_samples = 0;
    let mut chunk_count = 0;

    for chunk in stream {
        chunk_count += 1;
        total_samples += chunk.samples.len();

        // Print progress
        print_chunk_info(&chunk, chunk_count);

        // Write to file if requested
        if let Some(ref mut f) = file {
            for sample in &chunk.samples {
                f.write_all(&sample.to_le_bytes())
                    .expect("Failed to write to file");
            }
        }
    }

    println!();
    println!("Streaming complete!");
    println!("Total chunks: {}", chunk_count);
    println!("Total samples: {}", total_samples);
    println!(
        "Total duration: {:.2} seconds",
        total_samples as f64 / 22050.0
    );

    if let Some(filename) = output_file {
        println!();
        println!("Audio saved to: {}", filename);
        println!("To convert to WAV, use:");
        println!("  sox -r 22050 -b 16 -e signed -c 1 {} output.wav", filename);
    }

    Ok(())
}

fn print_chunk_info(chunk: &AudioChunk, chunk_number: usize) {
    let progress_bar_width = 30;
    let filled = (chunk.progress * progress_bar_width as f32) as usize;
    let empty = progress_bar_width - filled;

    let bar: String = "█".repeat(filled) + &"░".repeat(empty);

    print!(
        "\rChunk {:3}: {:5} samples | [{}] {:5.1}%",
        chunk_number,
        chunk.samples.len(),
        bar,
        chunk.progress * 100.0
    );

    if chunk.is_final {
        println!(" ✓ Done!");
    }

    std::io::stdout().flush().ok();
}
