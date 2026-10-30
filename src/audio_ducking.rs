use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDuckingConfig {
    pub bgm_volume: f64,        // e.g. 0.25 (background volume)
    pub ducking_threshold: f64, // sidechain threshold e.g. 0.12
    pub ducking_ratio: f64,     // compression ratio e.g. 6.0
    pub attack_ms: f64,         // attack time in ms e.g. 50.0
    pub release_ms: f64,        // release time in ms e.g. 300.0
    pub target_lufs: f64,       // integrated loudness e.g. -16.0 for YouTube Shorts/TikTok
}

impl Default for AudioDuckingConfig {
    fn default() -> Self {
        Self {
            bgm_volume: 0.20,
            ducking_threshold: 0.12,
            ducking_ratio: 5.0,
            attack_ms: 50.0,
            release_ms: 300.0,
            target_lufs: -16.0,
        }
    }
}

pub struct AudioDuckingEngine;

impl AudioDuckingEngine {
    /// Builds the FFmpeg audio filtergraph that sidechains the speech stream (input 0:a)
    /// against the background music stream (input 1:a) and mixes them together.
    pub fn build_ducking_filtergraph(config: &AudioDuckingConfig) -> String {
        format!(
            "[1:a]volume={:.2}[bgm_vol];\
             [bgm_vol][0:a]sidechaincompress=threshold={:.2}:ratio={:.1}:attack={:.1}:release={:.1}:makeup=1[ducked_bgm];\
             [0:a][ducked_bgm]amix=inputs=2:duration=first:dropout_transition=2,loudnorm=I={:.1}:TP=-1.5:LRA=11[outa]",
            config.bgm_volume,
            config.ducking_threshold,
            config.ducking_ratio,
            config.attack_ms,
            config.release_ms,
            config.target_lufs
        )
    }

    /// Single-track speech audio normalization filtergraph
    pub fn build_speech_norm_filtergraph(target_lufs: f64) -> String {
        format!("loudnorm=I={:.1}:TP=-1.5:LRA=11[outa]", target_lufs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ducking_filtergraph() {
        let cfg = AudioDuckingConfig::default();
        let fg = AudioDuckingEngine::build_ducking_filtergraph(&cfg);
        assert!(fg.contains("sidechaincompress="));
        assert!(fg.contains("amix=inputs=2"));
        assert!(fg.contains("loudnorm=I=-16.0"));
    }
}
