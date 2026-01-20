# Parlador

Rust-powered multiplatform speech synthesizer engine with formant synthesis.

## Features

- **Cross-platform**: Pure Rust implementation, works on Linux, macOS, and Windows
- **No external dependencies**: Self-contained formant synthesis engine
- **Multiple languages**: Support for English and Spanish
- **Voice customization**: Adjust rate, pitch, volume, and voice variants
- **Phoneme generation**: Compatible with TTS models like [Kokoro](https://github.com/hexgrad/kokoro)
- **SSML support**: Parse and synthesize SSML documents for fine-grained control
- **Real-time streaming**: Generate audio incrementally for low-latency applications
- **Improved prosody**: Natural intonation patterns for different sentence types
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

## SSML Support

Use SSML (Speech Synthesis Markup Language) for fine-grained control over speech synthesis:

```rust
use parlador::Synthesizer;

let synth = Synthesizer::new()?;
let ssml = r#"<speak>
    Hello <break time="500ms"/> world!
    <prosody rate="fast" pitch="high">This is fast and high.</prosody>
    <emphasis level="strong">Important!</emphasis>
</speak>"#;
let audio = synth.synthesize_ssml(ssml)?;
```

### Supported SSML Elements

| Element | Description | Attributes |
|---------|-------------|------------|
| `<speak>` | Root element | - |
| `<break>` | Insert a pause | `time`, `strength` |
| `<prosody>` | Modify rate, pitch, volume | `rate`, `pitch`, `volume` |
| `<emphasis>` | Apply emphasis | `level` |
| `<p>` / `<s>` | Paragraph/sentence markers | - |
| `<say-as>` | Interpretation hints | `interpret-as` |
| `<sub>` | Pronunciation substitution | `alias` |

## Real-time Streaming

Generate audio incrementally for low-latency applications:

```rust
use parlador::StreamingSynthesizer;

let synth = StreamingSynthesizer::new();
for chunk in synth.synthesize_stream("Hello, world!")? {
    // Process each audio chunk as it's generated
    println!("Got {} samples, progress: {:.1}%", 
             chunk.samples.len(), 
             chunk.progress * 100.0);
    
    // In a real application, you would send these samples
    // to an audio output device
}
```

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

# Real-time streaming
cargo run --example streaming
cargo run --example streaming -- --chunk-size 2048 "Larger chunks"
```

## Documentation

See [SPEECH_SYNTHESIZER.md](SPEECH_SYNTHESIZER.md) for comprehensive documentation including:

- Detailed API reference
- Phoneme system documentation
- Formant synthesis explanation
- SSML support documentation
- Real-time streaming guide
- Integration with TTS models like Kokoro
- Platform-specific considerations

## Architecture

Parlador uses Klatt-style formant synthesis:

1. **Text Input** → **G2P Conversion** → **Phoneme Sequence**
2. **Phoneme Sequence** → **Formant Synthesis** → **Audio Output**

No neural networks or external TTS engines are used - all synthesis is done through DSP algorithms.

## License

MIT License
