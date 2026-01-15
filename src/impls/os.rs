use crate::error::CliError;
use crate::impls::handlers::CommandHandler;
use crate::impls::osystem::cpu::CpuWidget;
use crate::impls::osystem::disk::DiskWidget;
use crate::impls::osystem::memory::MemoryWidget;
use crate::impls::osystem::process::ProcessWidget;
use crate::ui::Coordinate;
use crate::ui::event::poll_input;
use crate::ui::theme::Theme;
use crate::ui::widget::{List, Panel, Widget};
use crate::utils::consts;
use clap::{Parser, ValueEnum};
use crossterm::cursor::{Hide, Show};
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::style::SetBackgroundColor;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::io::{Write, stdout};
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use sysinfo::{Components, System};

#[derive(Debug, Parser)]
pub struct OsHandler {
    #[arg(short, long,value_enum, default_value_t = SizeMode::Large, help = "面板大小/small/middle/large")]
    size: SizeMode,
    #[arg(short, long,value_enum, default_value_t = Theme::Cyberpunk, help = "主题/cyberpunk/blackgold/fire/ocean/aurora")]
    theme: Theme,
}
#[derive(Debug, Clone, ValueEnum)]
pub enum SizeMode {
    Small,
    Middle,
    Large,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum LayoutPosition {
    Top,
    LeftTop,
    RightTop,
    LeftBottom,
    RightBottom,
}

//布局面板
struct LayoutPanel {
    overview_panel: Rc<RefCell<Panel<List<String>>>>,
    cpu_panel: Rc<RefCell<Panel<CpuWidget>>>,
    disk_panel: Rc<RefCell<Panel<DiskWidget>>>,
    memory_panel: Rc<RefCell<Panel<MemoryWidget>>>,
    process_panel: Rc<RefCell<Panel<ProcessWidget>>>,
    widgets: Vec<Rc<RefCell<dyn Widget>>>,
    focus_idx: usize,
    focus_mode: bool,
    layout: HashMap<LayoutPosition, (Coordinate, Coordinate)>,
    theme: Theme,
}
impl LayoutPanel {
    fn calculate_layout(
        width: u16,
        height: u16,
    ) -> HashMap<LayoutPosition, (Coordinate, Coordinate)> {
        let half_w = width / 2;
        let percent_h = height / 3;
        let mut layout = HashMap::with_capacity(5);
        layout.insert(
            LayoutPosition::Top,
            (Coordinate::new(1, 1), Coordinate::new(width - 1, percent_h - 1)),
        ); // 顶部区域
        layout.insert(
            LayoutPosition::LeftTop,
            (Coordinate::new(1, percent_h + 1), Coordinate::new(half_w, percent_h * 2)),
        ); // 左上
        layout.insert(
            LayoutPosition::RightTop,
            (
                Coordinate::new(half_w + 1, percent_h + 1),
                Coordinate::new(width - 1, percent_h * 2 - 1),
            ),
        ); // 右上
        layout.insert(
            LayoutPosition::LeftBottom,
            (Coordinate::new(1, percent_h * 2 + 1), Coordinate::new(half_w - 1, height - 1)),
        ); // 左下
        layout.insert(
            LayoutPosition::RightBottom,
            (
                Coordinate::new(half_w + 1, percent_h * 2 + 1),
                Coordinate::new(width - 1, height - 1),
            ),
        ); // 右下
        layout
    }
    fn new(
        width: u16,
        height: u16,
        sys: &mut System,
        theme: Theme,
    ) -> Self {
        // queue!(&mut stdout(),MoveTo(0,height+2),Print(format!("尺寸：{} x {}" ,width,height)) ).unwrap();
        let layout = Self::calculate_layout(width, height);
        let top = layout.get(&LayoutPosition::Top).unwrap();
        let left_top = layout.get(&LayoutPosition::LeftTop).unwrap();
        let right_top = layout.get(&LayoutPosition::RightTop).unwrap();
        let left_bottom = layout.get(&LayoutPosition::LeftBottom).unwrap();
        let right_bottom = layout.get(&LayoutPosition::RightBottom).unwrap();

        let process_panel = Panel::new(
            "Process",
            ProcessWidget::new(top.0.clone(), top.1.clone(), theme.clone(), sys),
            theme.clone(),
        );
        let mut overview_list = List::new_with_padding(left_top.0.clone(), left_top.1.clone(), theme.clone(), 2);
        Self::set_overview_panel_list(&mut overview_list, sys);
        let overview_panel = Panel::new("INFO", overview_list, theme.clone());
        let cpu_panel = Panel::new(
            "CPU",
            CpuWidget::new(right_top.0.clone(), right_top.1.clone(), theme.clone(), sys),
            theme.clone(),
        );
        let disk_panel = Panel::new(
            "Disk",
            DiskWidget::new(left_bottom.0.clone(), left_bottom.1.clone(), theme.clone()),
            theme.clone(),
        );
        let memory_panel = Panel::new(
            "Memory",
            MemoryWidget::new(right_bottom.0.clone(), right_bottom.1.clone(), theme.clone(), sys),
            theme.clone(),
        );

        let mut layout_panel = LayoutPanel {
            overview_panel: Rc::new(RefCell::new(overview_panel)),
            cpu_panel: Rc::new(RefCell::new(cpu_panel)),
            disk_panel: Rc::new(RefCell::new(disk_panel)),
            memory_panel: Rc::new(RefCell::new(memory_panel)),
            process_panel: Rc::new(RefCell::new(process_panel)),
            widgets: vec![],
            layout,
            theme,
            focus_idx: 0,
            focus_mode: false,
        };
        layout_panel.widgets.push(layout_panel.process_panel.clone());
        layout_panel.widgets.push(layout_panel.overview_panel.clone());
        layout_panel.widgets.push(layout_panel.cpu_panel.clone());
        layout_panel.widgets.push(layout_panel.disk_panel.clone());
        layout_panel.widgets.push(layout_panel.memory_panel.clone());
        layout_panel
    }

    // 添加更新系统信息的方法
    fn update_system_info(
        &mut self,
        sys: &mut System,
    ) {
        sys.refresh_all();
        let top = self.layout.get(&LayoutPosition::Top).unwrap();
        let left_top = self.layout.get(&LayoutPosition::LeftTop).unwrap();
        let right_top = self.layout.get(&LayoutPosition::RightTop).unwrap();
        let left_bottom = self.layout.get(&LayoutPosition::LeftBottom).unwrap();
        let right_bottom = self.layout.get(&LayoutPosition::RightBottom).unwrap();
        // 更新进程面板
        {
            let mut panel = self.process_panel.borrow_mut();
            let new_widget = ProcessWidget::new(top.0.clone(), top.1.clone(), self.theme.clone(), sys);
            panel.update_widget(new_widget);
        }
        // 更新CPU面板
        {
            let mut panel = self.cpu_panel.borrow_mut();
            let new_widget = CpuWidget::new(right_top.0.clone(), right_top.1.clone(), self.theme.clone(), sys);
            panel.update_widget(new_widget);
        }
        // 更新内存面板
        {
            let mut panel = self.memory_panel.borrow_mut();
            let new_widget = MemoryWidget::new(right_bottom.0.clone(), right_bottom.1.clone(), self.theme.clone(), sys);
            panel.update_widget(new_widget);
        }
    }

    fn render(
        &mut self,
        stdout: &mut io::Stdout,
    ) -> Result<(), CliError> {
        for (index, panel) in self.widgets.iter_mut().enumerate() {
            let mut panel_mut = panel.borrow_mut();
            if index == self.focus_idx {
                panel_mut.set_focus(true);
            }
            panel_mut.render(stdout)?;
            // stdout.flush()?;
        }
        Ok(())
    }
    fn set_overview_panel_list(
        list: &mut List<String>,
        sys: &mut System,
    ) {
        let mut vec = vec![];
        vec.push(format!(
            "System name:             {}",
            System::name().unwrap_or(consts::UNKNOWN.to_string())
        ));
        vec.push(format!(
            "System OS version:       {}",
            System::os_version().unwrap_or(consts::UNKNOWN.to_string())
        ));
        vec.push(format!(
            "System kernel version:   {}",
            System::kernel_version().unwrap_or(consts::UNKNOWN.to_string())
        ));
        vec.push(format!(
            "System host name:        {}",
            System::host_name().unwrap_or(consts::UNKNOWN.to_string())
        ));
        vec.push(format!(
            "System cpu_arch:         {}",
            System::cpu_arch().unwrap_or(consts::UNKNOWN.to_string())
        ));
        let components = Components::new_with_refreshed_list();
        for component in &components {
            vec.push(format!(
                "组件: {:<10}  温度: {:<5.1}°C",
                component.label(),
                component.temperature()
            ));
        }
        list.set_items(vec);
    }
    /// 切换面板选中
    fn next_focus(
        &mut self,
        key_code: KeyCode,
    ) {
        if let Some(pan) = self.widgets.get_mut(self.focus_idx) {
            pan.borrow_mut().set_focus(false);
        }
        if key_code == KeyCode::Up {
            self.focus_idx = if self.focus_idx == 0 {
                self.widgets.len() - 1
            } else {
                self.focus_idx - 1
            };
        } else {
            self.focus_idx = (self.focus_idx + 1) % self.widgets.len();
        }
        if let Some(pan) = self.widgets.get_mut(self.focus_idx) {
            pan.borrow_mut().set_focus(true);
        }
    }
    /// 处理按键
    /// true：表示需要重建UI组件， false表示仅重新渲染数据即可
    fn handle_event(
        &mut self,
        key_code: KeyCode,
    ) -> bool {
        match key_code {
            KeyCode::Up | KeyCode::Down | KeyCode::Tab => {
                if !self.focus_mode {
                    self.next_focus(key_code);
                } else {
                    self.widgets[self.focus_idx].borrow_mut().handle_event(key_code);
                }
                false
            }
            KeyCode::Enter => {
                self.focus_mode = true;
                false
            }
            KeyCode::Esc => {
                self.focus_mode = false;
                false
            }
            _ => self.widgets[self.focus_idx].borrow_mut().handle_event(key_code),
        }
    }
}
impl CommandHandler for OsHandler {
    fn run(&self) -> Result<(), CliError> {
        let mut stdout = stdout();
        let mut sys = System::new_all();
        // enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, SetBackgroundColor(self.theme.background_color()))?;
        execute!(stdout, Clear(ClearType::All), Hide)?;
        let (terminal_width, terminal_height) = size()?;
        //terminal_width = terminal_width & !1 // 可以利用 & !1 对1按位取反操作。实现位运算向下取偶操作
        let (mut width, mut height) = match self.size {
            SizeMode::Small => (terminal_width / 3, terminal_height / 3),
            SizeMode::Middle => (terminal_width / 2, terminal_height / 2),
            SizeMode::Large => (terminal_width - 2, terminal_height - 2),
        };
        width = (width / 2) * 2;
        height = (height / 3) * 3;
        let mut layout_panel = LayoutPanel::new(width, height, &mut sys, self.theme.clone());
        let mut last_refresh = 1;
        loop {
            layout_panel.render(&mut stdout)?;
            stdout.flush()?;
            //接收输入
            if let Some(code) = poll_input()? {
                match code {
                    KeyCode::Char('q') => break,
                    _ => {
                        if layout_panel.handle_event(code) {
                            layout_panel.update_system_info(&mut sys);
                            last_refresh = 0;
                        }
                    }
                }
            } else {
                sleep(Duration::from_millis(100));
                if last_refresh % 20 == 0 {
                    layout_panel.update_system_info(&mut sys);
                }
            }
            last_refresh += 1;
            execute!(stdout, Clear(ClearType::All))?;
        }
        // disable_raw_mode()?;
        //恢复终端
        execute!(stdout, LeaveAlternateScreen, Show)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sysinfo::{Disks, System};

    #[test]
    fn test_sysinfo() {
        let sys = System::new_all();
        let mb = 1024 * 1024;
        println!("=> system: {:?}", sys);
        // RAM and swap information:
        println!("total memory: {} MB", sys.total_memory() / mb);
        println!("used memory : {} MB", sys.used_memory() / mb);
        println!("total swap  : {} MB", sys.total_swap() / mb);
        println!("used swap   : {} MB", sys.used_swap() / mb);

        // Display system information:
        println!("System name:             {:?}", System::name());
        println!("System kernel version:   {:?}", System::kernel_version());
        println!("System OS version:       {:?}", System::os_version());
        println!("System host name:        {:?}", System::host_name());
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            println!("{disk:?}");
        }
        // Display processes ID, name and disk usage:
        for (pid, process) in sys.processes() {
            println!("[{pid}] {:?} {:?}", process.name(), process.disk_usage());
        }
    }
}
