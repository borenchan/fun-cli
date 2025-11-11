use crate::impls::curl::CurlHandler;
use crate::impls::game::GameHandler;
use crate::impls::handlers::CombineHandler;
use crate::impls::music::MusicHandler;
use crate::impls::os::OsHandler;
use crate::impls::weather::WeatherHandler;
use crate::utils::consts::BANNER;
use clap::{Parser, Subcommand};
use crossterm::style::Stylize;

#[derive(Debug, Parser)]
#[command(name = "fun", author, version, about, long_about = BANNER )]
pub struct FunCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    //天气系统
    #[command(name = "weather", version, about = "天气系统")]
    Weather(WeatherHandler),
    //音乐系统
    #[command(name = "music", version, about = "音乐系统")]
    Music(MusicHandler),

    #[command(name = "curl", version, about = "curl系统")]
    Curl(CurlHandler),

    #[command(name = "game", version, about = "游戏系统")]
    Game(GameHandler),

    #[command(name = "os", version, about = "操作系统")]
    Os(OsHandler),
}

impl Commands {
    /// 执行子命令
    pub fn run(self) {
        let combine_handlers = CombineHandler::new();
        match combine_handlers.matches_handler(self) {
            Ok(handler) => {
                if let Err(cli_err) = handler.run() {
                    eprintln!("{}: {}", "error".red().bold(), cli_err.to_string().italic());
                }
            }
            Err(cli_err) => {
                eprintln!("{}: {}", "error".red().bold(), cli_err.to_string().italic());
            }
        }
    }
}
