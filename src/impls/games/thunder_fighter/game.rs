use crate::error::CliError;
use crate::impls::game::Game;
use crate::impls::games::entities::{Entity, GameEntity};
use crate::impls::games::thunder_fighter::entity::{Enemy, Player};
use crate::ui::event::poll_input;
use crate::utils::consts;
use crossterm::event::KeyCode;
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, terminal};
use std::io::{Stdout, Write, stdout};
use std::ops::Deref;
use std::panic::catch_unwind;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

pub struct ThunderFighterGame;
struct ThunderFighterGameState {
    width: u16,
    height: u16,
    difficulty: u8,
    stdout: Stdout,
    player: Player,
    enemies: Vec<Enemy>,
    score: u16,
    frame_count: u128,
}
pub const PLAYER_DISPLAY: &str = "☮";
pub const PLAYER_BULLET: &str = "♝";
pub const ENEMY_BULLET: &str = "✦";
//"🐱‍🚀" ★☮♝♗ ♕ i⛴⛟✈♞☢♛
// pub const ENEMY_DISPLAY: [&str; 4] = ["🐱‍👤", "‍👓", "🐱‍💻", "🐱‍🐉"];
pub const ENEMY_DISPLAY: [&str; 6] = ["⛴", "‍⛟", "✈", "♞", "☢", "♛"];

impl ThunderFighterGameState {
    pub fn new(width: u16, height: u16, difficulty: u8) -> Self {
        Self {
            width,
            height,
            difficulty,
            stdout: stdout(),
            player: Player {
                entity: Entity {
                    x: width / 2,
                    y: height - height / 4,
                    display: PLAYER_DISPLAY.to_string(),
                    width: UnicodeWidthStr::width(PLAYER_DISPLAY) as u16,
                    last_x: width / 2,
                    last_y: height / 2,
                },
                health: 100,
                bullets: vec![],
            },
            enemies: vec![],
            score: 0,
            frame_count: 0,
        }
    }

    ///
    /// 处理玩家输入
    /// 返回true表示退出游戏
    fn handle_input(&mut self, code: KeyCode) -> Result<bool, CliError> {
        match code {
            KeyCode::Up => {
                let mut y = self.player.position().y;
                if y >= 1 {
                    y -= 1;
                }
                self.player.move_to(self.player.position().x, y)
            }
            KeyCode::Down => {
                let mut y = self.player.position().y;
                if y + 1 <= self.height {
                    y += 1;
                }
                self.player.move_to(self.player.position().x, y)
            }
            KeyCode::Left => {
                let width = self.player.width();
                let mut x = self.player.position().x;
                if x >= width {
                    x -= width
                }
                self.player.move_to(x, self.player.position().y)
            }
            KeyCode::Right => {
                let width = self.player.width();
                let mut x = self.player.position().x;
                if x + width <= self.width {
                    x += width
                }
                self.player.move_to(x, self.player.position().y)
            }
            KeyCode::Char(' ') => {
                self.player.attack_bullet();
            }
            KeyCode::Char('q') => {
                return Ok(true);
            }
            _ => {}
        }
        Ok(false)
    }

    /// 更新地图上实体的位置
    fn update_enemies(&mut self) {
        //攻击频率
        let diff_rx = 1.0 / f32::from(self.difficulty);
        let is_attack_frame = self.frame_count % ((4.0 * diff_rx * 10.0) as u128) == 0;
        let is_move_frame = self.frame_count % ((1.0 * diff_rx * 10.0) as u128) == 0;

        for enemy in self.enemies.iter_mut() {
            //enemy.update(self.height);
            if is_move_frame {
                enemy.update_bullets_by_speed(self.height);
            }

            if is_attack_frame {
                enemy.attack_bullet(self.height);
            }
            //碰撞检测
            if enemy.deref().coll_detect(&self.player) {
                self.player.health -= 10;
            }
            for bullet in &enemy.bullets {
                if bullet.coll_detect(&self.player) {
                    self.player.health -= 10;
                }
            }

            if enemy.is_dead() {
                self.score += 1;
            }
        }
        let max_enemy_count = self.difficulty * 10;
        if is_attack_frame && self.enemies.len() < max_enemy_count as usize {
            let random_enemy = Enemy::new_random_enemy(self.width, self.height);
            self.enemies.push(random_enemy);
        }
    }

    /// 更新玩家信息
    fn update_player(&mut self) {
        self.player.update_bullets_by_speed(self.height);
        //碰撞检测
        for enemy in self.enemies.iter_mut() {
            for bullet in &self.player.bullets {
                if bullet.coll_detect(enemy.deref()) {
                    enemy.health -= 10;
                }
            }
        }
    }

    /// 渲染玩家
    fn render_player(&mut self) -> Result<(), CliError> {
        self.player.render(&mut self.stdout, self.height)?;
        self.render_player_score()?;
        // 刷新到终端
        self.stdout.flush()?;
        Ok(())
    }

    /// 游戏实体渲染
    fn render_enemies(&mut self) -> Result<(), CliError> {
        // // 每30帧清一次屏
        // if self.frame_count % 60 == 0 {
        //     execute!(self.stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?
        // }
        for enemy in self.enemies.iter_mut() {
            enemy.render(&mut self.stdout, self.height, &self.player)?;
        }
        //更新敌人数量
        self.enemies.retain(|enemy| !enemy.is_dead());
        // 刷新到终端
        self.stdout.flush()?;
        Ok(())
    }

    /// 渲染分数
    fn render_player_score(&mut self) -> Result<(), CliError> {
        execute!(
            self.stdout,
            cursor::MoveTo(self.width + 2, 0),
            Clear(ClearType::UntilNewLine),
            Print(format!("得分🥇：{}", self.score.to_string().blue()))
        )?;
        execute!(
            self.stdout,
            cursor::MoveTo(self.width + 2, 1),
            Clear(ClearType::UntilNewLine),
            Print(format!("生命🩸：{}", self.player.health.to_string().blue()))
        )?;
        if self.player.health <= 0 {
            execute!(
                self.stdout,
                cursor::MoveTo(self.width / 2, self.height / 2),
                Print(consts::GAME_OVER.red())
            )?;
            return Err(CliError::UnknownError("Game Over!".to_owned()));
        }
        Ok(())
    }
}

impl Game for ThunderFighterGame {
    fn name(&self) -> &'static str {
        "雷霆战机✈️"
    }

    fn help(&self) -> &'static str {
        "按上下左右操作战机，空格键发射"
    }

    fn run(&self, width: u16, height: u16, difficulty: u8) -> Result<(), CliError> {
        println!("{}运行中", self.name());
        let game = Arc::new(Mutex::new(ThunderFighterGameState::new(
            width, height, difficulty,
        )));
        // 进入全屏 EnterAlternateScreen
        let mut stdout = stdout();
        execute!(stdout, cursor::Hide)?;
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            EnterAlternateScreen,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        let g1 = game.clone();
        {
            let mut game_state = g1.lock().unwrap();
            game_state.render_player_score()?;
        }
        let (tx, rx) = channel();
        // 定时刷新敌人
        thread::spawn(move || {
            loop {
                //捕获panic!
                let result = catch_unwind(|| {
                    let mut game_state = g1.lock().unwrap();
                    game_state.update_enemies();
                    game_state.render_enemies().unwrap();
                    game_state.frame_count += 1;
                });

                if result.is_err() {
                    tx.send("games update error!".to_owned()).unwrap();
                }

                //控制刷新帧率
                sleep(Duration::from_millis(100));
            }
        });
        let g2 = game.clone();
        let mut error = None;
        //接收玩家输入并渲染
        loop {
            if let Ok(err) = rx.try_recv() {
                error = Some(CliError::UnknownError(err));
                break;
            }
            let mut game_state = g2.lock().unwrap();
            //接收输入
            if let Some(code) = poll_input()? {
                if game_state.handle_input(code)? {
                    break;
                }
            }
            game_state.update_player();
            // 绘制
            if let Err(e) = game_state.render_player() {
                error = Some(e);
                break;
            }
            drop(game_state);
            // 控制刷新帧率
            sleep(Duration::from_millis(20));
        }
        //恢复终端
        execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        // println!("游戏结束！最终分数: {}", game_state.score);
        if error.is_some() {
            return Err(error.unwrap());
        }
        Ok(())
    }
}
