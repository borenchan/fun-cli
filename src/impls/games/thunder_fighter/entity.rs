use crate::error::CliError;
use crate::impls::games::entities::{Entity, GameEntity};
use crate::impls::games::thunder_fighter::game::{ENEMY_BULLET, ENEMY_DISPLAY, PLAYER_BULLET};
use crate::ui::Coordinate;
use crossterm::style::{Color, Print, Stylize};
use crossterm::{cursor, queue};
use rand::Rng;
use std::io::Stdout;
use unicode_width::UnicodeWidthStr;

//玩家
pub struct Player {
    pub entity: Entity,
    pub health: i16,
    pub bullets: Vec<Bullet>,
}

pub struct Bullet {
    pub entity: Entity,
    pub speed: u16,
}

//敌人
pub struct Enemy {
    pub entity: Entity,
    pub health: i16,
    pub bullets: Vec<Bullet>,
}
impl Player {
    pub fn render(
        &mut self,
        stdout: &mut Stdout,
        max_bound: u16,
    ) -> Result<(), CliError> {
        for bullet in self.bullets.iter_mut() {
            bullet.render(stdout, max_bound, Color::DarkYellow, None)?;
        }
        self.bullets.retain(|bullet| !bullet.is_out_of_bound(max_bound));
        // 清除玩家上一次位置
        queue!(
            stdout,
            cursor::MoveTo(self.last_position().x, self.last_position().y),
            Print(" ".repeat(self.width() as usize))
        )?;
        // 绘制玩家
        queue!(
            stdout,
            cursor::MoveTo(self.position().x, self.position().y),
            Print(self.display().yellow())
        )?;
        Ok(())
    }

    pub fn attack_bullet(&mut self) {
        let coordinate = self.position();
        let mut y = coordinate.y;
        if y >= 1 {
            y -= 1;
        }
        let bullet = Bullet {
            entity: Entity {
                x: coordinate.x,
                y,
                display: PLAYER_BULLET.to_string(),
                width: UnicodeWidthStr::width(PLAYER_BULLET) as u16,
                last_x: coordinate.x,
                last_y: y,
            },
            speed: 1,
        };
        self.bullets.push(bullet);
        /*        queue!(
            stdout,
            cursor::MoveTo(coordinate.x, y),
            Print(PLAYER_BULLET.to_string().green())
        )?;*/
    }
    ///
    /// 基于子弹速度移动
    pub fn update_bullets_by_speed(
        &mut self,
        max_bound: u16,
    ) {
        for bullet in &mut self.bullets {
            let mut y = bullet.position().y;
            if y < bullet.speed {
                y = 0;
            } else {
                y -= bullet.speed;
            }
            bullet.move_to(bullet.position().x, y);
        }
    }
}
impl GameEntity for Player {
    fn position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.x,
            y: self.entity.y,
        }
    }

    fn last_position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.last_x,
            y: self.entity.last_y,
        }
    }

    fn move_to(
        &mut self,
        x: u16,
        y: u16,
    ) {
        self.entity.last_x = self.entity.x;
        self.entity.last_y = self.entity.y;
        self.entity.x = x;
        self.entity.y = y;
    }

    fn display(&self) -> &str {
        self.entity.display.as_str()
    }
    fn width(&self) -> u16 {
        self.entity.width
    }
}
impl Enemy {
    /// Update the enemy position
    pub fn update(
        &mut self,
        max_bound: u16,
    ) {
        let mut y = self.position().y;
        if y == max_bound {
            self.health = 0;
        } else {
            y += 1;
        }
        self.move_to(self.entity.x, y);
    }
    pub fn render(
        &mut self,
        stdout: &mut Stdout,
        max_bound: u16,
        enemy_entity: &dyn GameEntity,
    ) -> Result<(), CliError> {
        for bullet in self.bullets.iter_mut() {
            bullet.render(stdout, max_bound, Color::DarkMagenta, Some(enemy_entity))?;
        }
        self.bullets
            .retain(|bullet| !bullet.is_out_of_bound(max_bound) && !bullet.coll_detect(enemy_entity));
        // 清除敌人上一次位置
        queue!(
            stdout,
            cursor::MoveTo(self.last_position().x, self.last_position().y),
            Print(" ".repeat(self.width() as usize))
        )?;
        if !self.is_dead() {
            // 绘制敌人
            queue!(
                stdout,
                cursor::MoveTo(self.position().x, self.position().y),
                Print(self.display().red())
            )?;
        }
        Ok(())
    }

    pub fn new_random_enemy(
        width: u16,
        height: u16,
    ) -> Enemy {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..width / 2);
        let y = rng.gen_range(0..height / 3);
        let display = ENEMY_DISPLAY[rng.gen_range(0..ENEMY_DISPLAY.len())];
        Enemy {
            entity: Entity {
                x,
                y,
                display: display.to_string(),
                width: UnicodeWidthStr::width(display) as u16,
                last_x: x,
                last_y: y,
            },
            health: 100,
            bullets: vec![],
        }
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0
    }
    pub fn attack_bullet(
        &mut self,
        max_bound: u16,
    ) {
        let coordinate = self.position();
        let mut y = self.position().y;
        if y < max_bound {
            y += 1;
        }
        let bullet = Bullet {
            entity: Entity {
                x: coordinate.x,
                y,
                display: ENEMY_BULLET.to_string(),
                width: UnicodeWidthStr::width(PLAYER_BULLET) as u16,
                last_x: coordinate.x,
                last_y: y,
            },
            speed: 1,
        };
        self.bullets.push(bullet);
        /*        queue!(
            stdout,
            cursor::MoveTo(coordinate.x, y),
            Print(PLAYER_BULLET.to_string().green())
        )?;*/
    }
    /// 基于子弹速度移动
    pub fn update_bullets_by_speed(
        &mut self,
        max_bound: u16,
    ) {
        for bullet in &mut self.bullets {
            let mut y = bullet.position().y;
            if y + bullet.speed >= max_bound {
                y = max_bound;
            } else {
                y += bullet.speed;
            }
            bullet.move_to(bullet.position().x, y);
        }
    }
}
impl GameEntity for Enemy {
    fn position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.x,
            y: self.entity.y,
        }
    }

    fn last_position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.last_x,
            y: self.entity.last_y,
        }
    }

    fn move_to(
        &mut self,
        x: u16,
        y: u16,
    ) {
        self.entity.last_x = self.entity.x;
        self.entity.last_y = self.entity.y;
        self.entity.x = x;
        self.entity.y = y;
    }
    fn display(&self) -> &str {
        self.entity.display.as_str()
    }
    fn width(&self) -> u16 {
        self.entity.width
    }
}
impl Bullet {
    pub fn render(
        &mut self,
        stdout: &mut Stdout,
        max_bound: u16,
        color: Color,
        enemy_entity: Option<&dyn GameEntity>,
    ) -> Result<(), CliError> {
        // 清除子弹上一次位置
        queue!(
            stdout,
            cursor::MoveTo(self.last_position().x, self.last_position().y),
            Print(" ".repeat(self.width() as usize))
        )?;
        if self.is_out_of_bound(max_bound) || (enemy_entity.is_some() && self.coll_detect(enemy_entity.unwrap())) {
            return Ok(());
        }
        // 绘制子弹
        queue!(
            stdout,
            cursor::MoveTo(self.position().x, self.position().y),
            Print(self.display().with(color))
        )?;
        Ok(())
    }
    /// 判断子弹是否超出屏幕边界
    pub fn is_out_of_bound(
        &self,
        max_bound: u16,
    ) -> bool {
        self.last_position().y == 0 || self.last_position().y >= max_bound
    }
}
impl GameEntity for Bullet {
    fn position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.x,
            y: self.entity.y,
        }
    }

    fn last_position(&self) -> Coordinate {
        Coordinate {
            x: self.entity.last_x,
            y: self.entity.last_y,
        }
    }

    fn move_to(
        &mut self,
        x: u16,
        y: u16,
    ) {
        self.entity.last_x = self.entity.x;
        self.entity.last_y = self.entity.y;
        self.entity.x = x;
        self.entity.y = y;
    }

    fn display(&self) -> &str {
        self.entity.display.as_str()
    }

    fn width(&self) -> u16 {
        self.entity.width
    }
}
