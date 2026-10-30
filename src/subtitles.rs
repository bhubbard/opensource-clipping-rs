use crate::transcript::{Transcript, WordTiming};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleStyle {
    pub font_name: String,
    pub font_size: u32,
    pub primary_color: String,   // ASS format &H00FFFFFF& (White)
    pub highlight_color: String, // ASS format &H0000FFFF& (Yellow)
    pub outline_color: String,   // ASS format &H00000000& (Black)
    pub back_color: String,      // Shadow/Box color
    pub outline_width: f64,
    pub shadow_depth: f64,
    pub alignment: u32,          // 2 = bottom-center, 5 = center
    pub margin_v: u32,
}

impl Default for SubtitleStyle {
    fn default() -> Self {
        Self {
            font_name: "Montserrat ExtraBold".to_string(),
            font_size: 72,
            primary_color: "&H00FFFFFF&".to_string(),   // White
            highlight_color: "&H0000E6FF&".to_string(), // Vivid Golden Yellow
            outline_color: "&H00000000&".to_string(),   // Black outline
            back_color: "&H80000000&".to_string(),      // Semi-transparent black shadow
            outline_width: 4.5,
            shadow_depth: 3.0,
            alignment: 2,                               // Bottom center
            margin_v: 240,                              // Raised up for TikTok UI bar
        }
    }
}

pub struct SubtitleEngine;

impl SubtitleEngine {
    /// Format timestamp into ASS format: `H:MM:SS.CC` (centiseconds)
    pub fn format_ass_time(seconds: f64) -> String {
        let total_cs = (seconds * 100.0).round() as i64;
        let cs = total_cs % 100;
        let total_s = total_cs / 100;
        let s = total_s % 60;
        let total_m = total_s / 60;
        let m = total_m % 60;
        let h = total_m / 60;

        format!("{}:{:02}:{:02}.{:02}", h, m, s, cs)
    }

    /// Generate full `.ass` karaoke subtitle file content
    pub fn generate_karaoke_ass(
        transcript: &Transcript,
        clip_start: f64,
        clip_end: f64,
        style: &SubtitleStyle,
        max_words_per_line: usize,
    ) -> String {
        let mut ass = String::new();

        // 1. Script Info
        ass.push_str("[Script Info]\n");
        ass.push_str("Title: OpenSource Clipping Kinetic Karaoke Subtitles\n");
        ass.push_str("ScriptType: v4.00+\n");
        ass.push_str("WrapStyle: 0\n");
        ass.push_str("ScaledBorderAndShadow: yes\n");
        ass.push_str("PlayResX: 1080\n");
        ass.push_str("PlayResY: 1920\n\n");

        // 2. Styles
        ass.push_str("[V4+ Styles]\n");
        ass.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
        ass.push_str(&format!(
            "Style: Default,{},{},{},{},{},{},1,0,0,0,100,100,0,0,1,{:.1},{:.1},{},40,40,{},1\n\n",
            style.font_name,
            style.font_size,
            style.primary_color,
            style.highlight_color,
            style.outline_color,
            style.back_color,
            style.outline_width,
            style.shadow_depth,
            style.alignment,
            style.margin_v
        ));

        // 3. Events
        ass.push_str("[Events]\n");
        ass.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

        // Extract words within clip window and offset by clip_start
        let mut words: Vec<WordTiming> = Vec::new();
        for seg in &transcript.segments {
            for w in &seg.words {
                if w.end >= clip_start && w.start <= clip_end {
                    let mut rel_word = w.clone();
                    rel_word.start = (rel_word.start - clip_start).max(0.0);
                    rel_word.end = (rel_word.end - clip_start).max(0.0);
                    words.push(rel_word);
                }
            }
        }

        if words.is_empty() {
            return ass;
        }

        // Group into lines of `max_words_per_line` words
        for chunk in words.chunks(max_words_per_line) {
            let line_start = chunk.first().unwrap().start;
            let line_end = chunk.last().unwrap().end;

            let start_str = Self::format_ass_time(line_start);
            let end_str = Self::format_ass_time(line_end);

            let mut line_text = String::new();
            for word in chunk {
                let cs = word.duration_centiseconds().max(1);
                // ASS karaoke tag \k with kinetic scale pop
                line_text.push_str(&format!(
                    "{{\\k{}\\t(0,70,\\fscx110\\fscy110)\\t(70,140,\\fscx100\\fscy100)}}{} ",
                    cs,
                    word.word.to_uppercase()
                ));
            }

            ass.push_str(&format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                start_str,
                end_str,
                line_text.trim()
            ));
        }

        ass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcript::TranscriptSegment;

    #[test]
    fn test_format_ass_time() {
        assert_eq!(SubtitleEngine::format_ass_time(0.0), "0:00:00.00");
        assert_eq!(SubtitleEngine::format_ass_time(65.42), "0:01:05.42");
        assert_eq!(SubtitleEngine::format_ass_time(3661.05), "1:01:01.05");
    }

    #[test]
    fn test_generate_karaoke_ass() {
        let seg = TranscriptSegment {
            id: 0,
            start: 1.0,
            end: 4.0,
            text: "This is pure gold".to_string(),
            words: vec![
                WordTiming::new("This", 1.0, 1.5),
                WordTiming::new("is", 1.5, 2.0),
                WordTiming::new("pure", 2.0, 2.8),
                WordTiming::new("gold", 2.8, 3.5),
            ],
            speaker: None,
        };
        let transcript = Transcript::new(vec![seg]);
        let style = SubtitleStyle::default();
        let ass = SubtitleEngine::generate_karaoke_ass(&transcript, 1.0, 4.0, &style, 4);

        assert!(ass.contains("[Script Info]"));
        assert!(ass.contains("[V4+ Styles]"));
        assert!(ass.contains("[Events]"));
        assert!(ass.contains("Dialogue: 0,0:00:00.00,0:00:02.50,Default"));
        assert!(ass.contains("GOLD"));
    }
}
