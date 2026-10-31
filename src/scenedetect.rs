use serde::{Deserialize, Serialize};

/// 64-bin normalized RGB/HSV color histogram for ultra-fast shot boundary detection (`PySceneDetect` parity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameHistogram {
    pub frame_idx: usize,
    pub timestamp_sec: f64,
    pub bins: [f32; 64],
    pub mean_luma: f32,
}

impl FrameHistogram {
    /// Computes a 64-bin (4x4x4 RGB quantized) normalized histogram and BT.709 mean luma from an RGB24 frame buffer
    pub fn from_rgb24(frame_idx: usize, timestamp_sec: f64, rgb: &[u8]) -> Self {
        let mut counts = [0u32; 64];
        let mut luma_sum = 0.0f64;
        let pixel_count = rgb.len() / 3;

        if pixel_count == 0 {
            return Self {
                frame_idx,
                timestamp_sec,
                bins: [0.0; 64],
                mean_luma: 0.0,
            };
        }

        for px in rgb.chunks_exact(3) {
            let r = px[0];
            let g = px[1];
            let b = px[2];
            let bin = (((r >> 6) as usize) << 4) | (((g >> 6) as usize) << 2) | ((b >> 6) as usize);
            counts[bin & 63] += 1;
            luma_sum += 0.2126 * (r as f64) + 0.7152 * (g as f64) + 0.0722 * (b as f64);
        }

        let inv_n = 1.0 / (pixel_count as f32);
        let mut bins = [0.0f32; 64];
        for (dst, &c) in bins.iter_mut().zip(counts.iter()) {
            *dst = (c as f32) * inv_n;
        }

        Self {
            frame_idx,
            timestamp_sec,
            bins,
            mean_luma: (luma_sum / (pixel_count as f64)) as f32,
        }
    }

    /// Manhattan (L1) histogram delta distance in `[0.0, 100.0]`
    pub fn delta_score(&self, prev: &Self) -> f32 {
        let l1: f32 = self
            .bins
            .iter()
            .zip(prev.bins.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        (l1 * 50.0).clamp(0.0, 100.0)
    }

    /// Cosine visual similarity in `[0.0, 1.0]` for `Netflix/matchcut` visual continuity scoring
    pub fn cosine_similarity(&self, other: &Self) -> f32 {
        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;
        for (&a, &b) in self.bins.iter().zip(other.bins.iter()) {
            dot += a * b;
            norm_a += a * a;
            norm_b += b * b;
        }
        let denom = (norm_a.sqrt() * norm_b.sqrt()).max(1e-8);
        (dot / denom).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneBoundary {
    pub cut_frame: usize,
    pub timestamp_sec: f64,
    pub delta_score: f32,
    pub detector_kind: String,
}

pub struct SceneDetector;

impl SceneDetector {
    /// Content-aware cut detector (`Breakthrough/PySceneDetect` `ContentDetector` parity)
    pub fn detect_content_cuts(
        histograms: &[FrameHistogram],
        threshold: f32,
        min_scene_len_frames: usize,
    ) -> Vec<SceneBoundary> {
        let mut cuts = Vec::new();
        let mut last_cut_frame: Option<usize> = None;

        for pair in histograms.windows(2) {
            let prev = &pair[0];
            let curr = &pair[1];
            let score = curr.delta_score(prev);

            let enough_gap = match last_cut_frame {
                Some(last) => curr.frame_idx.saturating_sub(last) >= min_scene_len_frames,
                None => curr.frame_idx >= min_scene_len_frames,
            };

            if score >= threshold && enough_gap {
                cuts.push(SceneBoundary {
                    cut_frame: curr.frame_idx,
                    timestamp_sec: curr.timestamp_sec,
                    delta_score: score,
                    detector_kind: "content_hsv_l1".to_string(),
                });
                last_cut_frame = Some(curr.frame_idx);
            }
        }

        cuts
    }

    /// Adaptive rolling-window cut detector (`PySceneDetect` `AdaptiveDetector` parity)
    pub fn detect_adaptive_cuts(
        histograms: &[FrameHistogram],
        window_radius: usize,
        adaptive_ratio: f32,
        min_delta: f32,
    ) -> Vec<SceneBoundary> {
        if histograms.len() < 2 {
            return Vec::new();
        }

        let deltas: Vec<f32> = histograms
            .windows(2)
            .map(|w| w[1].delta_score(&w[0]))
            .collect();

        let mut cuts = Vec::new();
        let r = window_radius.max(1);

        for (i, &d) in deltas.iter().enumerate() {
            let start = i.saturating_sub(r);
            let end = (i + r + 1).min(deltas.len());
            let sum: f32 = deltas[start..end].iter().sum();
            let mean = (sum - d).max(0.0) / ((end - start - 1).max(1) as f32);

            if d >= min_delta && d >= mean * adaptive_ratio {
                let curr = &histograms[i + 1];
                cuts.push(SceneBoundary {
                    cut_frame: curr.frame_idx,
                    timestamp_sec: curr.timestamp_sec,
                    delta_score: d,
                    detector_kind: "adaptive_rolling".to_string(),
                });
            }
        }

        cuts
    }

    /// Threshold fade-in / fade-out detector (`PySceneDetect` `ThresholdDetector` parity)
    pub fn detect_luma_fades(histograms: &[FrameHistogram], black_threshold: f32) -> Vec<SceneBoundary> {
        let mut fades = Vec::new();
        for pair in histograms.windows(2) {
            let prev = &pair[0];
            let curr = &pair[1];
            let crossed_into_black = prev.mean_luma >= black_threshold && curr.mean_luma < black_threshold;
            let crossed_out_of_black = prev.mean_luma < black_threshold && curr.mean_luma >= black_threshold;

            if crossed_into_black || crossed_out_of_black {
                fades.push(SceneBoundary {
                    cut_frame: curr.frame_idx,
                    timestamp_sec: curr.timestamp_sec,
                    delta_score: (curr.mean_luma - prev.mean_luma).abs(),
                    detector_kind: if crossed_into_black {
                        "fade_out".to_string()
                    } else {
                        "fade_in".to_string()
                    },
                });
            }
        }
        fades
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_and_adaptive_scene_detection() {
        let dark_red = [200u8, 10, 10].repeat(64);
        let bright_blue = [10u8, 20, 240].repeat(64);

        let mut hists = Vec::new();
        for f in 0..10 {
            let buf = if f < 5 { &dark_red } else { &bright_blue };
            hists.push(FrameHistogram::from_rgb24(f, f as f64 / 30.0, buf));
        }

        let cuts = SceneDetector::detect_content_cuts(&hists, 25.0, 2);
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].cut_frame, 5);

        let adaptive = SceneDetector::detect_adaptive_cuts(&hists, 2, 2.0, 15.0);
        assert_eq!(adaptive.len(), 1);
        assert_eq!(adaptive[0].cut_frame, 5);
    }
}
