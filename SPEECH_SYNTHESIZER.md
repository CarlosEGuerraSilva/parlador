# Speech Synthesizer Documentation

Parlador is a Rust-powered multiplatform speech synthesizer engine that uses formant synthesis to generate speech audio from text. It is a completely custom implementation with no external TTS dependencies, designed to be compatible with the espeak-ng API for easy integration with existing projects.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
  - [Synthesizer](#synthesizer)
  - [VoiceConfig](#voiceconfig)
  - [Language](#language)
  - [VoiceVariant](#voicevariant)
  - [PhonemeFormat](#phonemeformat)
  - [AudioOutput](#audiooutput)
- [espeak-ng Compatible API](#espeak-ng-compatible-api)
- [Common Options](#common-options)
- [Phoneme System](#phoneme-system)
- [Formant Synthesis](#formant-synthesis)
- [Integration with TTS Models](#integration-with-tts-models)
  - [Kokoro Integration](#kokoro-integration)
  - [Custom TTS Models](#custom-tts-models)
- [Platform-Specific Considerations](#platform-specific-considerations)
- [Architecture](#architecture)
- [Examples](#examples)
- [Future Improvements](#future-improvements)

## Features

- **Cross-platform**: Pure Rust implementation, works on Linux, macOS, and Windows
- **No external dependencies**: Self-contained formant synthesis engine
- **Multiple languages**: Support for English and Spanish
- **Voice customization**: Adjust rate, pitch, volume, and voice variants
- **Phoneme generation**: Compatible with TTS models like Kokoro
- **Audio synthesis**: Get raw audio data (16-bit PCM) for further processing
- **espeak-ng compatible API**: Easy migration from espeak-ng

## Installation

Add Parlador to your `Cargo.toml`:

```toml
[dependencies]
parlador = "0.1"
```

Or add it via cargo:
```bash
cargo add parlador
```

No external dependencies are required - Parlador is a pure Rust implementation.

## Quick Start

```rust
use parlador::{Synthesizer, VoiceConfig, Language};

fn main() -> Result<(), parlador::SynthesizerError> {
    // Create a synthesizer with default English voice
    let synth = Synthesizer::new()?;
    
    // Synthesize speech and get audio data
    let audio = synth.synthesize("Hello, world!")?;
    println!("Generated {} samples at {} Hz", audio.samples.len(), audio.sample_rate);
    println!("Duration: {:.2} seconds", audio.duration_secs());
    
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

## API Reference

### Synthesizer

The main struct for speech synthesis.

```rust
use parlador::{Synthesizer, VoiceConfig, Language};

// Create with default settings
let synth = Synthesizer::new()?;

// Create with custom configuration
let config = VoiceConfig::new(Language::Spanish)
    .with_rate(200)
    .with_pitch(20);
let synth = Synthesizer::with_config(config)?;
```

**Methods:**

| Method | Description |
|--------|-------------|
| `new()` | Create with default settings (English) |
| `with_config(config)` | Create with custom voice configuration |
| `synthesize(text)` | Synthesize text to `AudioOutput` |
| `text_to_phonemes(text, format)` | Convert text to phonemes |
| `set_language(lang)` | Change the language |
| `set_rate(wpm)` | Change speech rate |
| `set_pitch(pitch)` | Change pitch (-100 to 100) |
| `set_volume(volume)` | Change volume (0-200) |
| `sample_rate()` | Get the audio sample rate (22050 Hz) |
| `supported_languages()` | Get list of supported languages |

### VoiceConfig

Configuration for voice settings.

```rust
use parlador::{VoiceConfig, Language, VoiceVariant};

let config = VoiceConfig::new(Language::English)
    .with_variant(VoiceVariant::Female1)
    .with_rate(175)      // Words per minute
    .with_pitch(0)       // -100 to 100
    .with_volume(100);   // 0-200
```

### Language

Supported languages:

| Language | Code | Aliases |
|----------|------|---------|
| English | `en` | `eng`, `english`, `en-us`, `en-gb` |
| Spanish | `es` | `spa`, `spanish`, `es-es`, `es-mx` |

```rust
use parlador::Language;

let lang = Language::from_code("es").unwrap();
assert_eq!(lang.code(), "es");
assert_eq!(lang.name(), "Spanish");
```

### VoiceVariant

Available voice variants with different base pitch frequencies:

| Variant | Base Pitch | Description |
|---------|------------|-------------|
| `Default` | 130 Hz | Default voice |
| `Male1` | 100 Hz | Low male voice |
| `Male2` | 120 Hz | Medium male voice |
| `Male3` | 140 Hz | High male voice |
| `Female1` | 180 Hz | Low female voice |
| `Female2` | 200 Hz | Medium female voice |
| `Female3` | 220 Hz | High female voice |

### PhonemeFormat

Phoneme output formats:

| Format | Description |
|--------|-------------|
| `Ipa` | International Phonetic Alphabet (IPA) symbols |
| `Ascii` | Internal ASCII phoneme representation |

### AudioOutput

Audio data returned by `synthesize()`:

```rust
let audio = synth.synthesize("Hello")?;

println!("Samples: {}", audio.samples.len());
println!("Sample rate: {} Hz", audio.sample_rate);
println!("Channels: {}", audio.channels);
println!("Duration: {:.2}s", audio.duration_secs());
```

| Field | Type | Description |
|-------|------|-------------|
| `samples` | `Vec<i16>` | Raw 16-bit signed PCM audio |
| `sample_rate` | `u32` | Sample rate (22050 Hz) |
| `channels` | `u16` | Number of channels (1 = mono) |

## espeak-ng Compatible API

Parlador provides an espeak-ng compatible API for easy migration:

```rust
use parlador::{
    espeak_initialize, espeak_set_voice_by_name, 
    espeak_synth, espeak_text_to_phonemes, 
    espeak_terminate, AudioOutputType
};

// Initialize (returns sample rate)
let sample_rate = espeak_initialize(AudioOutputType::Retrieval, 500, None, 0)?;

// Set voice
espeak_set_voice_by_name("en")?;

// Synthesize speech
let audio = espeak_synth("Hello, world!", "en")?;

// Get phonemes
let phonemes = espeak_text_to_phonemes("Hello", "en", true)?; // true = IPA

// Terminate (no-op, for API compatibility)
espeak_terminate();
```

### Migration from espeak-ng

| espeak-ng Function | Parlador Equivalent |
|--------------------|---------------------|
| `espeak_Initialize()` | `espeak_initialize()` |
| `espeak_SetVoiceByName()` | `espeak_set_voice_by_name()` |
| `espeak_Synth()` | `espeak_synth()` |
| `espeak_TextToPhonemes()` | `espeak_text_to_phonemes()` |
| `espeak_Terminate()` | `espeak_terminate()` |

## Common Options

### Speech Rate

The rate is specified in words per minute (WPM):

| Rate | Description |
|------|-------------|
| 50 | Very slow (minimum) |
| 100 | Slow |
| 175 | Normal (default) |
| 250 | Fast |
| 400 | Very fast |
| 500 | Maximum |

```rust
let config = VoiceConfig::new(Language::English).with_rate(200);
```

### Pitch

Pitch adjustment from -100 (lower) to 100 (higher):

| Pitch | Description |
|-------|-------------|
| -100 | Much lower |
| -50 | Lower |
| 0 | Normal (default) |
| 50 | Higher |
| 100 | Much higher |

The effective pitch is calculated as: `base_pitch * (1 + pitch/100 * 0.5)`

```rust
let config = VoiceConfig::new(Language::English).with_pitch(30);
```

### Volume

Volume ranges from 0 (silent) to 200 (very loud):

| Volume | Description |
|--------|-------------|
| 0 | Silent |
| 50 | Quiet |
| 100 | Normal (default) |
| 150 | Loud |
| 200 | Maximum |

```rust
let config = VoiceConfig::new(Language::English).with_volume(150);
```

## Phoneme System

Parlador uses an internal phoneme representation based on ASCII symbols, with mappings to IPA.

### English Phonemes

**Vowels:**
| Symbol | IPA | Example |
|--------|-----|---------|
| `i` | iː | beet |
| `I` | ɪ | bit |
| `e` | eɪ | bait |
| `E` | ɛ | bet |
| `&` | æ | bat |
| `A` | ɑː | bot |
| `O` | ɔː | bought |
| `o` | oʊ | boat |
| `U` | ʊ | book |
| `u` | uː | boot |
| `@` | ə | about |

**Diphthongs:**
| Symbol | IPA | Example |
|--------|-----|---------|
| `aI` | aɪ | bite |
| `aU` | aʊ | bout |
| `OI` | ɔɪ | boy |

**Consonants:**
| Symbol | IPA | Example |
|--------|-----|---------|
| `p`, `b`, `t`, `d`, `k`, `g` | p, b, t, d, k, g | pat, bat, etc. |
| `f`, `v`, `T`, `D`, `s`, `z` | f, v, θ, ð, s, z | fat, vat, thin, this, etc. |
| `S`, `Z`, `h` | ʃ, ʒ, h | ship, measure, hat |
| `tS`, `dZ` | tʃ, dʒ | chip, judge |
| `m`, `n`, `N` | m, n, ŋ | map, nap, sing |
| `l`, `r`, `w`, `j` | l, ɹ, w, j | lip, rip, wit, yes |

### Spanish Phonemes

**Vowels:**
| Symbol | IPA | Example |
|--------|-----|---------|
| `a` | a | casa |
| `e` | e | peso |
| `i` | i | piso |
| `o` | o | oso |
| `u` | u | luna |

**Consonants:**
| Symbol | IPA | Example |
|--------|-----|---------|
| `p`, `b`, `t`, `d`, `k`, `g` | p, b, t, d, k, g | pan, bien, etc. |
| `f`, `s`, `x`, `T` | f, s, x, θ | feo, sol, jota, cena |
| `tS` | tʃ | chico |
| `m`, `n`, `J` | m, n, ɲ | mano, nada, año |
| `l`, `L`, `r`, `rr` | l, ʎ, ɾ, r | lado, llave, pero, perro |
| `j`, `w` | j, w | hielo, huevo |

## Formant Synthesis

Parlador uses a Klatt-style formant synthesizer to generate speech. The synthesis process involves:

1. **Source Generation**: Glottal pulses for voiced sounds, noise for unvoiced sounds
2. **Formant Filtering**: Bandpass resonators tuned to vowel formant frequencies (F1, F2, F3)
3. **Amplitude Envelope**: Natural attack and decay for each phoneme

### Formant Frequencies

Each vowel has characteristic formant frequencies that define its sound:

| Vowel | F1 (Hz) | F2 (Hz) | F3 (Hz) |
|-------|---------|---------|---------|
| /i/ | 270 | 2290 | 3010 |
| /ɛ/ | 610 | 1900 | 2530 |
| /ɑ/ | 730 | 1090 | 2440 |
| /ɔ/ | 570 | 840 | 2410 |
| /u/ | 300 | 870 | 2240 |

## Integration with TTS Models

### Kokoro Integration

Parlador can generate phonemes for use with [Kokoro](https://github.com/hexgrad/kokoro) TTS:

```rust
use parlador::{Synthesizer, PhonemeFormat, Language, VoiceConfig};

// Create synthesizer for the target language
let synth = Synthesizer::with_config(
    VoiceConfig::new(Language::English)
)?;

// Generate IPA phonemes for Kokoro
let result = synth.text_to_phonemes(
    "Hello, this is a test.",
    PhonemeFormat::Ipa
)?;

println!("Phonemes for Kokoro: {}", result.phonemes);
// Use these phonemes with Kokoro model inference
```

### Custom TTS Models

For custom TTS models, you can:

1. Use `PhonemeFormat::Ascii` for the internal representation
2. Use `PhonemeFormat::Ipa` for IPA-based models
3. Access the `PhonemeInventory` directly for phoneme metadata

```rust
use parlador::{PhonemeInventory, G2PConverter};

// Get phoneme inventory for detailed information
let inventory = PhonemeInventory::english();

// Get phoneme details
if let Some(phoneme) = inventory.get("i") {
    println!("IPA: {}", phoneme.ipa);
    println!("Duration: {} ms", phoneme.duration_ms);
    if let Some(formants) = &phoneme.formants {
        println!("F1: {} Hz, F2: {} Hz, F3: {} Hz", 
                 formants.f1, formants.f2, formants.f3);
    }
}
```

## Platform-Specific Considerations

### Linux

- No additional dependencies required
- Audio output is returned as raw PCM data
- Convert to WAV using external tools if needed:
  ```bash
  sox -r 22050 -b 16 -e signed -c 1 output.raw output.wav
  ```

### macOS

- No additional dependencies required
- Works on both Intel and Apple Silicon

### Windows

- No additional dependencies required
- Pure Rust implementation compiles natively

### Cross-Compilation

Since Parlador is pure Rust with no native dependencies:

```bash
# Cross-compile to ARM Linux
cargo build --target aarch64-unknown-linux-gnu

# Cross-compile to Windows
cargo build --target x86_64-pc-windows-gnu
```

## Architecture

```
parlador/
├── src/
│   ├── lib.rs          # Main library exports
│   ├── error.rs        # Error types
│   ├── voice.rs        # Voice configuration
│   ├── phoneme.rs      # Phoneme definitions and inventories
│   ├── g2p.rs          # Grapheme-to-phoneme conversion
│   ├── formant.rs      # Formant synthesis engine
│   └── synthesizer.rs  # Main synthesizer implementation
└── examples/
    ├── speak.rs        # Speech synthesis example
    └── phonemes.rs     # Phoneme generation example
```

### Module Responsibilities

| Module | Description |
|--------|-------------|
| `error` | Error types and Result alias |
| `voice` | Language and voice configuration |
| `phoneme` | Phoneme inventory with acoustic properties |
| `g2p` | Text-to-phoneme conversion rules |
| `formant` | Klatt-style formant synthesis |
| `synthesizer` | Main API and espeak-ng compatibility |

## Examples

### Basic Speech Synthesis

```bash
cargo run --example speak
cargo run --example speak -- "Hello, world!"
cargo run --example speak -- --language es "¡Hola, mundo!"
cargo run --example speak -- --rate 200 --pitch 20 "Fast speech"
cargo run --example speak -- --output audio.raw "Save to file"
```

### Phoneme Generation

```bash
cargo run --example phonemes
cargo run --example phonemes -- "Hello"
cargo run --example phonemes -- --format ascii "Hello"
cargo run --example phonemes -- --language es "Hola"
```

### Saving Audio to File

```rust
use parlador::Synthesizer;
use std::io::Write;

let synth = Synthesizer::new()?;
let audio = synth.synthesize("Hello, world!")?;

// Write raw PCM data
let mut file = std::fs::File::create("output.raw")?;
for sample in &audio.samples {
    file.write_all(&sample.to_le_bytes())?;
}

// Convert to WAV using external tools:
// sox -r 22050 -b 16 -e signed -c 1 output.raw output.wav
```

## Future Improvements

The following features are planned for future releases:

- [ ] Improved prosody and intonation
- [ ] Additional languages (French, German, Portuguese)
- [ ] SSML (Speech Synthesis Markup Language) support
- [ ] Real-time audio streaming
- [ ] WAV file output without external tools
- [ ] Neural network-based G2P for better accuracy
- [ ] Voice cloning support
- [ ] Emotion/style control

## License

This project is licensed under the MIT License.
