use crate::error::CliError;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEventKind};
use std::time::Duration;

/// 非阻塞输入轮询
pub fn poll_input() -> Result<Option<KeyCode>, CliError> {
    // 非阻塞输入轮询
    if event::poll(Duration::from_millis(100))? {
        if let event::Event::Key(key_event) = event::read()? {
            // 只处理按下事件
            if key_event.kind == KeyEventKind::Press {
                return Ok(Some(key_event.code));
            }
        }
    }
    Ok(None)
}
