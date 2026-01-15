use crate::ui::Coordinate;
use crate::ui::theme::Theme;
use crossterm::cursor::MoveTo;
use crossterm::event::KeyCode;
use crossterm::queue;
use crossterm::style::{Print, ResetColor, SetForegroundColor};
use std::fmt::Display;
use std::io::{self, Stdout, Write};

// 所有UI组件的基础特征
pub trait Widget {
    /// 获取组件边界坐标
    fn bounds(&self) -> (Coordinate, Coordinate) {
        (self.coordinate(), Coordinate::new(self.width(), self.height()))
    }
    // 组件左上角坐标
    fn coordinate(&self) -> Coordinate;
    // 组件宽度
    fn width(&self) -> u16;
    // 组件高度
    fn height(&self) -> u16;

    /// 渲染组件到终端
    fn render(&self, stdout: &mut Stdout) -> io::Result<()>;

    /// 处理输入事件（返回true表示事件被消费）
    fn handle_event(&mut self, event: KeyCode) -> bool {
        false
    }

    /// 设置组件是否有焦点
    fn set_focus(&mut self, focused: bool) {
        // 默认空实现，需要焦点的组件可重写
    }
    /// 在组件窗口范围内渲染，超出范围的内容被忽略
    fn render_on_window(&self, stdout: &mut Stdout, coordinate: Coordinate, content: String) -> io::Result<()> {
        queue!(stdout, MoveTo(coordinate.x, coordinate.y), Print(content))?;
        Ok(())
    }
}

// 带边框的面板组件
pub struct Panel<T: Widget> {
    title: String,
    child: T, // 面板内的子组件
    focused: bool,
    theme: Theme, // 主题，
}

impl<T: Widget> Panel<T> {
    pub fn new(title: &str, child: T, theme: Theme) -> Self {
        Self {
            title: title.to_string(),
            child,
            focused: false,
            theme,
        }
    }

    /// 渲染边框和标题
    fn render_border(&self, stdout: &mut Stdout) -> io::Result<()> {
        let coordinate = self.coordinate();
        // 绘制标题（带焦点时用不同颜色）
        let offset = (self.title.len() / 2) as u16;
        if self.focused {
            // 焦点状态：高亮标题
            queue!(stdout, SetForegroundColor(self.theme.highlight_color()))?;
        } else {
            queue!(stdout, SetForegroundColor(self.theme.primary_text_color()))?;
        }
        queue!(
            stdout,
            MoveTo(coordinate.x + self.width() / 2 - offset, coordinate.y + 1),
            Print(&self.title)
        )?;
        //绘制边框
        if self.focused {
            // 焦点状态：高亮标题
            queue!(stdout, SetForegroundColor(self.theme.highlight_color()))?;
        } else {
            queue!(stdout, SetForegroundColor(self.theme.secondary_color()))?;
        }
        // 绘制顶部边框
        queue!(stdout, MoveTo(coordinate.x, coordinate.y), Print("┌"))?;
        queue!(
            stdout,
            MoveTo(coordinate.x + 1, coordinate.y),
            Print("-".repeat((self.width() - 2) as usize))
        )?;
        queue!(stdout, MoveTo(coordinate.x + self.width() - 1, coordinate.y), Print("┐"))?;

        // queue!(stdout,ResetColor)?;
        // 绘制两边边框
        for i in 1..(self.height() - 1) {
            queue!(stdout, MoveTo(coordinate.x, coordinate.y + i), Print("|"))?;
            queue!(stdout, MoveTo(coordinate.x + self.width() - 1, coordinate.y + i), Print("|"))?;
        }
        // 绘制底部边框
        queue!(stdout, MoveTo(coordinate.x, coordinate.y + self.height() - 1), Print("└"))?;
        queue!(
            stdout,
            MoveTo(coordinate.x + 1, coordinate.y + self.height() - 1),
            Print("-".repeat((self.width() - 2) as usize))
        )?;
        queue!(
            stdout,
            MoveTo(coordinate.x + self.width() - 1, coordinate.y + self.height() - 1),
            Print("┘")
        )?;
        Ok(())
    }
    /// 更新子组件
    pub fn update_widget(&mut self, new_widget: T) {
        self.child = new_widget;
    }
}

// 实现Widget接口
impl<T: Widget> Widget for Panel<T> {
    fn coordinate(&self) -> Coordinate {
        let child = self.child.coordinate();
        Coordinate::new(child.x - 1, child.y - 1)
    }

    fn width(&self) -> u16 {
        self.child.width() + 2
    }
    fn height(&self) -> u16 {
        self.child.height() + 2
    }

    fn render(&self, stdout: &mut Stdout) -> io::Result<()> {
        // 先渲染边框
        self.render_border(stdout)?;
        // 再渲染子组件（调整子组件位置到边框内部）
        self.child.render(stdout)
    }

    fn handle_event(&mut self, event: KeyCode) -> bool {
        // 面板本身不处理事件，转发给子组件
        self.child.handle_event(event)
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
        // 将焦点状态传递给子组件
        self.child.set_focus(focused);
    }
}

// 列表组件（支持选中和滚动）
pub struct List<T: Display> {
    coordinate: Coordinate,
    width: u16,
    height: u16,
    items: Vec<T>,        // 列表项
    selected_idx: usize,  // 选中项索引
    scroll_offset: usize, // 滚动偏移量（处理内容超出可视区域）
    focused: bool,        // 是否获得焦点
    theme: Theme,         // 主题
    padding: u16,         // 内边距
}

impl<T: Display> List<T> {
    pub fn new(left_top: Coordinate, right_bottom: Coordinate, theme: Theme) -> Self {
        Self::new_with_padding(left_top, right_bottom, theme, 0)
    }
    pub fn new_with_padding(left_top: Coordinate, right_bottom: Coordinate, theme: Theme, padding: u16) -> Self {
        Self {
            width: right_bottom.x - left_top.x,
            height: right_bottom.y - left_top.y,
            coordinate: left_top,
            items: Vec::new(),
            selected_idx: 0,
            scroll_offset: 0,
            focused: false,
            theme,
            padding,
        }
    }
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }
    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }
    pub fn get_selected(&self) -> Option<&T> {
        self.items.get(self.selected_idx)
    }

    /// 确保选中项在可视区域内（自动滚动）
    fn adjust_scroll(&mut self) {
        let visible_lines = (self.height - 2) as usize; // 减去边框占用的行
        if self.selected_idx < self.scroll_offset {
            self.scroll_offset = self.selected_idx;
        } else if self.selected_idx >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.selected_idx - visible_lines + 1;
        }
    }
}

// 实现Widget接口
impl<T: Display> Widget for List<T> {
    fn coordinate(&self) -> Coordinate {
        self.coordinate.clone()
    }
    fn width(&self) -> u16 {
        self.width
    }
    fn height(&self) -> u16 {
        self.height
    }

    fn render(&self, stdout: &mut Stdout) -> io::Result<()> {
        let visible_lines = (self.height) as usize;
        let display_items =
            &self.items[self.scroll_offset..std::cmp::min(self.scroll_offset + visible_lines, self.items.len())];
        queue!(stdout, ResetColor)?;
        // 渲染列表项
        for (i, item) in display_items.iter().enumerate() {
            let line_y = self.coordinate.y + self.padding + i as u16;
            queue!(
                stdout,
                MoveTo(self.coordinate.x + self.padding, line_y),
                SetForegroundColor(self.theme.primary_text_color())
            )?;
            // 选中项高亮
            if self.focused && self.scroll_offset + i == self.selected_idx {
                queue!(stdout, SetForegroundColor(self.theme.highlight_color()))?;
            }

            // 截断过长的文本以适应宽度
            let max_len = (self.width - 2) as usize;
            let item_str = item.to_string();
            let display_str = if item_str.len() > max_len {
                format!("{}...", &item_str[..max_len - 3])
            } else {
                item_str
            };
            queue!(stdout, Print(display_str), ResetColor)?;
        }
        // stdout.flush()?;
        Ok(())
    }

    fn handle_event(&mut self, event: KeyCode) -> bool {
        if self.items.is_empty() {
            return false;
        }

        match event {
            KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                    self.adjust_scroll();
                }
                true
            }
            KeyCode::Down => {
                if self.selected_idx < self.items.len() - 1 {
                    self.selected_idx += 1;
                    self.adjust_scroll();
                }
                true
            }
            _ => false,
        }
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::impls::osystem::disk::DiskWidget;
    use crossterm::execute;
    use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen};
    use std::io::stdout;
    use std::thread::sleep;
    use std::time::Duration;
    #[test]
    fn test_panel_widget() {
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All)).unwrap();
        let (width, height) = (80, 21);
        let half_w = width / 2;
        let percent_h = height / 3;
        let theme = Theme::Ocean;
        let panel1 = Panel::new(
            "Panel",
            DiskWidget::new(Coordinate::new(1, 1), Coordinate::new(width - 1, percent_h - 1), theme.clone()),
            theme.clone(),
        );
        let panel2 = Panel::new(
            "left1",
            DiskWidget::new(
                Coordinate::new(1, percent_h + 1),
                Coordinate::new(half_w - 1, percent_h * 2 - 1),
                theme.clone(),
            ),
            theme.clone(),
        );
        let panel3 = Panel::new(
            "right1",
            DiskWidget::new(
                Coordinate::new(half_w + 1, percent_h + 1),
                Coordinate::new(width - 1, percent_h * 2 - 1),
                theme.clone(),
            ),
            theme.clone(),
        );
        let panel4 = Panel::new(
            "left2",
            DiskWidget::new(
                Coordinate::new(1, percent_h * 2 + 1),
                Coordinate::new(half_w - 1, height - 1),
                theme.clone(),
            ),
            theme.clone(),
        );
        let panel5 = Panel::new(
            "right2",
            DiskWidget::new(
                Coordinate::new(half_w + 1, percent_h * 2 + 1),
                Coordinate::new(width - 1, height - 1),
                theme.clone(),
            ),
            theme.clone(),
        );
        let panels = vec![panel1, panel2, panel3, panel4, panel5];
        for x in panels {
            x.render(&mut stdout).unwrap();
        }
        stdout.flush().unwrap();
        sleep(Duration::from_secs(1000));
    }
}
