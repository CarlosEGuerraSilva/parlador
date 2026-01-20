# Parlador

Rust-powered multiplatform speech synthesizer engine with formant synthesis.

## Features

- **Cross-platform**: Pure Rust implementation, works on Linux, macOS, and Windows
- **No external dependencies**: Self-contained formant synthesis engine
- **Multiple languages**: Support for English and Spanish
- **Voice customization**: Adjust rate, pitch, volume, and voice variants
- **Phoneme generation**: Compatible with TTS models like [Kokoro](https://github.com/hexgrad/kokoro)
- **espeak-ng compatible API**: Easy migration from espeak-ng

## Quick Start

```rust
use parlador::{Synthesizer, VoiceConfig, Language};

fn main() -> Result<(), parlador::SynthesizerError> {
    // Create a synthesizer with default English voice
    let synth = Synthesizer::new()?;
    
    // Synthesize speech and get audio data
    let audio = synth.synthesize("Hello, world!")?;
    println!("Generated {} samples at {} Hz", audio.samples.len(), audio.sample_rate);

    // Create a Spanish synthesizer with custom settings
    let spanish_synth = Synthesizer::with_config(
        VoiceConfig::new(Language::Spanish)
            .with_rate(150)
            .with_pitch(20)
    )?;
    let audio = spanish_synth.synthesize("¡Hola, mundo!")?;

    Ok(())
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
parlador = "0.1"
```

No external dependencies required - Parlador is a pure Rust implementation.

## Phoneme Generation for TTS Models

Generate phonemes for use with external TTS models like Kokoro:

```rust
use parlador::{Synthesizer, PhonemeFormat};

let synth = Synthesizer::new()?;
let result = synth.text_to_phonemes("Hello, world!", PhonemeFormat::Ipa)?;
println!("Phonemes: {}", result.phonemes);
```

## espeak-ng Compatible API

For projects migrating from espeak-ng:

```rust
use parlador::{espeak_initialize, espeak_synth, espeak_text_to_phonemes, AudioOutputType};

// Initialize
let sample_rate = espeak_initialize(AudioOutputType::Retrieval, 500, None, 0)?;

// Synthesize speech
let audio = espeak_synth("Hello", "en")?;

// Get phonemes
let phonemes = espeak_text_to_phonemes("Hello", "en", true)?;
```

## Examples

```bash
# Basic speech synthesis
cargo run --example speak
cargo run --example speak -- --language es "¡Hola, mundo!"

# Phoneme generation
cargo run --example phonemes
cargo run --example phonemes -- --format ascii "Hello"
```

## Documentation

See [SPEECH_SYNTHESIZER.md](SPEECH_SYNTHESIZER.md) for comprehensive documentation including:

- Detailed API reference
- Phoneme system documentation
- Formant synthesis explanation
- Integration with TTS models like Kokoro
- Platform-specific considerations

## Architecture

Parlador uses Klatt-style formant synthesis:

1. **Text Input** → **G2P Conversion** → **Phoneme Sequence**
2. **Phoneme Sequence** → **Formant Synthesis** → **Audio Output**

No neural networks or external TTS engines are used - all synthesis is done through DSP algorithms.

## License

MIT License
