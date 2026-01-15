use clap::ValueEnum;
use crossterm::style::Color;

#[derive(Debug, Clone, ValueEnum)]
pub enum Theme {
    Cyberpunk, // 赛博朋克
    BlackGold, // 黑金奢华
    Fire,      // 火焰主题
    Ocean,     // 海洋深度
    Aurora,    // 极光风格
}
impl Theme {
    /// 背景色
    pub fn background_color(&self) -> Color {
        match self {
            Theme::Cyberpunk => Color::Rgb { r: 10, g: 5, b: 20 },
            Theme::BlackGold => Color::Rgb { r: 15, g: 15, b: 15 },
            Theme::Fire => Color::Rgb { r: 20, g: 10, b: 10 },
            Theme::Ocean => Color::Rgb { r: 5, g: 15, b: 30 },
            Theme::Aurora => Color::Rgb { r: 5, g: 5, b: 25 },
        }
    }
    /// 文本色
    pub fn primary_text_color(&self) -> Color {
        match self {
            Theme::Cyberpunk => Color::Rgb { r: 0, g: 255, b: 255 },
            Theme::BlackGold => Color::Rgb { r: 255, g: 215, b: 0 },
            Theme::Fire => Color::Rgb { r: 255, g: 69, b: 0 },
            Theme::Ocean => Color::Rgb { r: 30, g: 144, b: 255 },
            Theme::Aurora => Color::Rgb { r: 50, g: 205, b: 50 },
        }
    }
    /// 高亮色
    pub fn highlight_color(&self) -> Color {
        match self {
            Theme::Cyberpunk => Color::Rgb { r: 255, g: 0, b: 255 },
            Theme::BlackGold => Color::Rgb { r: 255, g: 255, b: 255 },
            Theme::Fire => Color::Rgb { r: 255, g: 215, b: 0 },
            Theme::Ocean => Color::Rgb { r: 0, g: 255, b: 255 },
            Theme::Aurora => Color::Rgb { r: 138, g: 43, b: 226 },
        }
    }
    /// 辅助色
    pub fn secondary_color(&self) -> Color {
        match self {
            Theme::Cyberpunk => Color::Rgb { r: 0, g: 255, b: 0 },
            Theme::BlackGold => Color::Rgb { r: 184, g: 134, b: 11 },
            Theme::Fire => Color::Rgb { r: 220, g: 20, b: 60 },
            Theme::Ocean => Color::Rgb { r: 173, g: 216, b: 230 },
            Theme::Aurora => Color::Rgb { r: 255, g: 105, b: 180 },
        }
    }
}
