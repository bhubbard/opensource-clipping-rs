use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoundingBox {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn center_x(&self) -> f64 {
        self.x + (self.width / 2.0)
    }

    pub fn center_y(&self) -> f64 {
        self.y + (self.height / 2.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CropWindow {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl CropWindow {
    /// Format as FFmpeg crop filter: `crop=w:h:x:y`
    pub fn to_ffmpeg_crop(&self) -> String {
        format!("crop={}:{}:{}:{}", self.width, self.height, self.x, self.y)
    }
}

#[derive(Debug, Clone)]
pub struct FaceTrackerSmoother {
    pub alpha: f64,
    current_center_x: Option<f64>,
}

impl FaceTrackerSmoother {
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.01, 1.0),
            current_center_x: None,
        }
    }

    /// Updates smoothed center position with Exponential Moving Average (EMA)
    pub fn update(&mut self, target_center_x: f64) -> f64 {
        match self.current_center_x {
            Some(curr) => {
                let smoothed = curr + self.alpha * (target_center_x - curr);
                self.current_center_x = Some(smoothed);
                smoothed
            }
            None => {
                self.current_center_x = Some(target_center_x);
                target_center_x
            }
        }
    }
}

pub struct FramingEngine;

impl FramingEngine {
    /// Calculates the 9:16 crop window from a 16:9 source (e.g. 1920x1080) centered around
    /// a speaker's face X coordinate.
    pub fn calculate_9_16_crop(
        source_width: i32,
        source_height: i32,
        face_center_x: f64,
    ) -> CropWindow {
        // In 9:16 target, target_width = (source_height * 9) / 16
        // For 1080p height: 1080 * 9 / 16 = 607.5 -> 608 (even number)
        let crop_height = source_height;
        let mut crop_width = ((source_height as f64 * 9.0) / 16.0).round() as i32;
        if crop_width % 2 != 0 {
            crop_width += 1;
        }

        // Center the crop window at face_center_x
        let ideal_x = (face_center_x - (crop_width as f64 / 2.0)).round() as i32;

        // Clamp inside frame boundaries
        let max_x = source_width - crop_width;
        let clamped_x = ideal_x.clamp(0, max_x.max(0));

        CropWindow {
            x: clamped_x,
            y: 0,
            width: crop_width,
            height: crop_height,
        }
    }

    /// Builds FFmpeg filter for podcast split-screen mode:
    /// Crops speaker 1 (top half) and speaker 2 (bottom half) and stacks them into a 1080x1920 9:16 canvas.
    pub fn build_split_screen_filter(
        source_width: i32,
        source_height: i32,
        speaker1_center_x: f64,
        speaker2_center_x: f64,
    ) -> String {
        // Half height for each speaker is 960 (out of 1920)
        // 960 * 16 / 9 aspect or crop from source
        let crop1 = Self::calculate_9_16_crop(source_width, source_height, speaker1_center_x);
        let crop2 = Self::calculate_9_16_crop(source_width, source_height, speaker2_center_x);

        format!(
            "[0:v]{},scale=1080:960[top];[0:v]{},scale=1080:960[bottom];[top][bottom]vstack=inputs=2[outv]",
            crop1.to_ffmpeg_crop(),
            crop2.to_ffmpeg_crop()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crop_calculation_centered() {
        // 1920x1080, face right in the middle at 960.0
        let crop = FramingEngine::calculate_9_16_crop(1920, 1080, 960.0);
        assert_eq!(crop.height, 1080);
        assert_eq!(crop.width, 608);
        assert_eq!(crop.x, 960 - 304);
        assert_eq!(crop.y, 0);
    }

    #[test]
    fn test_crop_boundary_clamping() {
        // Face near left edge
        let crop_left = FramingEngine::calculate_9_16_crop(1920, 1080, 50.0);
        assert_eq!(crop_left.x, 0);

        // Face near right edge
        let crop_right = FramingEngine::calculate_9_16_crop(1920, 1080, 1900.0);
        assert_eq!(crop_right.x, 1920 - 608);
    }

    #[test]
    fn test_face_tracker_smoother() {
        let mut smoother = FaceTrackerSmoother::new(0.2);
        let p1 = smoother.update(100.0);
        assert_eq!(p1, 100.0);

        let p2 = smoother.update(200.0);
        assert_eq!(p2, 100.0 + 0.2 * 100.0); // 120.0
    }
}
