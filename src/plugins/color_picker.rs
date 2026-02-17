use std::sync::Mutex;

use anyhow::Result;

use crate::core::plugin::Plugin;
use crate::core::search::{ActionData, ResultType, SearchResult};

#[derive(Clone, Debug)]
pub struct ColorValue {
    pub hex: String,
    pub rgb: (u8, u8, u8),
    pub hsl: (u16, u8, u8),
    pub name: String,
}

pub struct ColorPickerPlugin {
    enabled: bool,
    parsed_color: Mutex<Option<ColorValue>>,
}

impl ColorPickerPlugin {
    pub fn new() -> Self {
        Self { enabled: true, parsed_color: Mutex::new(None) }
    }

    fn parse_hex(&self, input: &str) -> Option<ColorValue> {
        let hex = input.trim().to_uppercase();

        let hex = if hex.starts_with('#') { hex[1..].to_string() } else { hex };

        let hex = if hex.len() == 3 {
            let chars: Vec<char> = hex.chars().collect();
            format!("{}{}{}{}{}{}", chars[0], chars[0], chars[1], chars[1], chars[2], chars[2])
        } else {
            hex
        };

        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(ColorValue {
            hex: format!("#{}", hex),
            rgb: (r, g, b),
            hsl: self.rgb_to_hsl(r, g, b),
            name: String::new(),
        })
    }

    fn parse_rgb(&self, input: &str) -> Option<ColorValue> {
        let input = input.trim();

        let input = if input.starts_with("rgb(") && input.ends_with(')') {
            &input[4..input.len() - 1]
        } else if input.starts_with("rgb") {
            &input[3..]
        } else {
            return None;
        };

        let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();

        if parts.len() != 3 {
            return None;
        }

        let r: u8 = parts[0].parse().ok()?;
        let g: u8 = parts[1].parse().ok()?;
        let b: u8 = parts[2].parse().ok()?;

        let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);

        Some(ColorValue { hex, rgb: (r, g, b), hsl: self.rgb_to_hsl(r, g, b), name: String::new() })
    }

    fn parse_hsl(&self, input: &str) -> Option<ColorValue> {
        let input = input.trim();

        let input = if input.starts_with("hsl(") && input.ends_with(')') {
            &input[4..input.len() - 1]
        } else if input.starts_with("hsl") {
            &input[3..]
        } else {
            return None;
        };

        let parts: Vec<&str> = input.split(',').map(|s| s.trim().trim_end_matches('%')).collect();

        if parts.len() != 3 {
            return None;
        }

        let h: u16 = parts[0].parse().ok()?;
        let s: u8 = parts[1].parse().ok()?;
        let l: u8 = parts[2].parse().ok()?;

        if h > 360 || s > 100 || l > 100 {
            return None;
        }

        let (r, g, b) = self.hsl_to_rgb(h, s, l);
        let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);

        Some(ColorValue { hex, rgb: (r, g, b), hsl: (h, s, l), name: String::new() })
    }

    fn parse_color(&self, input: &str) -> Option<ColorValue> {
        let input = input.trim();

        if input.starts_with("rgb") {
            return self.parse_rgb(input);
        }

        if input.starts_with("hsl") {
            return self.parse_hsl(input);
        }

        if input.starts_with('#') || input.chars().all(|c| c.is_ascii_hexdigit()) {
            return self.parse_hex(input);
        }

        None
    }

    fn rgb_to_hsl(&self, r: u8, g: u8, b: u8) -> (u16, u8, u8) {
        let r = r as f64 / 255.0;
        let g = g as f64 / 255.0;
        let b = b as f64 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        if (max - min).abs() < f64::EPSILON {
            return (0, 0, (l * 100.0) as u8);
        }

        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

        let h = if (max - r).abs() < f64::EPSILON {
            ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
        } else if (max - g).abs() < f64::EPSILON {
            ((b - r) / d + 2.0) / 6.0
        } else {
            ((r - g) / d + 4.0) / 6.0
        };

        ((h * 360.0) as u16, (s * 100.0) as u8, (l * 100.0) as u8)
    }

    fn hsl_to_rgb(&self, h: u16, s: u8, l: u8) -> (u8, u8, u8) {
        let h = h as f64 / 360.0;
        let s = s as f64 / 100.0;
        let l = l as f64 / 100.0;

        if s.abs() < f64::EPSILON {
            let v = (l * 255.0) as u8;
            return (v, v, v);
        }

        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;

        let r = self.hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = self.hue_to_rgb(p, q, h);
        let b = self.hue_to_rgb(p, q, h - 1.0 / 3.0);

        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }

    fn hue_to_rgb(&self, p: f64, q: f64, mut t: f64) -> f64 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    fn get_color_name(&self, color: &ColorValue) -> String {
        let (r, g, b) = color.rgb;

        if r > 200 && g > 200 && b > 200 {
            return "白色".to_string();
        }
        if r < 50 && g < 50 && b < 50 {
            return "黑色".to_string();
        }
        if r > g && r > b {
            if r > 150 && g < 100 && b < 100 {
                return "红色".to_string();
            }
            return "红色系".to_string();
        }
        if g > r && g > b {
            if g > 150 && r < 100 && b < 100 {
                return "绿色".to_string();
            }
            return "绿色系".to_string();
        }
        if b > r && b > g {
            if b > 150 && r < 100 && g < 100 {
                return "蓝色".to_string();
            }
            return "蓝色系".to_string();
        }

        if r > 150 && g > 150 && b < 100 {
            return "黄色".to_string();
        }
        if r > 150 && g < 100 && b > 150 {
            return "紫色".to_string();
        }
        if r < 100 && g > 150 && b > 150 {
            return "青色".to_string();
        }
        if r > 150 && g > 100 && b < 100 {
            return "橙色".to_string();
        }
        if r > 200 && g > 200 && b > 200 {
            return "白色".to_string();
        }

        "自定义颜色".to_string()
    }
}

impl Plugin for ColorPickerPlugin {
    fn id(&self) -> &str {
        "color_picker"
    }

    fn name(&self) -> &str {
        "颜色选择器"
    }

    fn description(&self) -> &str {
        "识别颜色值并提供预览和复制"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn initialize(&mut self) -> Result<()> {
        log::info!("初始化颜色选择器插件...");
        Ok(())
    }

    fn search(&self, query: &str, _limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        if let Some(color) = self.parse_color(query) {
            let color_name = self.get_color_name(&color);
            let (r, g, b) = color.rgb;
            let (h, s, l) = color.hsl;

            if let Ok(mut guard) = self.parsed_color.lock() {
                *guard = Some(color.clone());
            }

            results.push(SearchResult::new(
                format!("color_picker:{}", color.hex),
                color_name.clone(),
                format!(
                    "HEX: {} | RGB:({}, {}, {}) | HSL({}, {}%, {}%)",
                    color.hex, r, g, b, h, s, l
                ),
                ResultType::Custom("color".to_string()),
                1000,
                ActionData::CopyToClipboard { text: color.hex.clone() },
            ));

            results.push(SearchResult::new(
                format!("color_picker:rgb:{}", color.hex),
                format!("RGB({}, {}, {})", r, g, b),
                "点击复制 RGB 值".to_string(),
                ResultType::Custom("color".to_string()),
                950,
                ActionData::CopyToClipboard { text: format!("rgb({}, {}, {})", r, g, b) },
            ));

            results.push(SearchResult::new(
                format!("color_picker:hsl:{}", color.hex),
                format!("HSL({}, {}%, {}%)", h, s, l),
                "点击复制 HSL 值".to_string(),
                ResultType::Custom("color".to_string()),
                940,
                ActionData::CopyToClipboard { text: format!("hsl({}, {}%, {}%)", h, s, l) },
            ));

            results.push(SearchResult::new(
                format!("color_picker:hex:{}", color.hex),
                color.hex.clone(),
                "点击复制 HEX 值".to_string(),
                ResultType::Custom("color".to_string()),
                960,
                ActionData::CopyToClipboard { text: color.hex.clone() },
            ));
        }

        Ok(results)
    }

    fn execute(&self, result: &SearchResult) -> Result<()> {
        if let ActionData::CopyToClipboard { text } = &result.action {
            log::info!("复制颜色: {}", text);
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        if let Ok(mut guard) = self.parsed_color.lock() {
            *guard = None;
        }
        Ok(())
    }
}

impl Default for ColorPickerPlugin {
    fn default() -> Self {
        Self::new()
    }
}
