use crate::ui::Coordinate;
use crate::ui::theme::Theme;
use crate::ui::widget::{List, Widget};
use crate::utils::consts;
use crossterm::cursor::MoveTo;
use crossterm::event::KeyCode;
use crossterm::queue;
use crossterm::style::{Print, SetForegroundColor};
use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, get_sockets_info};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Stdout;
use sysinfo::{Pid, System};

pub struct ProcessWidget {
    coordinate: Coordinate,
    width: u16,
    height: u16,
    process_list: List<ProcessInfo>,
    theme: Theme,
}
#[derive(Debug)]
struct ProcessInfo {
    name: String,
    pid: u32,
    cpu: f32,
    mem: u64,
    ports: Vec<String>,
}
impl Display for ProcessInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //<25 左对齐 ，不足25个字符，用空格填充
        // <: 左对齐（默认用于字符串）
        // >: 右对齐（默认用于数字）
        // ^: 居中对齐
        write!(
            f,
            "{:<25}{:<10}{:>8.2}%{:>10}MB  {:^10}",
            self.name,
            self.pid,
            self.cpu,
            self.mem,
            self.ports.join(" ")
        )
    }
}

impl ProcessWidget {
    pub fn new(left_top: Coordinate, right_bottom: Coordinate, theme: Theme, sys: &System) -> Self {
        // 先计算基本字段
        let width = (right_bottom.x - left_top.x) + 1;
        let height = (right_bottom.y - left_top.y) + 1;
        let sockets_info = get_sockets_info(
            AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
            ProtocolFlags::TCP | ProtocolFlags::UDP,
        )
        .unwrap_or(vec![]);
        let mut socket_map = HashMap::<u32, Vec<String>>::new();
        sockets_info.iter().for_each(|socket_info| {
            let protocol = match socket_info.protocol_socket_info {
                ProtocolSocketInfo::Tcp(_) => "TCP",
                ProtocolSocketInfo::Udp(_) => "UDP",
            };
            for pid in &socket_info.associated_pids {
                socket_map.entry(pid.clone()).or_insert(vec![]).push(format!(
                    "{}/{}",
                    protocol,
                    socket_info.local_port()
                ));
            }
        });
        Self {
            width,
            height,
            theme: theme.clone(),
            coordinate: left_top.clone(),
            process_list: {
                let mut process_list = sys
                    .processes()
                    .iter()
                    .map(|(pid, process)| ProcessInfo {
                        name: process.name().to_str().unwrap().to_string(),
                        pid: pid.as_u32(),
                        cpu: process.cpu_usage() / sys.physical_core_count().unwrap() as f32,
                        mem: process.memory() / consts::SIZE_MB,
                        ports: socket_map.remove(&pid.as_u32()).unwrap_or(vec![]),
                    })
                    .collect::<Vec<ProcessInfo>>();
                // 按照 CPU 使用率降序排序
                process_list.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap());
                let (x, y) = (left_top.x + 2, left_top.y + 2);
                let mut list = List::new(
                    Coordinate::new(x, y),
                    Coordinate::new(x + width - 2, y + height - 3),
                    theme.clone(),
                );
                list.set_items(process_list);
                list
            },
        }
    }
}
impl Widget for ProcessWidget {
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
        queue!(
            stdout,
            MoveTo(self.coordinate.x + 2, self.coordinate.y + 1),
            SetForegroundColor(self.theme.primary_text_color())
        )?;
        queue!(
            stdout,
            Print(format!("{:<23}{:<10}{:>8}{:>10}{:>10}", "进程", "pid", "cpu", "内存", "端口"))
        )?;
        self.process_list.render(stdout)?;
        Ok(())
    }

    fn handle_event(&mut self, event: KeyCode) -> bool {
        match event {
            KeyCode::Up | KeyCode::Down => self.process_list.handle_event(event),
            KeyCode::Char('k') | KeyCode::Delete => {
                let process_info = self.process_list.get_selected();
                let sys = System::new_all();
                if process_info.is_some() {
                    let pid = process_info.unwrap().pid;
                    let process = sys.process(Pid::from_u32(pid)).unwrap();
                    return process.kill();
                }
                false
            }
            _ => false,
        }
    }
    // 设置焦点
    fn set_focus(&mut self, focused: bool) {
        self.process_list.set_focus(focused)
    }
}
