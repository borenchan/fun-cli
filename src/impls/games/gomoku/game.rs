use super::ai::{AIStrategy, select_strategy};
use super::entity::{Cell, GomokuBoard, Position};
use crate::error::CliError;
use crate::impls::game::Game;
use crate::ui::event::poll_input;
use crossterm::event::KeyCode;
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, queue, terminal};
use std::io::{Stdout, Write, stdout};
use std::thread::sleep;
use std::time::Duration;

pub struct GomokuGame;

struct GomokuGameState {
    board: GomokuBoard,
    cursor_pos: Position,
    difficulty: u8,
    stdout: Stdout,
    ai_strategy: Box<dyn AIStrategy>,
    game_over: bool,
    winner: Option<&'static str>,
}

impl GomokuGameState {
    pub fn new(size: usize, difficulty: u8) -> Self {
        let ai_strategy = select_strategy(difficulty);
        let cursor_pos = Position::new(size / 2, size / 2);

        Self {
            board: GomokuBoard::new(size),
            cursor_pos,
            difficulty,
            stdout: stdout(),
            ai_strategy,
            game_over: false,
            winner: None,
        }
    }

    /// 渲染棋盘
    fn render(&mut self) -> Result<(), CliError> {
        // 清屏
        queue!(self.stdout, Clear(ClearType::All))?;

        let size = self.board.size();

        // 显示标题（第0行）
        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
            Print(format!("五子棋 {}x{} - 难度: {}", size, size, self.difficulty).cyan().bold())
        )?;

        // 显示操作说明（第1行）
        queue!(
            self.stdout,
            cursor::MoveTo(0, 1),
            Print("方向键移动 | Enter落子 | Q退出".dark_grey())
        )?;

        // 绘制列标（第3行）
        queue!(self.stdout, cursor::MoveTo(0, 3), Print("   "))?;
        for col in 0..size {
            let label = (b'A' + col as u8) as char;
            queue!(self.stdout, Print(format!("{} ", label)))?;
        }

        // 绘制棋盘（从第4行开始）
        for row in 0..size {
            // 定位到行首
            queue!(
                self.stdout,
                cursor::MoveTo(0, 4 + row as u16),
                Print(format!("{:2} ", row + 1))
            )?;

            // 绘制该行的所有位置
            for col in 0..size {
                let pos = Position::new(row, col);
                let cell = self.board.get(pos).unwrap();
                let is_cursor = pos == self.cursor_pos;

                let symbol = match cell {
                    Cell::Empty => {
                        if is_cursor {
                            "+".yellow().bold()
                        } else {
                            ".".dark_grey()
                        }
                    }
                    Cell::Black => "@".white().bold(),  // 使用 @ 代替 ●
                    Cell::White => "O".red().bold(),    // 使用 O 代替 ○
                };

                queue!(self.stdout, Print(symbol), Print(" "))?;
            }
        }

        self.stdout.flush()?;
        Ok(())
    }

    /// 处理玩家输入
    fn handle_input(&mut self) -> Result<bool, CliError> {
        if let Some(key_code) = poll_input()? {
            match key_code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    return Ok(true); // 退出游戏
                }
                KeyCode::Up => {
                    if self.cursor_pos.row > 0 {
                        self.cursor_pos.row -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.cursor_pos.row < self.board.size() - 1 {
                        self.cursor_pos.row += 1;
                    }
                }
                KeyCode::Left => {
                    if self.cursor_pos.col > 0 {
                        self.cursor_pos.col -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.cursor_pos.col < self.board.size() - 1 {
                        self.cursor_pos.col += 1;
                    }
                }
                KeyCode::Enter => {
                    // 玩家尝试落子
                    if self.board.place(self.cursor_pos, Cell::Black) {
                        // 检查玩家是否获胜
                        if self.board.check_win(self.cursor_pos) {
                            self.game_over = true;
                            self.winner = Some("玩家");
                            return Ok(false);
                        }

                        // 检查平局
                        if self.board.is_full() {
                            self.game_over = true;
                            self.winner = None;
                            return Ok(false);
                        }

                        // AI 回合
                        self.ai_turn()?;
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }

    /// AI 回合
    fn ai_turn(&mut self) -> Result<(), CliError> {
        // 显示 AI 思考提示（困难和地狱模式）
        if self.difficulty >= 3 {
            let thinking_msg = if self.difficulty >= 4 {
                "地狱 AI 深度思考中...".red().bold()
            } else {
                "AI 思考中...".yellow()
            };
            queue!(
                self.stdout,
                cursor::MoveTo(0, (self.board.size() + 5) as u16),
                Print(thinking_msg)
            )?;
            self.stdout.flush()?;
        }

        // AI 计算落子位置
        if let Some(ai_pos) = self.ai_strategy.next_move(&self.board) {
            self.board.place(ai_pos, Cell::White);

            // 检查 AI 是否获胜
            if self.board.check_win(ai_pos) {
                self.game_over = true;
                self.winner = Some("AI");
            }

            // 检查平局
            if self.board.is_full() {
                self.game_over = true;
                self.winner = None;
            }
        }

        Ok(())
    }

    /// 显示游戏结束界面，返回 true 表示用户选择退出
    fn show_game_over(&mut self) -> Result<bool, CliError> {
        self.render()?;

        queue!(
            self.stdout,
            cursor::MoveTo(0, (self.board.size() + 5) as u16),
            Print("\n")
        )?;

        if let Some(winner) = self.winner {
            queue!(
                self.stdout,
                Print(format!("🎉 {} 获胜！\n", winner).green().bold())
            )?;
        } else {
            queue!(self.stdout, Print("平局！\n".yellow().bold()))?;
        }

        queue!(self.stdout, Print("按 R 重新开始，按 Q 退出\n".dark_grey()))?;
        self.stdout.flush()?;

        // 等待用户选择
        loop {
            if let Some(key_code) = poll_input()? {
                match key_code {
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        // 重新开始
                        let size = self.board.size();
                        *self = Self::new(size, self.difficulty);
                        return Ok(false); // 不退出，继续游戏
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        return Ok(true); // 退出游戏
                    }
                    _ => {}
                }
            }
        }
    }

    /// 游戏主循环
    fn run(&mut self) -> Result<(), CliError> {
        // 进入备用屏幕
        execute!(self.stdout, EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;

        let result = (|| -> Result<(), CliError> {
            loop {
                self.render()?;

                if self.game_over {
                    if self.show_game_over()? {
                        break; // 用户选择退出
                    }
                    // 用户选择重新开始，继续循环
                    continue;
                }

                if self.handle_input()? {
                    break; // 用户退出
                }

                sleep(Duration::from_millis(16)); // 约 60 FPS
            }
            Ok(())
        })();

        // 恢复终端状态
        terminal::disable_raw_mode()?;
        execute!(self.stdout, LeaveAlternateScreen)?;

        result
    }
}

impl Game for GomokuGame {
    fn name(&self) -> &'static str {
        "五子棋 ⚫⚪"
    }

    fn help(&self) -> &'static str {
        "人机对战五子棋，支持四种难度等级（1简单 2中等 3困难 4地狱）"
    }

    fn run(&self, width: u16, height: u16, difficulty: u8) -> Result<(), CliError> {
        // 确定棋盘大小，如果使用默认值（80x30），则使用 15x15
        let size = if width == 80 && height == 30 {
            15 // 标准五子棋棋盘大小
        } else {
            let requested_size = width.min(height) as usize;
            if requested_size < 9 {
                return Err(CliError::UnknownError(
                    "棋盘大小至少为 9x9，请使用 -w 9 -H 9 或更大".to_string(),
                ));
            }
            if requested_size > 19 {
                return Err(CliError::UnknownError(
                    "棋盘大小最大为 19x19，请使用 -w 19 -H 19 或更小".to_string(),
                ));
            }
            requested_size
        };

        // 验证难度等级
        let difficulty = if !(1..=4).contains(&difficulty) {
            eprintln!("警告：难度等级无效，使用默认难度 1（简单）");
            1
        } else {
            difficulty
        };

        let mut game_state = GomokuGameState::new(size, difficulty);
        game_state.run()
    }
}
