use crate::error::CliError;
use crate::impls::games::gomoku::game::GomokuGame;
use crate::impls::games::tetris::game::TetrisGame;
use crate::impls::games::thunder_fighter::game::ThunderFighterGame;
use crate::impls::handlers::CommandHandler;
use clap::Parser;
use crossterm::style::Stylize;
use std::fmt::{Display, Formatter};
use std::sync::OnceLock;

#[derive(Debug, Parser)]
pub struct GameHandler {
    #[arg(short, long, help = "请选择游戏1-6")]
    select: Option<u8>,

    #[arg(short, long, default_value_t = 80, help = "游戏宽度")]
    width: u16,

    #[arg(short = 'H', long, default_value_t = 30, help = "游戏高度")]
    height: u16,

    #[arg(short, long, default_value_t = 1, help = "游戏难度1-3")]
    difficulty: u8,
}
//Send + Sync 确保可以跨线程调用
pub trait Game: Send + Sync {
    fn name(&self) -> &'static str;

    fn help(&self) -> &'static str;

    fn run(&self, width: u16, height: u16, difficulty: u8) -> Result<(), CliError>;
}
impl Display for dyn Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\t游戏名：{} \t 玩法说明：{}", self.name().blue(), self.help().dark_blue())
    }
}
static GAME_REGISTRY: OnceLock<Vec<Box<dyn Game>>> = OnceLock::new();
fn init_game_list() -> Vec<Box<dyn Game>> {
    vec![
        Box::new(ThunderFighterGame {}),
        Box::new(TetrisGame {}),
        Box::new(GomokuGame {}),
    ]
}
fn get_game_list() -> &'static Vec<Box<dyn Game>> {
    GAME_REGISTRY.get_or_init(init_game_list)
}

impl CommandHandler for GameHandler {
    fn run(&self) -> Result<(), CliError> {
        let game_list = get_game_list();
        if self.select.is_none() {
            for (index, game) in game_list.iter().enumerate() {
                println!("{}. {}", index + 1, game.name());
            }
            return Ok(());
        }
        let select = self.select.unwrap();
        if select > game_list.len() as u8 || select < 1 {
            return Err(CliError::UnknownError(format!("游戏序号{}不存在", select)));
        }
        let game = game_list.get(select as usize - 1).unwrap();
        println!("🎮 启动游戏中 {}", game);
        println!("🖥 分辨率：{}x{}，难度：{}", self.width, self.height, self.difficulty);
        println!("{}", "按q退出游戏".green());

        game.run(self.width, self.height, self.difficulty)?;
        Ok(())
    }
}
