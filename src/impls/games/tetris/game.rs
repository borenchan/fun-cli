use crate::error::CliError;
use crate::impls::game::Game;
use crate::impls::games::entities::{Entity, GameEntity};
use crate::impls::games::tetris::entity::{TetrisBlock, TetrisPiece};
use crate::ui::event::poll_input;
use crossterm::event::KeyCode;
use crossterm::style::{Color, Print, Stylize};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, queue, terminal};
use std::io::{Stdout, Write, stdout};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub struct TetrisGame;

struct TetrisGameState {
    width: u16,
    height: u16,
    difficulty: u8,
    stdout: Stdout,
    current_piece: Option<TetrisPiece>,
    placed_blocks: Vec<TetrisBlock>,
    score: u32,
    lines_cleared: u32,
    level: u32,
    game_over: bool,
    drop_timer: u64,
    last_drop: u64,
}

impl TetrisGameState {
    pub fn new(width: u16, height: u16, difficulty: u8) -> Self {
        Self {
            width,
            height,
            difficulty,
            stdout: stdout(),
            current_piece: None,
            placed_blocks: Vec::new(),
            score: 0,
            lines_cleared: 0,
            level: 1,
            game_over: false,
            drop_timer: 0,
            last_drop: 0,
        }
    }

    fn spawn_new_piece(&mut self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let piece_type = rng.gen_range(0..7);

        // 计算居中的初始位置
        // 每个方块占2个字符宽，所以位置必须是偶数
        let center_x = self.width / 2;
        let start_x = if self.width > 10 {
            // 确保是偶数位置（对齐到2的倍数）
            let x = center_x - 4;
            if x % 2 == 0 { x } else { x + 1 }
        } else {
            2 // 最小边界（偶数）
        };

        // 从顶部开始，留一些空间给方块显示
        let start_y = 1;

        let new_piece = TetrisPiece::new(start_x, start_y, piece_type);

        // Check if spawn position is valid
        if self.is_valid_position(&new_piece) {
            self.current_piece = Some(new_piece);
            true
        } else {
            // Game over - cannot spawn new piece
            false
        }
    }

    fn is_valid_position(&self, piece: &TetrisPiece) -> bool {
        let blocks = piece.get_blocks();
        self.are_blocks_valid(&blocks)
    }

    fn are_blocks_valid(&self, blocks: &[crate::ui::Coordinate]) -> bool {
        let width = self.width;
        let height = self.height;

        for block in blocks {
            // Check boundaries - 每个方块占2个字符宽，所以需要检查 x+1
            if block.x < 1 || block.x + 2 > width - 1 || block.y >= height - 1 {
                return false;
            }
        }

        // Check collision with placed blocks
        for block in blocks {
            for placed_block in &self.placed_blocks {
                if block.x == placed_block.position().x && block.y == placed_block.position().y {
                    return false;
                }
            }
        }
        true
    }

    fn try_move_piece(&mut self, delta_x: i16, delta_y: i16) -> bool {
        if let Some(ref mut piece) = self.current_piece {
            let current_pos = piece.position();

            // Calculate new position with overflow protection
            let new_x = if delta_x >= 0 {
                current_pos.x.saturating_add(delta_x as u16)
            } else {
                current_pos.x.saturating_sub((-delta_x) as u16)
            };

            let new_y = if delta_y >= 0 {
                current_pos.y.saturating_add(delta_y as u16)
            } else {
                current_pos.y.saturating_sub((-delta_y) as u16)
            };

            let mut test_piece = piece.clone();
            test_piece.move_to(new_x, new_y);

            // Extract all data needed for validation to avoid borrowing conflicts
            let blocks = test_piece.get_blocks();
            let width = self.width;
            let height = self.height;
            let placed_positions: Vec<(u16, u16)> =
                self.placed_blocks.iter().map(|b| (b.position().x, b.position().y)).collect();

            // Use static function to avoid borrowing self
            let is_valid = Self::validate_move(&blocks, width, height, &placed_positions);

            if is_valid {
                piece.move_to(new_x, new_y);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn validate_move(
        blocks: &[crate::ui::Coordinate],
        width: u16,
        height: u16,
        placed_positions: &[(u16, u16)],
    ) -> bool {
        for block in blocks {
            // Check boundaries - 每个方块占2个字符宽，所以需要检查 x+1
            if block.x < 1 || block.x + 2 > width - 1 || block.y >= height - 1 {
                return false;
            }
        }

        // Check collision with placed blocks
        for block in blocks {
            if placed_positions.contains(&(block.x, block.y)) {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    fn check_move_validity(
        &self,
        blocks: &[crate::ui::Coordinate],
        width: u16,
        height: u16,
        placed_positions: &[(u16, u16)],
    ) -> bool {
        for block in blocks {
            // Check boundaries - 每个方块占2个字符宽
            if block.x < 1 || block.x + 2 > width - 1 || block.y >= height - 1 {
                return false;
            }
        }

        // Check collision with placed blocks
        for block in blocks {
            if placed_positions.contains(&(block.x, block.y)) {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    fn check_blocks_validity(&self, blocks: &[crate::ui::Coordinate], width: u16, height: u16) -> bool {
        for block in blocks {
            // Check boundaries - 每个方块占2个字符宽
            if block.x < 1 || block.x + 2 > width - 1 || block.y >= height - 1 {
                return false;
            }
        }

        // Check collision with placed blocks
        for block in blocks {
            for placed_block in &self.placed_blocks {
                if block.x == placed_block.position().x && block.y == placed_block.position().y {
                    return false;
                }
            }
        }
        true
    }

    fn try_rotate_piece(&mut self) -> bool {
        if let Some(ref mut piece) = self.current_piece {
            let mut test_piece = piece.clone();
            test_piece.rotate();

            // Extract all data needed for validation to avoid borrowing conflicts
            let blocks = test_piece.get_blocks();
            let width = self.width;
            let height = self.height;
            let placed_positions: Vec<(u16, u16)> =
                self.placed_blocks.iter().map(|b| (b.position().x, b.position().y)).collect();

            // Use static function to avoid borrowing self
            let is_valid = Self::validate_move(&blocks, width, height, &placed_positions);

            if is_valid {
                piece.rotate();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn move_piece_down(&mut self) -> bool {
        self.try_move_piece(0, 1)
    }

    fn move_piece_down_with_score(&mut self) -> bool {
        let moved = self.move_piece_down();
        if moved {
            // 方块成功向下移动，给予软降奖励
            self.score = self.score.saturating_add(1);
        }
        moved
    }

    fn move_piece_left(&mut self) {
        // 每个方块占2个字符宽，所以移动步长为2
        self.try_move_piece(-2, 0);
    }

    fn move_piece_right(&mut self) {
        // 每个方块占2个字符宽，所以移动步长为2
        self.try_move_piece(2, 0);
    }

    fn rotate_piece(&mut self) {
        self.try_rotate_piece();
    }

    fn hard_drop(&mut self) {
        if let Some(ref mut piece) = self.current_piece {
            let current_pos = piece.position();
            let mut drop_distance = 0;

            // Extract all data needed for validation to avoid borrowing conflicts
            let width = self.width;
            let height = self.height;
            let placed_positions: Vec<(u16, u16)> =
                self.placed_blocks.iter().map(|b| (b.position().x, b.position().y)).collect();

            // Calculate how far we can drop with overflow protection
            loop {
                let mut test_piece = piece.clone();
                let new_y = current_pos.y.saturating_add(drop_distance + 1);

                // Prevent overflow by checking if we've reached the bottom
                if new_y >= height - 1 {
                    break;
                }

                test_piece.move_to(current_pos.x, new_y);

                let blocks = test_piece.get_blocks();
                let is_valid = Self::validate_move(&blocks, width, height, &placed_positions);

                if is_valid {
                    drop_distance += 1;
                } else {
                    break;
                }
            }

            // Apply the full drop and add score once
            if drop_distance > 0 {
                let final_y = current_pos.y.saturating_add(drop_distance);
                piece.move_to(current_pos.x, final_y);
                // Hard drop bonus: 2 points per cell dropped
                self.score = self.score.saturating_add(2 * drop_distance as u32);
            }
        }
    }

    fn place_piece(&mut self) {
        if let Some(piece) = &self.current_piece {
            let blocks = piece.get_blocks();
            let color_code = piece.color_code;

            for block_pos in blocks {
                self.placed_blocks.push(TetrisBlock::new(block_pos.x, block_pos.y, color_code));
            }

            self.clear_lines();
            self.current_piece = None;
        }
    }

    fn clear_lines(&mut self) {
        let mut lines_to_clear = Vec::new();

        for y in 1..self.height - 1 {
            // 计算这一行应该有多少个方块
            // 左右边界各占2个字符（[]），所以减去4，然后除以2
            // 例如：width=40时，左边界x=0-1，右边界x=38-39，可玩区域x=2-37，共36个位置，可放18个方块
            let expected_blocks = (self.width - 4) / 2;

            // 统计这一行实际有多少个方块
            let actual_blocks = self.placed_blocks.iter().filter(|block| block.position().y == y).count() as u16;

            // 如果方块数量等于预期数量，说明这一行满了
            if actual_blocks == expected_blocks {
                lines_to_clear.push(y);
            }
        }

        // Clear lines and update score
        for &line_y in lines_to_clear.iter().rev() {
            self.placed_blocks.retain(|block| block.position().y != line_y);

            // Move all blocks above down
            for block in &mut self.placed_blocks {
                if block.position().y < line_y {
                    block.move_to(block.position().x, block.position().y + 1);
                }
            }

            self.lines_cleared += 1;
            self.score = self.score.saturating_add(100 * self.level);
        }

        // Update level based on lines cleared
        self.level = 1 + (self.lines_cleared / 10);
    }

    fn handle_input(&mut self, code: KeyCode) -> Result<bool, CliError> {
        match code {
            KeyCode::Left => {
                self.move_piece_left();
            }
            KeyCode::Right => {
                self.move_piece_right();
            }
            KeyCode::Down => {
                self.move_piece_down_with_score();
            }
            KeyCode::Up => {
                self.rotate_piece();
            }
            KeyCode::Char(' ') => {
                self.hard_drop();
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.game_over = true; // 设置game_over标志，通知更新线程也退出
                return Ok(true);
            }
            _ => {}
        }
        Ok(false)
    }

    fn update(&mut self) -> Result<(), CliError> {
        self.drop_timer += 1;

        // Drop speed based on level and difficulty
        // 游戏更新频率: 每100ms一次
        // 下降间隔单位: ticks (1 tick = 100ms)
        //
        // 目标速度表:
        // | 等级 | 难度1 | 难度2 | 难度3 |
        // |------|-------|-------|-------|
        // | 1    | 3.0秒 | 2.0秒 | 1.0秒 |
        // | 5    | 1.5秒 | 1.0秒 | 0.5秒 |
        // | 10+  | 0.3秒 | 0.3秒 | 0.3秒 |
        //
        // 难度1: 等级1=30ticks -> 等级5=15ticks -> 等级10+=3ticks
        // 难度2: 等级1=20ticks -> 等级5=10ticks -> 等级10+=3ticks
        // 难度3: 等级1=10ticks -> 等级5=5ticks -> 等级10+=3ticks
        let (base_speed, level_multiplier) = match self.difficulty {
            1 => (34u64, 4u64), // 难度1: 34 - 等级*4, 等级1=30, 等级5=14, 等级8+=2
            2 => (23u64, 3u64), // 难度2: 23 - 等级*3, 等级1=20, 等级5=8, 等级7+=2
            3 => (12u64, 2u64), // 难度3: 12 - 等级*2, 等级1=10, 等级5=2
            _ => (34u64, 4u64), // 默认难度1
        };

        let drop_interval = std::cmp::max(3, base_speed.saturating_sub(self.level as u64 * level_multiplier));

        if self.current_piece.is_none() {
            if !self.spawn_new_piece() {
                self.game_over = true;
                return Err(CliError::UnknownError("Game Over!".to_owned()));
            }
        } else if self.drop_timer >= drop_interval {
            if !self.move_piece_down() {
                // 方块落地了，放置方块并立即生成新方块
                self.place_piece();
                // 立即尝试生成新方块
                if !self.spawn_new_piece() {
                    self.game_over = true;
                    return Err(CliError::UnknownError("Game Over!".to_owned()));
                }
            }
            self.drop_timer = 0;
        }

        Ok(())
    }

    fn render(&mut self) -> Result<(), CliError> {
        // 使用queue!而不是execute!来批量准备所有绘制命令
        // 这样可以减少闪烁，因为所有命令会在flush时一次性执行

        // Clear screen and move to origin
        queue!(self.stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        // Draw border
        self.draw_border()?;

        // Draw placed blocks first (behind current piece)
        // 直接遍历，避免不必要的克隆
        for block in &self.placed_blocks {
            let color = self.get_color(block.color_code);
            let pos = block.position();

            // 检查边界，确保 x+1 也在范围内（因为每个方块占2个字符宽）
            if pos.y < self.height - 1 && pos.x + 1 < self.width - 1 {
                queue!(self.stdout, cursor::MoveTo(pos.x, pos.y), Print("[]".with(color)))?;
            }
        }

        // Draw current piece
        if let Some(ref piece) = self.current_piece {
            let blocks = piece.get_blocks();
            let color = self.get_color(piece.color_code);

            for block in blocks {
                // 检查边界，确保 x+1 也在范围内（因为每个方块占2个字符宽）
                if block.y < self.height - 1 && block.x + 1 < self.width - 1 {
                    queue!(self.stdout, cursor::MoveTo(block.x, block.y), Print("[]".with(color)))?;
                }
            }
        }

        // Draw UI
        self.draw_ui()?;

        // 最后一次性flush所有队列的命令，减少闪烁
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_border(&mut self) -> Result<(), CliError> {
        // Top and bottom borders - 使用 "[]" 样式
        let mut x = 0;
        while x < self.width {
            queue!(self.stdout, cursor::MoveTo(x, 0), Print("[]".with(Color::DarkGrey)))?;
            queue!(
                self.stdout,
                cursor::MoveTo(x, self.height - 1),
                Print("[]".with(Color::DarkGrey))
            )?;
            x += 2; // 每次跳2个字符
        }

        // Side borders
        for y in 0..self.height {
            queue!(self.stdout, cursor::MoveTo(0, y), Print("[]".with(Color::DarkGrey)))?;
            // 右边界位置需要对齐到偶数位置
            let right_border_x = if self.width % 2 == 0 {
                self.width - 2
            } else {
                self.width - 1
            };
            queue!(
                self.stdout,
                cursor::MoveTo(right_border_x, y),
                Print("[]".with(Color::DarkGrey))
            )?;
        }

        Ok(())
    }

    fn draw_ui(&mut self) -> Result<(), CliError> {
        // 使用queue!而不是execute!来批量处理，减少闪烁

        // Score
        queue!(
            self.stdout,
            cursor::MoveTo(self.width + 2, 2),
            Clear(ClearType::UntilNewLine),
            Print(format!("得分: {}", self.score.to_string().blue()))
        )?;

        // Lines
        queue!(
            self.stdout,
            cursor::MoveTo(self.width + 2, 3),
            Clear(ClearType::UntilNewLine),
            Print(format!("行数: {}", self.lines_cleared.to_string().blue()))
        )?;

        // Level
        queue!(
            self.stdout,
            cursor::MoveTo(self.width + 2, 4),
            Clear(ClearType::UntilNewLine),
            Print(format!("等级: {}", self.level.to_string().blue()))
        )?;

        // Difficulty
        let difficulty_text = match self.difficulty {
            1 => "简单",
            2 => "中等",
            3 => "困难",
            _ => "未知",
        };
        queue!(
            self.stdout,
            cursor::MoveTo(self.width + 2, 5),
            Clear(ClearType::UntilNewLine),
            Print(format!("难度: {}", difficulty_text.yellow()))
        )?;

        // Controls
        queue!(self.stdout, cursor::MoveTo(self.width + 2, 7), Print("控制:".green()))?;
        queue!(self.stdout, cursor::MoveTo(self.width + 2, 8), Print("←→: 移动".dark_green()))?;
        queue!(self.stdout, cursor::MoveTo(self.width + 2, 9), Print("↑: 旋转".dark_green()))?;
        queue!(self.stdout, cursor::MoveTo(self.width + 2, 10), Print("↓: 软降".dark_green()))?;
        queue!(
            self.stdout,
            cursor::MoveTo(self.width + 2, 11),
            Print("空格: 硬降".dark_green())
        )?;
        queue!(self.stdout, cursor::MoveTo(self.width + 2, 12), Print("q: 退出".dark_green()))?;

        Ok(())
    }

    fn get_color(&self, color_code: u8) -> Color {
        match color_code {
            1 => Color::Cyan,
            2 => Color::Yellow,
            3 => Color::Magenta,
            4 => Color::Green,
            5 => Color::Red,
            6 => Color::Blue,
            7 => Color::White,
            _ => Color::White,
        }
    }
}

impl Game for TetrisGame {
    fn name(&self) -> &'static str {
        "俄罗斯方块🟦"
    }

    fn help(&self) -> &'static str {
        "使用方向键控制方块，空格键快速下降"
    }

    fn run(&self, width: u16, height: u16, difficulty: u8) -> Result<(), CliError> {
        println!("{}运行中", self.name());

        // Ensure minimum dimensions
        let game_width = std::cmp::max(20, width);
        let game_height = std::cmp::max(25, height);

        let game_state = Arc::new(Mutex::new(TetrisGameState::new(game_width, game_height, difficulty)));

        // 立即生成第一个方块
        {
            let mut state = game_state.lock().unwrap();
            if !state.spawn_new_piece() {
                println!("无法生成初始方块，游戏无法开始！");
                return Err(CliError::UnknownError("无法生成初始方块".to_owned()));
            }
        }

        // Setup terminal
        let mut stdout = stdout();
        execute!(stdout, cursor::Hide)?;
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        // 初始渲染一次
        {
            let mut state = game_state.lock().unwrap();
            state.render()?;
        }

        let g1 = game_state.clone();
        let g2 = game_state.clone();

        // Game update thread (100ms intervals)
        let update_handle = thread::spawn(move || {
            loop {
                {
                    let result = g1.lock();
                    let mut game_state = match result {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            // Handle poisoned mutex by recovering the guard
                            eprintln!("警告: 游戏状态锁被污染，尝试恢复...");
                            poisoned.into_inner()
                        }
                    };

                    if game_state.game_over {
                        break;
                    }
                    if let Err(_) = game_state.update() {
                        game_state.game_over = true;
                        break;
                    }
                }
                sleep(Duration::from_millis(100));
            }
        });

        // Input and render thread (50ms intervals)
        let mut error = None;
        loop {
            {
                let result = g2.lock();
                let mut game_state = match result {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        // Handle poisoned mutex by recovering the guard
                        eprintln!("警告: 游戏状态锁被污染，尝试恢复...");
                        poisoned.into_inner()
                    }
                };

                if game_state.game_over {
                    break;
                }

                // Handle input
                if let Some(code) = poll_input()? {
                    if game_state.handle_input(code)? {
                        break;
                    }
                }

                // Render
                if let Err(e) = game_state.render() {
                    error = Some(e);
                    break;
                }
            }
            sleep(Duration::from_millis(50));
        }

        // Cleanup
        update_handle.join().unwrap_or_else(|_| {});

        // 恢复终端状态 - 使用独立的execute调用来确保每个操作都成功
        let _ = execute!(stdout, LeaveAlternateScreen);
        let _ = execute!(stdout, cursor::Show);
        let _ = terminal::disable_raw_mode();

        if let Some(err) = error {
            return Err(err);
        }

        // 安全地获取最终分数
        let final_score = match game_state.lock() {
            Ok(guard) => guard.score,
            Err(poisoned) => {
                eprintln!("警告: 无法获取最终分数");
                poisoned.into_inner().score
            }
        };

        println!("游戏结束！最终得分: {}", final_score);
        Ok(())
    }
}
