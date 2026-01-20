//! SSML (Speech Synthesis Markup Language) parser and support.
//!
//! This module provides parsing and processing of SSML documents
//! for controlling speech synthesis with markup.

use crate::error::{Result, SynthesizerError};
use crate::prosody::{PitchContour, ProsodyConfig};
use std::collections::HashMap;

/// An SSML element with its content and attributes.
#[derive(Debug, Clone)]
pub enum SsmlElement {
    /// Plain text content.
    Text(String),
    /// A break/pause element.
    Break(BreakSpec),
    /// Prosody modification element.
    Prosody {
        /// Child elements.
        children: Vec<SsmlElement>,
        /// Prosody settings.
        config: ProsodyConfig,
    },
    /// Emphasis element.
    Emphasis {
        /// Child elements.
        children: Vec<SsmlElement>,
        /// Emphasis level.
        level: EmphasisLevel,
    },
    /// Say-as element for interpretation.
    SayAs {
        /// The text to interpret.
        text: String,
        /// Interpretation type.
        interpret_as: String,
    },
    /// Sub element for pronunciation substitution.
    Sub {
        /// The alias to speak.
        alias: String,
    },
    /// Voice element to change voice characteristics.
    Voice {
        /// Child elements.
        children: Vec<SsmlElement>,
        /// Voice name/language.
        name: Option<String>,
    },
    /// Paragraph element.
    Paragraph(Vec<SsmlElement>),
    /// Sentence element.
    Sentence(Vec<SsmlElement>),
}

/// Break/pause specification.
#[derive(Debug, Clone)]
pub struct BreakSpec {
    /// Break duration in milliseconds.
    pub time_ms: u32,
    /// Break strength (if time not specified).
    pub strength: BreakStrength,
}

impl Default for BreakSpec {
    fn default() -> Self {
        Self {
            time_ms: 0,
            strength: BreakStrength::Medium,
        }
    }
}

/// Break strength levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreakStrength {
    /// No break.
    None,
    /// Very short break (~100ms).
    XWeak,
    /// Short break (~200ms).
    Weak,
    /// Medium break (~400ms).
    #[default]
    Medium,
    /// Long break (~600ms).
    Strong,
    /// Very long break (~1000ms).
    XStrong,
}

impl BreakStrength {
    /// Convert strength to milliseconds.
    #[must_use]
    pub fn to_ms(self) -> u32 {
        match self {
            BreakStrength::None => 0,
            BreakStrength::XWeak => 100,
            BreakStrength::Weak => 200,
            BreakStrength::Medium => 400,
            BreakStrength::Strong => 600,
            BreakStrength::XStrong => 1000,
        }
    }

    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" => BreakStrength::None,
            "x-weak" => BreakStrength::XWeak,
            "weak" => BreakStrength::Weak,
            "medium" => BreakStrength::Medium,
            "strong" => BreakStrength::Strong,
            "x-strong" => BreakStrength::XStrong,
            _ => BreakStrength::Medium,
        }
    }
}

/// Emphasis level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmphasisLevel {
    /// No emphasis (reduced).
    Reduced,
    /// No change.
    None,
    /// Moderate emphasis.
    #[default]
    Moderate,
    /// Strong emphasis.
    Strong,
}

impl EmphasisLevel {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "reduced" => EmphasisLevel::Reduced,
            "none" => EmphasisLevel::None,
            "moderate" => EmphasisLevel::Moderate,
            "strong" => EmphasisLevel::Strong,
            _ => EmphasisLevel::Moderate,
        }
    }

    /// Convert to prosody emphasis value.
    #[must_use]
    pub fn to_emphasis_value(self) -> f32 {
        match self {
            EmphasisLevel::Reduced => 0.0,
            EmphasisLevel::None => 0.0,
            EmphasisLevel::Moderate => 0.5,
            EmphasisLevel::Strong => 1.0,
        }
    }

    /// Convert to volume multiplier.
    #[must_use]
    pub fn to_volume_multiplier(self) -> f32 {
        match self {
            EmphasisLevel::Reduced => 0.8,
            EmphasisLevel::None => 1.0,
            EmphasisLevel::Moderate => 1.1,
            EmphasisLevel::Strong => 1.3,
        }
    }
}

/// A parsed SSML document.
#[derive(Debug, Clone)]
pub struct SsmlDocument {
    /// Root elements.
    pub elements: Vec<SsmlElement>,
}

impl SsmlDocument {
    /// Creates a new empty SSML document.
    #[must_use]
    pub fn new() -> Self {
        Self { elements: vec![] }
    }

    /// Extracts plain text from the document.
    #[must_use]
    pub fn to_plain_text(&self) -> String {
        let mut result = String::new();
        Self::extract_text_from_elements(&self.elements, &mut result);
        result
    }

    fn extract_text_from_elements(elements: &[SsmlElement], result: &mut String) {
        for element in elements {
            match element {
                SsmlElement::Text(text) => {
                    if !result.is_empty() && !result.ends_with(' ') && !text.starts_with(' ') {
                        result.push(' ');
                    }
                    result.push_str(text);
                }
                SsmlElement::Break(spec) => {
                    // Add pause indicator
                    let pause_count = (spec.time_ms.max(spec.strength.to_ms()) / 200).max(1);
                    for _ in 0..pause_count {
                        result.push(' ');
                    }
                }
                SsmlElement::Prosody { children, .. } => {
                    Self::extract_text_from_elements(children, result);
                }
                SsmlElement::Emphasis { children, .. } => {
                    Self::extract_text_from_elements(children, result);
                }
                SsmlElement::SayAs { text, .. } => {
                    result.push_str(text);
                }
                SsmlElement::Sub { alias } => {
                    result.push_str(alias);
                }
                SsmlElement::Voice { children, .. } => {
                    Self::extract_text_from_elements(children, result);
                }
                SsmlElement::Paragraph(children) | SsmlElement::Sentence(children) => {
                    Self::extract_text_from_elements(children, result);
                    result.push(' ');
                }
            }
        }
    }

    /// Gets synthesis segments with prosody information.
    #[must_use]
    pub fn to_synthesis_segments(&self) -> Vec<SynthesisSegment> {
        let mut segments = Vec::new();
        let base_prosody = ProsodyConfig::default();
        Self::collect_segments(&self.elements, &base_prosody, &mut segments);
        segments
    }

    fn collect_segments(
        elements: &[SsmlElement],
        parent_prosody: &ProsodyConfig,
        segments: &mut Vec<SynthesisSegment>,
    ) {
        for element in elements {
            match element {
                SsmlElement::Text(text) => {
                    if !text.trim().is_empty() {
                        segments.push(SynthesisSegment {
                            text: text.clone(),
                            prosody: parent_prosody.clone(),
                        });
                    }
                }
                SsmlElement::Break(spec) => {
                    let duration = if spec.time_ms > 0 {
                        spec.time_ms
                    } else {
                        spec.strength.to_ms()
                    };
                    segments.push(SynthesisSegment {
                        text: String::new(),
                        prosody: ProsodyConfig::default()
                            .with_volume(0.0), // Silence
                    });
                    // Store break duration in a special way
                    if let Some(last) = segments.last_mut() {
                        last.text = format!("__break_{}__", duration);
                    }
                }
                SsmlElement::Prosody { children, config } => {
                    // Merge prosody configs
                    let merged = ProsodyConfig {
                        pitch_multiplier: parent_prosody.pitch_multiplier * config.pitch_multiplier,
                        rate_multiplier: parent_prosody.rate_multiplier * config.rate_multiplier,
                        volume_multiplier: parent_prosody.volume_multiplier * config.volume_multiplier,
                        contour: if config.contour != PitchContour::Flat {
                            config.contour
                        } else {
                            parent_prosody.contour
                        },
                        emphasis: config.emphasis.max(parent_prosody.emphasis),
                    };
                    Self::collect_segments(children, &merged, segments);
                }
                SsmlElement::Emphasis { children, level } => {
                    let emphasis_prosody = ProsodyConfig {
                        pitch_multiplier: parent_prosody.pitch_multiplier,
                        rate_multiplier: parent_prosody.rate_multiplier,
                        volume_multiplier: parent_prosody.volume_multiplier * level.to_volume_multiplier(),
                        contour: parent_prosody.contour,
                        emphasis: level.to_emphasis_value(),
                    };
                    Self::collect_segments(children, &emphasis_prosody, segments);
                }
                SsmlElement::SayAs { text, .. } => {
                    segments.push(SynthesisSegment {
                        text: text.clone(),
                        prosody: parent_prosody.clone(),
                    });
                }
                SsmlElement::Sub { alias } => {
                    segments.push(SynthesisSegment {
                        text: alias.clone(),
                        prosody: parent_prosody.clone(),
                    });
                }
                SsmlElement::Voice { children, .. } => {
                    Self::collect_segments(children, parent_prosody, segments);
                }
                SsmlElement::Paragraph(children) | SsmlElement::Sentence(children) => {
                    Self::collect_segments(children, parent_prosody, segments);
                }
            }
        }
    }
}

impl Default for SsmlDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// A segment of text with associated prosody.
#[derive(Debug, Clone)]
pub struct SynthesisSegment {
    /// The text content.
    pub text: String,
    /// Prosody configuration.
    pub prosody: ProsodyConfig,
}

/// SSML parser.
pub struct SsmlParser;

impl SsmlParser {
    /// Parse an SSML string into a document.
    pub fn parse(input: &str) -> Result<SsmlDocument> {
        let trimmed = input.trim();
        
        // Check if it looks like SSML (starts with <)
        if !trimmed.starts_with('<') {
            // Plain text - wrap in a document
            return Ok(SsmlDocument {
                elements: vec![SsmlElement::Text(input.to_string())],
            });
        }

        // Simple XML-like parser
        let mut parser = SimpleXmlParser::new(trimmed);
        let elements = parser.parse()?;

        Ok(SsmlDocument { elements })
    }

    /// Check if a string is SSML.
    #[must_use]
    pub fn is_ssml(input: &str) -> bool {
        let trimmed = input.trim();
        trimmed.starts_with("<speak") || trimmed.starts_with("<?xml")
    }
}

/// Simple XML parser for SSML.
struct SimpleXmlParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> SimpleXmlParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse(&mut self) -> Result<Vec<SsmlElement>> {
        let mut elements = Vec::new();
        
        while self.pos < self.input.len() {
            self.skip_whitespace();
            
            if self.pos >= self.input.len() {
                break;
            }

            if self.peek() == Some('<') {
                if self.peek_str("<?") {
                    // XML declaration - skip
                    self.skip_until("?>");
                    self.pos += 2;
                } else if self.peek_str("<!--") {
                    // Comment - skip
                    self.skip_until("-->");
                    self.pos += 3;
                } else if self.peek_str("</") {
                    // Closing tag - return to parent
                    break;
                } else {
                    // Opening tag
                    if let Some(element) = self.parse_element()? {
                        elements.push(element);
                    }
                }
            } else {
                // Text content
                let text = self.parse_text();
                if !text.is_empty() {
                    elements.push(SsmlElement::Text(text));
                }
            }
        }

        Ok(elements)
    }

    fn parse_element(&mut self) -> Result<Option<SsmlElement>> {
        // Skip '<'
        self.pos += 1;
        self.skip_whitespace();

        // Get tag name
        let tag_name = self.parse_name();
        if tag_name.is_empty() {
            return Err(SynthesizerError::SynthesisError("Invalid SSML: empty tag name".to_string()));
        }

        // Parse attributes
        let attributes = self.parse_attributes()?;

        self.skip_whitespace();

        // Check for self-closing tag
        let self_closing = self.peek() == Some('/');
        if self_closing {
            self.pos += 1;
        }

        // Skip '>'
        if self.peek() != Some('>') {
            return Err(SynthesizerError::SynthesisError(
                format!("Invalid SSML: expected '>' after tag '{}'", tag_name)
            ));
        }
        self.pos += 1;

        // Parse children if not self-closing
        let children = if self_closing {
            vec![]
        } else {
            let children = self.parse()?;
            // Skip closing tag
            self.skip_closing_tag(&tag_name);
            children
        };

        // Convert to SsmlElement
        Ok(Some(self.create_element(&tag_name, attributes, children)?))
    }

    fn create_element(
        &self,
        tag_name: &str,
        attributes: HashMap<String, String>,
        children: Vec<SsmlElement>,
    ) -> Result<SsmlElement> {
        match tag_name.to_lowercase().as_str() {
            "speak" => {
                // Root element - return children directly (will be wrapped in first child)
                if children.is_empty() {
                    Ok(SsmlElement::Text(String::new()))
                } else if children.len() == 1 {
                    Ok(children.into_iter().next().unwrap())
                } else {
                    Ok(SsmlElement::Paragraph(children))
                }
            }
            "break" => {
                let mut spec = BreakSpec::default();
                
                if let Some(time) = attributes.get("time") {
                    spec.time_ms = parse_duration(time);
                }
                if let Some(strength) = attributes.get("strength") {
                    spec.strength = BreakStrength::parse(strength);
                }
                
                Ok(SsmlElement::Break(spec))
            }
            "prosody" => {
                let mut config = ProsodyConfig::default();
                
                if let Some(rate) = attributes.get("rate") {
                    config.rate_multiplier = parse_rate(rate);
                }
                if let Some(pitch) = attributes.get("pitch") {
                    config.pitch_multiplier = parse_pitch(pitch);
                }
                if let Some(volume) = attributes.get("volume") {
                    config.volume_multiplier = parse_volume(volume);
                }
                if let Some(contour) = attributes.get("contour") {
                    config.contour = parse_contour(contour);
                }
                
                Ok(SsmlElement::Prosody { children, config })
            }
            "emphasis" => {
                let level = attributes
                    .get("level")
                    .map(|s| EmphasisLevel::parse(s))
                    .unwrap_or(EmphasisLevel::Moderate);
                
                Ok(SsmlElement::Emphasis { children, level })
            }
            "say-as" => {
                let interpret_as = attributes
                    .get("interpret-as")
                    .cloned()
                    .unwrap_or_default();
                
                let text = Self::extract_text(&children);
                Ok(SsmlElement::SayAs { text, interpret_as })
            }
            "sub" => {
                let alias = attributes.get("alias").cloned().unwrap_or_default();
                Ok(SsmlElement::Sub { alias })
            }
            "voice" => {
                let name = attributes.get("name").cloned();
                Ok(SsmlElement::Voice { children, name })
            }
            "p" | "paragraph" => Ok(SsmlElement::Paragraph(children)),
            "s" | "sentence" => Ok(SsmlElement::Sentence(children)),
            _ => {
                // Unknown element - treat as container
                if children.is_empty() {
                    Ok(SsmlElement::Text(String::new()))
                } else if children.len() == 1 {
                    Ok(children.into_iter().next().unwrap())
                } else {
                    Ok(SsmlElement::Paragraph(children))
                }
            }
        }
    }

    fn extract_text(elements: &[SsmlElement]) -> String {
        let mut result = String::new();
        for element in elements {
            if let SsmlElement::Text(text) = element {
                result.push_str(text);
            }
        }
        result
    }

    fn parse_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ':' {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    fn parse_attributes(&mut self) -> Result<HashMap<String, String>> {
        let mut attrs = HashMap::new();
        
        loop {
            self.skip_whitespace();
            
            if self.pos >= self.input.len() {
                break;
            }
            
            let c = self.peek();
            if c == Some('>') || c == Some('/') {
                break;
            }

            let name = self.parse_name();
            if name.is_empty() {
                break;
            }

            self.skip_whitespace();
            
            if self.peek() == Some('=') {
                self.pos += 1;
                self.skip_whitespace();
                
                let value = self.parse_attribute_value()?;
                attrs.insert(name.to_lowercase(), value);
            }
        }

        Ok(attrs)
    }

    fn parse_attribute_value(&mut self) -> Result<String> {
        let quote = self.peek();
        if quote != Some('"') && quote != Some('\'') {
            return Err(SynthesizerError::SynthesisError(
                "Invalid SSML: expected quoted attribute value".to_string()
            ));
        }
        self.pos += 1;
        
        let start = self.pos;
        while self.pos < self.input.len() {
            if self.peek() == quote {
                let value = &self.input[start..self.pos];
                self.pos += 1;
                return Ok(decode_entities(value));
            }
            self.pos += 1;
        }

        Err(SynthesizerError::SynthesisError(
            "Invalid SSML: unclosed attribute value".to_string()
        ))
    }

    fn parse_text(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.input.len() && self.peek() != Some('<') {
            self.pos += 1;
        }
        let text = &self.input[start..self.pos];
        decode_entities(text.trim())
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_until(&mut self, pattern: &str) {
        if let Some(idx) = self.input[self.pos..].find(pattern) {
            self.pos += idx;
        } else {
            self.pos = self.input.len();
        }
    }

    fn skip_closing_tag(&mut self, tag_name: &str) {
        self.skip_whitespace();
        if self.peek_str("</") {
            self.pos += 2;
            let name = self.parse_name();
            if name.to_lowercase() == tag_name.to_lowercase() {
                self.skip_whitespace();
                if self.peek() == Some('>') {
                    self.pos += 1;
                }
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.as_bytes().get(self.pos).map(|&b| b as char)
    }

    fn peek_str(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }
}

/// Parse duration string (e.g., "500ms", "1s") to milliseconds.
fn parse_duration(s: &str) -> u32 {
    let s = s.trim().to_lowercase();
    
    if let Some(ms) = s.strip_suffix("ms") {
        ms.trim().parse().unwrap_or(0)
    } else if let Some(s_val) = s.strip_suffix('s') {
        let secs: f32 = s_val.trim().parse().unwrap_or(0.0);
        (secs * 1000.0) as u32
    } else {
        s.parse().unwrap_or(0)
    }
}

/// Parse rate string to multiplier.
fn parse_rate(s: &str) -> f32 {
    let s = s.trim().to_lowercase();
    
    match s.as_str() {
        "x-slow" => 0.5,
        "slow" => 0.75,
        "medium" => 1.0,
        "fast" => 1.25,
        "x-fast" => 1.5,
        "default" => 1.0,
        _ => {
            // Try percentage (e.g., "150%")
            if let Some(pct) = s.strip_suffix('%') {
                pct.trim().parse::<f32>().unwrap_or(100.0) / 100.0
            } else {
                // Try direct multiplier
                s.parse().unwrap_or(1.0)
            }
        }
    }
}

/// Parse pitch string to multiplier.
fn parse_pitch(s: &str) -> f32 {
    let s = s.trim().to_lowercase();
    
    match s.as_str() {
        "x-low" => 0.5,
        "low" => 0.75,
        "medium" => 1.0,
        "high" => 1.25,
        "x-high" => 1.5,
        "default" => 1.0,
        _ => {
            // Try percentage
            if let Some(pct) = s.strip_suffix('%') {
                pct.trim().parse::<f32>().unwrap_or(100.0) / 100.0
            // Try semitones (e.g., "+2st")
            } else if let Some(st) = s.strip_suffix("st") {
                let semitones: f32 = st.trim().parse().unwrap_or(0.0);
                2.0_f32.powf(semitones / 12.0)
            // Try Hz offset (e.g., "+50Hz")
            } else if s.ends_with("hz") {
                // Relative Hz is tricky, approximate
                1.0
            } else {
                s.parse().unwrap_or(1.0)
            }
        }
    }
}

/// Parse volume string to multiplier.
fn parse_volume(s: &str) -> f32 {
    let s = s.trim().to_lowercase();
    
    match s.as_str() {
        "silent" => 0.0,
        "x-soft" => 0.25,
        "soft" => 0.5,
        "medium" => 1.0,
        "loud" => 1.5,
        "x-loud" => 2.0,
        "default" => 1.0,
        _ => {
            // Try percentage
            if let Some(pct) = s.strip_suffix('%') {
                pct.trim().parse::<f32>().unwrap_or(100.0) / 100.0
            // Try dB
            } else if s.ends_with("db") {
                let db: f32 = s.strip_suffix("db").unwrap().trim().parse().unwrap_or(0.0);
                10.0_f32.powf(db / 20.0)
            } else {
                s.parse().unwrap_or(1.0)
            }
        }
    }
}

/// Parse contour string.
fn parse_contour(s: &str) -> PitchContour {
    let s = s.trim().to_lowercase();
    
    // Simple contour detection from SSML contour specification
    // Format: "(0%,+0%) (100%,+50%)" etc.
    if s.contains('+') || s.ends_with(')') {
        // Analyze the general trend
        if s.contains("100%,+") || s.contains("100%, +") {
            return PitchContour::Rising;
        } else if s.contains("100%,-") || s.contains("100%, -") {
            return PitchContour::Falling;
        }
    }
    
    PitchContour::Flat
}

/// Decode XML entities.
fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_text() {
        let doc = SsmlParser::parse("Hello world").unwrap();
        assert_eq!(doc.to_plain_text(), "Hello world");
    }

    #[test]
    fn test_parse_speak_element() {
        let ssml = "<speak>Hello world</speak>";
        let doc = SsmlParser::parse(ssml).unwrap();
        assert_eq!(doc.to_plain_text(), "Hello world");
    }

    #[test]
    fn test_parse_break() {
        let ssml = "<speak>Hello<break time=\"500ms\"/>world</speak>";
        let doc = SsmlParser::parse(ssml).unwrap();
        // Break should add spaces
        assert!(doc.to_plain_text().contains("Hello"));
        assert!(doc.to_plain_text().contains("world"));
    }

    #[test]
    fn test_parse_prosody() {
        let ssml = "<speak><prosody rate=\"fast\" pitch=\"high\">Hello</prosody></speak>";
        let doc = SsmlParser::parse(ssml).unwrap();
        let segments = doc.to_synthesis_segments();
        assert!(!segments.is_empty());
        
        let segment = &segments[0];
        assert!(segment.prosody.rate_multiplier > 1.0);
        assert!(segment.prosody.pitch_multiplier > 1.0);
    }

    #[test]
    fn test_parse_emphasis() {
        let ssml = "<speak><emphasis level=\"strong\">Important</emphasis></speak>";
        let doc = SsmlParser::parse(ssml).unwrap();
        let segments = doc.to_synthesis_segments();
        assert!(!segments.is_empty());
        
        let segment = &segments[0];
        assert!(segment.prosody.emphasis > 0.0);
    }

    #[test]
    fn test_is_ssml() {
        assert!(SsmlParser::is_ssml("<speak>Hello</speak>"));
        assert!(SsmlParser::is_ssml("<?xml version=\"1.0\"?><speak>Hello</speak>"));
        assert!(!SsmlParser::is_ssml("Hello world"));
    }

    #[test]
    fn test_duration_parsing() {
        assert_eq!(parse_duration("500ms"), 500);
        assert_eq!(parse_duration("1s"), 1000);
        assert_eq!(parse_duration("2.5s"), 2500);
    }

    #[test]
    fn test_rate_parsing() {
        assert!((parse_rate("fast") - 1.25).abs() < 0.01);
        assert!((parse_rate("150%") - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_nested_elements() {
        let ssml = r#"
            <speak>
                <p>First paragraph.</p>
                <p><emphasis level="strong">Important</emphasis> text.</p>
            </speak>
        "#;
        let doc = SsmlParser::parse(ssml).unwrap();
        let text = doc.to_plain_text();
        assert!(text.contains("First"));
        assert!(text.contains("Important"));
    }
}
