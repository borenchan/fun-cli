use crate::cli::Commands;
use crate::error::CliError;
///
/// 命令行输入的处理器
/// [`SubCommand`](clap::Subcommand) 应该实现此trait
pub trait CommandHandler {
    /// 执行命令
    ///
    /// 例如打印表格或AscII字符到控制台
    fn run(&self) -> Result<(), CliError>;
}
///  组合处理器
pub struct CombineHandler {}

impl CombineHandler {
    pub fn new() -> Self {
        CombineHandler {}
    }
    /// 匹配命令对应的处理器
    pub fn matches_handler(self, cmd: Commands) -> Result<Box<dyn CommandHandler>, CliError> {
        match cmd {
            Commands::Weather(handler) => Ok(Box::new(handler)),
            Commands::Music(handler) => Ok(Box::new(handler)),
            Commands::Curl(handler) => Ok(Box::new(handler)),
            Commands::Game(handler) => Ok(Box::new(handler)),
            Commands::Os(handler) => Ok(Box::new(handler)),
            _ => Err(CliError::NoMatchHandlerError),
        }
    }
}
