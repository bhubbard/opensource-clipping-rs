use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeaserConfig {
    pub duration_seconds: f64,
    pub hook_title: String,
    pub overlay_opacity: f64,
    pub enable_countdown: bool,
}

impl Default for TeaserConfig {
    fn default() -> Self {
        Self {
            duration_seconds: 3.0,
            hook_title: "WAIT FOR IT...".to_string(),
            overlay_opacity: 0.45,
            enable_countdown: true,
        }
    }
}

pub struct TeaserEngine;

impl TeaserEngine {
    /// Builds FFmpeg filtergraph for a cinematic 3-second hook teaser:
    /// - Slow zoom in (ken burns effect)
    /// - Dark contrast overlay
    /// - Bold title banner at top
    pub fn build_teaser_filter(config: &TeaserConfig) -> String {
        let dur = config.duration_seconds;
        let opacity = config.overlay_opacity;

        // Zoompan: gentle zoom from 1.0 to 1.15 over the duration
        // Followed by drawtext for hook headline
        let mut filter = format!(
            "zoompan=z='min(zoom+0.0015,1.15)':d={}:s=1080x1920:fps=30,",
            (dur * 30.0) as i64
        );

        // Vignette & color curves for cinematic tension
        filter.push_str(&format!(
            "vignette=PI/4,drawbox=y=0:color=black@{}:width=iw:height=ih:t=fill,",
            opacity
        ));

        // Hook headline banner
        let safe_title = config.hook_title.replace('\'', "\\'").replace(':', "\\:");
        filter.push_str(&format!(
            "drawtext=text='{}':fontcolor=white:fontsize=64:box=1:boxcolor=black@0.6:boxborderw=20:x=(w-text_w)/2:y=300",
            safe_title
        ));

        filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teaser_filter_builder() {
        let cfg = TeaserConfig {
            duration_seconds: 3.0,
            hook_title: "SHOCKING TRUTH".to_string(),
            overlay_opacity: 0.5,
            enable_countdown: true,
        };

        let filter = TeaserEngine::build_teaser_filter(&cfg);
        assert!(filter.contains("zoompan="));
        assert!(filter.contains("vignette="));
        assert!(filter.contains("SHOCKING TRUTH"));
    }
}
