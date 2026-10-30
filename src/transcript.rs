use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordTiming {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub probability: Option<f64>,
}

impl WordTiming {
    pub fn new(word: impl Into<String>, start: f64, end: f64) -> Self {
        Self {
            word: word.into(),
            start,
            end,
            probability: None,
        }
    }

    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }

    /// Duration in centiseconds (1/100th of a second), used by ASS \k tags
    pub fn duration_centiseconds(&self) -> i64 {
        (self.duration() * 100.0).round() as i64
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub id: usize,
    pub start: f64,
    pub end: f64,
    pub text: String,
    #[serde(default)]
    pub words: Vec<WordTiming>,
    pub speaker: Option<String>,
}

impl TranscriptSegment {
    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Transcript {
    pub segments: Vec<TranscriptSegment>,
    pub language: Option<String>,
}

impl Transcript {
    pub fn new(segments: Vec<TranscriptSegment>) -> Self {
        Self {
            segments,
            language: None,
        }
    }

    /// Parse Whisper JSON transcript (faster-whisper, whisper.cpp, or OpenAI format)
    pub fn from_whisper_json(json_str: &str) -> anyhow::Result<Self> {
        #[derive(Deserialize)]
        struct RawWord {
            word: Option<String>,
            text: Option<String>,
            start: f64,
            end: f64,
            probability: Option<f64>,
        }

        #[derive(Deserialize)]
        struct RawSegment {
            #[serde(default)]
            id: usize,
            start: f64,
            end: f64,
            text: String,
            #[serde(default)]
            words: Vec<RawWord>,
            speaker: Option<String>,
        }

        #[derive(Deserialize)]
        struct RawTranscript {
            segments: Vec<RawSegment>,
            language: Option<String>,
        }

        let raw: RawTranscript = serde_json::from_str(json_str)?;

        let segments = raw
            .segments
            .into_iter()
            .map(|s| {
                let words = s
                    .words
                    .into_iter()
                    .map(|w| WordTiming {
                        word: w.word.or(w.text).unwrap_or_default(),
                        start: w.start,
                        end: w.end,
                        probability: w.probability,
                    })
                    .collect();

                TranscriptSegment {
                    id: s.id,
                    start: s.start,
                    end: s.end,
                    text: s.text.trim().to_string(),
                    words,
                    speaker: s.speaker,
                }
            })
            .collect();

        Ok(Transcript {
            segments,
            language: raw.language,
        })
    }

    /// Parse SubRip (.srt) format
    pub fn from_srt(srt_str: &str) -> anyhow::Result<Self> {
        let mut segments = Vec::new();
        let blocks = srt_str.trim().split("\n\n");

        for (idx, block) in blocks.enumerate() {
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() < 2 {
                continue;
            }

            let time_line = if lines[0].contains("-->") { lines[0] } else { lines[1] };
            let text_lines = if lines[0].contains("-->") { &lines[1..] } else { &lines[2..] };

            if let Some((start_s, end_s)) = time_line.split_once("-->") {
                let start = parse_srt_timestamp(start_s.trim())?;
                let end = parse_srt_timestamp(end_s.trim())?;
                let text = text_lines.join(" ").trim().to_string();

                // Approximate word timings evenly if not present
                let raw_words: Vec<&str> = text.split_whitespace().collect();
                let word_count = raw_words.len();
                let seg_dur = end - start;
                let word_dur = if word_count > 0 { seg_dur / word_count as f64 } else { 0.0 };

                let words = raw_words
                    .into_iter()
                    .enumerate()
                    .map(|(w_idx, w)| WordTiming {
                        word: w.to_string(),
                        start: start + (w_idx as f64 * word_dur),
                        end: start + ((w_idx + 1) as f64 * word_dur),
                        probability: None,
                    })
                    .collect();

                segments.push(TranscriptSegment {
                    id: idx,
                    start,
                    end,
                    text,
                    words,
                    speaker: None,
                });
            }
        }

        Ok(Transcript {
            segments,
            language: None,
        })
    }

    /// Extract a sub-transcript within the given time span `[start_sec, end_sec]`
    pub fn slice_by_time(&self, start_sec: f64, end_sec: f64) -> Transcript {
        let mut sliced_segments = Vec::new();

        for seg in &self.segments {
            if seg.end >= start_sec && seg.start <= end_sec {
                // Filter words that fall inside window
                let words: Vec<WordTiming> = seg
                    .words
                    .iter()
                    .filter(|w| w.end >= start_sec && w.start <= end_sec)
                    .cloned()
                    .collect();

                sliced_segments.push(TranscriptSegment {
                    id: seg.id,
                    start: seg.start.max(start_sec),
                    end: seg.end.min(end_sec),
                    text: seg.text.clone(),
                    words,
                    speaker: seg.speaker.clone(),
                });
            }
        }

        Transcript {
            segments: sliced_segments,
            language: self.language.clone(),
        }
    }

    pub fn total_words(&self) -> usize {
        self.segments.iter().map(|s| s.words.len()).sum()
    }

    pub fn full_text(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn parse_srt_timestamp(ts: &str) -> anyhow::Result<f64> {
    let clean = ts.replace(',', ".");
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid timestamp: {}", ts);
    }
    let h: f64 = parts[0].parse()?;
    let m: f64 = parts[1].parse()?;
    let s: f64 = parts[2].parse()?;
    Ok(h * 3600.0 + m * 60.0 + s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_timing_centiseconds() {
        let w = WordTiming::new("awesome", 1.20, 1.55);
        assert_eq!(w.duration_centiseconds(), 35);
    }

    #[test]
    fn test_parse_srt() {
        let srt = "1\n00:00:01,000 --> 00:00:03,000\nThis is a viral short.\n\n2\n00:00:03,500 --> 00:00:05,000\nSubscribe for more.";
        let transcript = Transcript::from_srt(srt).unwrap();
        assert_eq!(transcript.segments.len(), 2);
        assert_eq!(transcript.segments[0].start, 1.0);
        assert_eq!(transcript.segments[0].end, 3.0);
        assert_eq!(transcript.segments[0].words.len(), 5);
    }

    #[test]
    fn test_slice_by_time() {
        let srt = "1\n00:00:00,000 --> 00:00:05,000\nIntro scene\n\n2\n00:00:10,000 --> 00:00:20,000\nViral climax\n\n3\n00:00:25,000 --> 00:00:30,000\nOutro";
        let transcript = Transcript::from_srt(srt).unwrap();
        let sliced = transcript.slice_by_time(8.0, 22.0);
        assert_eq!(sliced.segments.len(), 1);
        assert_eq!(sliced.segments[0].text, "Viral climax");
    }
}
