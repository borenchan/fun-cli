use crate::ui::Coordinate;
use crate::ui::theme::Theme;
use crate::ui::widget::Widget;
use crate::utils::consts;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Print, SetForegroundColor};
use std::io::Stdout;
use sysinfo::System;

pub struct CpuWidget {
    coordinate: Coordinate,
    width: u16,
    height: u16,
    theme: Theme,
    cpu_brand: String,          // CPU品牌
    cpu_usage: f32,             // CPU使用率
    core_count: usize,          // CPU物理核心数
    frequency: u64,             // CPU频率
    physical_core_count: usize, // CPU物理核心数
}

impl CpuWidget {
    pub fn new(
        left_top: Coordinate,
        right_bottom: Coordinate,
        theme: Theme,
        sys: &mut System,
    ) -> Self {
        Self {
            width: (right_bottom.x - left_top.x) + 1,
            height: (right_bottom.y - left_top.y) + 1,
            coordinate: left_top,
            theme,
            cpu_brand: sys
                .cpus()
                .first()
                .map(|c| c.brand().to_string())
                .unwrap_or(consts::UNKNOWN.to_string()),
            cpu_usage: sys.global_cpu_usage(),
            core_count: sys.cpus().len(),
            physical_core_count: sys.physical_core_count().unwrap_or(0),
            frequency: sys.cpus().first().map(|c| c.frequency()).unwrap_or(0),
        }
    }
}

impl Widget for CpuWidget {
    fn coordinate(&self) -> Coordinate {
        self.coordinate.clone()
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn render(
        &self,
        stdout: &mut Stdout,
    ) -> std::io::Result<()> {
        let (x, y) = (self.coordinate().x + 2, self.coordinate().y + 2);
        queue!(stdout, SetForegroundColor(self.theme.primary_text_color()))?;
        queue!(stdout, MoveTo(x, y), Print(format!("CPU型号:      {:>5}", self.cpu_brand)))?;
        queue!(
            stdout,
            MoveTo(x, y + 1),
            Print(format!("CPU使用率:     {:>5.2}%", self.cpu_usage))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 2),
            Print(format!("Core核心数:    {:>6}", self.core_count))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 3),
            Print(format!("Core物理核心数: {:>5}", self.core_count))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 4),
            Print(format!("Core频率:      {:>5} MHz", self.frequency))
        )?;
        Ok(())
    }
}
