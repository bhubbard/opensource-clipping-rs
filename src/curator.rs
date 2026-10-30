use crate::transcript::Transcript;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViralCandidate {
    pub start_time: f64,
    pub end_time: f64,
    pub duration: f64,
    pub hook_text: String,
    pub virality_score: f64,
    pub reason: String,
}

pub struct ContentCurator;

impl ContentCurator {
    /// Heuristic virality scoring based on question hooks, curiosity triggers, power words,
    /// and speech density (WPM).
    pub fn score_segment(text: &str, duration_sec: f64) -> (f64, String) {
        if duration_sec <= 0.0 || text.is_empty() {
            return (0.0, "Empty content".to_string());
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        let wpm = (word_count as f64 / duration_sec) * 60.0;

        let mut score = 40.0; // base score
        let mut reasons = Vec::new();

        let lower = text.to_lowercase();

        // 1. Question hook
        if lower.contains('?')
            || lower.starts_with("why")
            || lower.starts_with("how")
            || lower.starts_with("what if")
            || lower.starts_with("did you know")
        {
            score += 20.0;
            reasons.push("Strong question hook");
        }

        // 2. Curiosity & power words
        let power_words = [
            "secret", "insane", "crazy", "never", "always", "mistake", "truth", "unbelievable",
            "warning", "money", "rich", "stop", "hack", "trick", "shocking", "millionaire",
        ];

        let matched_power_words = power_words.iter().filter(|&&pw| lower.contains(pw)).count();
        if matched_power_words > 0 {
            score += (matched_power_words as f64 * 6.0).min(25.0);
            reasons.push("High-arousal curiosity triggers");
        }

        // 3. Optimal speech density for short-form retention (140 - 190 WPM)
        if (130.0..=200.0).contains(&wpm) {
            score += 15.0;
            reasons.push("Optimal pacing for TikTok/Shorts (140-190 WPM)");
        } else if wpm < 100.0 {
            score -= 10.0;
            reasons.push("Slow speech cadence");
        }

        // 4. Punchy length
        if (15.0..=45.0).contains(&duration_sec) {
            score += 10.0;
            reasons.push("Prime viral duration window (15-45s)");
        }

        let final_score = score.clamp(0.0, 100.0);
        let explanation = if reasons.is_empty() {
            "Standard conversational segment".to_string()
        } else {
            reasons.join("; ")
        };

        (final_score, explanation)
    }

    /// Scans a transcript with a sliding window to extract top viral candidates
    pub fn find_viral_candidates(
        transcript: &Transcript,
        min_duration: f64,
        max_duration: f64,
        top_k: usize,
    ) -> Vec<ViralCandidate> {
        let mut candidates = Vec::new();
        let segments = &transcript.segments;

        if segments.is_empty() {
            return candidates;
        }

        for i in 0..segments.len() {
            let start_time = segments[i].start;
            let mut accumulated_text = String::new();

            for seg in segments.iter().skip(i) {
                let end_time = seg.end;
                let duration = end_time - start_time;

                if !accumulated_text.is_empty() {
                    accumulated_text.push(' ');
                }
                accumulated_text.push_str(&seg.text);

                if duration >= min_duration && duration <= max_duration {
                    let (score, reason) = Self::score_segment(&accumulated_text, duration);
                    let hook_text = segments[i].text.clone();

                    candidates.push(ViralCandidate {
                        start_time,
                        end_time,
                        duration,
                        hook_text,
                        virality_score: score,
                        reason,
                    });
                }

                if duration > max_duration {
                    break;
                }
            }
        }

        // Sort descending by virality score
        candidates.sort_by(|a, b| b.virality_score.partial_cmp(&a.virality_score).unwrap());

        // Deduplicate overlapping candidate windows
        let mut filtered: Vec<ViralCandidate> = Vec::new();
        for c in candidates {
            let overlaps = filtered.iter().any(|existing| {
                let start_max = existing.start_time.max(c.start_time);
                let end_min = existing.end_time.min(c.end_time);
                (end_min - start_max) > 5.0 // more than 5s overlap
            });

            if !overlaps {
                filtered.push(c);
            }
            if filtered.len() >= top_k {
                break;
            }
        }

        filtered
    }

    /// Generate structured prompt for Google Gemini / OpenAI LLM analysis
    pub fn build_llm_curation_prompt(transcript: &Transcript) -> String {
        format!(
            r#"You are an expert viral content producer for YouTube Shorts and TikTok.
Analyze the following transcript with timestamps and extract the top 3-5 most engaging, high-retention highlights (15-60 seconds each).

Transcript:
{}

Respond with a strict JSON array of objects adhering to this schema:
[
  {{
    "start_time": 12.5,
    "end_time": 45.0,
    "hook_text": "Did you know that 90% of people make this mistake?",
    "virality_score": 95,
    "reason": "Starts with an irresistible curiosity gap and maintains high energy"
  }}
]
Do not output markdown codeblocks, only valid JSON."#,
            transcript.full_text()
        )
    }

    /// Parse LLM curation output
    pub fn parse_llm_candidates(json_response: &str) -> anyhow::Result<Vec<ViralCandidate>> {
        let cleaned = json_response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let candidates: Vec<ViralCandidate> = serde_json::from_str(cleaned)?;
        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcript::TranscriptSegment;

    #[test]
    fn test_score_segment_with_question_and_power_words() {
        let text = "Did you know the secret mistake everyone makes with money?";
        let (score, reasons) = ContentCurator::score_segment(text, 20.0);
        assert!(score >= 60.0);
        assert!(reasons.contains("Strong question hook"));
        assert!(reasons.contains("curiosity triggers"));
    }

    #[test]
    fn test_find_viral_candidates() {
        let segments = vec![
            TranscriptSegment {
                id: 0,
                start: 0.0,
                end: 10.0,
                text: "Hello everyone welcome to the podcast.".to_string(),
                words: vec![],
                speaker: None,
            },
            TranscriptSegment {
                id: 1,
                start: 10.0,
                end: 30.0,
                text: "Why does everyone fall for this crazy secret money hack?".to_string(),
                words: vec![],
                speaker: None,
            },
            TranscriptSegment {
                id: 2,
                start: 30.0,
                end: 45.0,
                text: "Here is the exact truth about how it works.".to_string(),
                words: vec![],
                speaker: None,
            },
        ];

        let transcript = Transcript::new(segments);
        let candidates = ContentCurator::find_viral_candidates(&transcript, 15.0, 40.0, 3);

        assert!(!candidates.is_empty());
        assert!(candidates[0].virality_score >= 60.0);
    }
}
