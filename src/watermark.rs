use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatermarkPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatermarkKind {
    Text {
        text: String,
        font_size: u32,
        font_color: String,
    },
    Image {
        path: String,
        width: Option<u32>,
        height: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watermark {
    pub kind: WatermarkKind,
    pub position: WatermarkPosition,
    pub opacity: f64,
    pub margin_x: u32,
    pub margin_y: u32,
}

impl Watermark {
    pub fn new_text(text: impl Into<String>, position: WatermarkPosition) -> Self {
        Self {
            kind: WatermarkKind::Text {
                text: text.into(),
                font_size: 36,
                font_color: "white".to_string(),
            },
            position,
            opacity: 0.8,
            margin_x: 40,
            margin_y: 60,
        }
    }

    /// Build FFmpeg video filter for the watermark
    pub fn to_ffmpeg_filter(&self) -> String {
        let (x_expr, y_expr) = match self.position {
            WatermarkPosition::TopLeft => (format!("{}", self.margin_x), format!("{}", self.margin_y)),
            WatermarkPosition::TopRight => (format!("w-text_w-{}", self.margin_x), format!("{}", self.margin_y)),
            WatermarkPosition::BottomLeft => (format!("{}", self.margin_x), format!("h-text_h-{}", self.margin_y)),
            WatermarkPosition::BottomRight => (format!("w-text_w-{}", self.margin_x), format!("h-text_h-{}", self.margin_y)),
            WatermarkPosition::Center => ("(w-text_w)/2".to_string(), "(h-text_h)/2".to_string()),
        };

        match &self.kind {
            WatermarkKind::Text { text, font_size, font_color } => {
                let safe_text = text.replace('\'', "\\'").replace(':', "\\:");
                format!(
                    "drawtext=text='{}':fontsize={}:fontcolor={}@{:.2}:x={}:y={}",
                    safe_text, font_size, font_color, self.opacity, x_expr, y_expr
                )
            }
            WatermarkKind::Image { .. } => {
                // Overlay position
                let (ox, oy) = match self.position {
                    WatermarkPosition::TopLeft => (format!("{}", self.margin_x), format!("{}", self.margin_y)),
                    WatermarkPosition::TopRight => (format!("W-w-{}", self.margin_x), format!("{}", self.margin_y)),
                    WatermarkPosition::BottomLeft => (format!("{}", self.margin_x), format!("H-h-{}", self.margin_y)),
                    WatermarkPosition::BottomRight => (format!("W-w-{}", self.margin_x), format!("H-h-{}", self.margin_y)),
                    WatermarkPosition::Center => ("(W-w)/2".to_string(), "(H-h)/2".to_string()),
                };
                format!("overlay=x={}:y={}", ox, oy)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_watermark_filter() {
        let wm = Watermark::new_text("@calljacob", WatermarkPosition::TopRight);
        let filter = wm.to_ffmpeg_filter();
        assert!(filter.contains("drawtext=text='@calljacob'"));
        assert!(filter.contains("x=w-text_w-40"));
        assert!(filter.contains("y=60"));
    }
}
