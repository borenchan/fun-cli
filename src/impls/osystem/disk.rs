use crate::ui::theme::Theme;
use crate::ui::widget::Widget;
use crate::ui::Coordinate;
use crate::utils::consts;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Print, SetForegroundColor};
use std::io::Stdout;
use sysinfo::Disks;

pub struct DiskWidget {
    coordinate: Coordinate,
    width: u16,
    height: u16,
    theme: Theme,
    disks: Vec<DiskInfo>,
}
struct DiskInfo {
    name: String,        // 磁盘名称
    file_system: String, // 文件系统
    kind: String,        // 磁盘类型
    total: u64,
    available: u64,
}
impl DiskWidget {
    pub fn new(left_top: Coordinate, right_bottom: Coordinate, theme: Theme) -> Self {
        Self {
            width: (right_bottom.x - left_top.x) + 1,
            height: (right_bottom.y - left_top.y) + 1,
            coordinate: left_top,
            theme,
            disks: Disks::new_with_refreshed_list()
                .list()
                .iter()
                .map(|disk| DiskInfo {
                    name: disk.name().to_str().unwrap_or(consts::UNKNOWN).to_string(),
                    file_system: disk
                        .file_system()
                        .to_str()
                        .unwrap_or(consts::UNKNOWN)
                        .to_string(),
                    kind: disk.kind().to_string(),
                    total: disk.total_space(),
                    available: disk.available_space(),
                })
                .collect::<Vec<_>>(),
        }
    }
}
impl Widget for DiskWidget {
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
        // for (i, disk) in self.disks.iter().enumerate() {
        //     if i>=1 {
        //         break;
        //     }
        //
        //     x = x + i as u16 * 20;
        // };
        let disk = self.disks.get(0).unwrap();
        queue!(
            stdout,
            MoveTo(x, y),
            Print(format!("磁盘名称:     {:>12}", disk.name))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 1),
            Print(format!("文件系统类型:  {:>12}", disk.file_system))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 2),
            Print(format!("磁盘类型:      {:>12}", disk.kind))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 3),
            Print(format!(
                "磁盘总空间:    {:>10}GB",
                disk.total / consts::SIZE_GB
            ))
        )?;
        queue!(
            stdout,
            MoveTo(x, y + 4),
            Print(format!(
                "可用空间:      {:>10}GB",
                disk.available / consts::SIZE_GB
            ))
        )?;
        Ok(())
    }
}
