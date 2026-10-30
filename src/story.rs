use crate::curator::ViralCandidate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryClipPlan {
    pub hook_clips: Vec<ViralCandidate>,
    pub main_climax_clip: ViralCandidate,
    pub total_estimated_duration: f64,
}

pub struct StoryEngine;

impl StoryEngine {
    /// Assembles a story clip plan from a list of viral candidates:
    /// Picks 2-3 short micro-hooks (1.5-3.0s teaser windows) before the main viral segment.
    pub fn assemble_story(candidates: &[ViralCandidate]) -> Option<StoryClipPlan> {
        if candidates.is_empty() {
            return None;
        }

        // Top scored candidate is the main climax clip
        let main = candidates[0].clone();

        // Secondary candidates form the multi-hook intro
        let hook_clips: Vec<ViralCandidate> = candidates
            .iter()
            .skip(1)
            .take(3)
            .map(|c| {
                let teaser_dur = 2.5;
                ViralCandidate {
                    start_time: c.start_time,
                    end_time: c.start_time + teaser_dur,
                    duration: teaser_dur,
                    hook_text: c.hook_text.clone(),
                    virality_score: c.virality_score,
                    reason: "Story teaser micro-hook".to_string(),
                }
            })
            .collect();

        let hooks_dur: f64 = hook_clips.iter().map(|h| h.duration).sum();
        let total = hooks_dur + main.duration;

        Some(StoryClipPlan {
            hook_clips,
            main_climax_clip: main,
            total_estimated_duration: total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_story() {
        let c1 = ViralCandidate {
            start_time: 10.0,
            end_time: 40.0,
            duration: 30.0,
            hook_text: "Climax".to_string(),
            virality_score: 95.0,
            reason: "".to_string(),
        };
        let c2 = ViralCandidate {
            start_time: 50.0,
            end_time: 70.0,
            duration: 20.0,
            hook_text: "Hook 2".to_string(),
            virality_score: 85.0,
            reason: "".to_string(),
        };

        let story = StoryEngine::assemble_story(&[c1, c2]).unwrap();
        assert_eq!(story.hook_clips.len(), 1);
        assert_eq!(story.hook_clips[0].duration, 2.5);
        assert_eq!(story.main_climax_clip.duration, 30.0);
    }
}
