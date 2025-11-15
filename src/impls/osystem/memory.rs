use crate::ui::Coordinate;
use crate::ui::theme::Theme;
use crate::ui::widget::Widget;
use crate::utils::consts;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Print, SetForegroundColor};
use std::io::Stdout;
use sysinfo::System;

pub struct MemoryWidget {
    coordinate: Coordinate,
    width: u16,
    height: u16,
    theme: Theme,
    total_memory: u64,
    used_memory: u64,
    free_memory: u64,
    available_memory: u64,
    memory_usage: f64,
}
impl MemoryWidget {
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
            total_memory: sys.total_memory(),
            used_memory: sys.used_memory(),
            free_memory: sys.free_memory(),
            available_memory: sys.available_memory(),
            memory_usage: (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0,
        }
    }
}
impl Widget for MemoryWidget {
    fn coordinate(&self) -> Coordinate {
        self.coordinate.clone()
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn render(&self, stdout: &mut Stdout) -> std::io::Result<()> {
        queue!(stdout, SetForegroundColor(self.theme.primary_text_color()))?;
        let (x, y) = (self.coordinate().x + 2, self.coordinate().y + 2);
        queue!(
            stdout,
            MoveTo(x, y),
            Print(format!(
                "内存总大小:  {:>10}MB",
                self.total_memory / consts::SIZE_MB
            ))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 1),
            Print(format!(
                "已使用内存:  {:>10}MB",
                self.used_memory / consts::SIZE_MB
            ))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 2),
            Print(format!(
                "空闲内存:    {:>10}MB",
                self.free_memory / consts::SIZE_MB
            ))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 3),
            Print(format!(
                "可用内存:    {:>10}MB",
                self.available_memory / consts::SIZE_MB
            ))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 4),
            Print(format!("内存使用率:  {:>11.2}%", self.memory_usage))
        )?;
        Ok(())
    }
}
