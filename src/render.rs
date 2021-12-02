/// pub mod render: utils drawing the gameplay on the terminal

pub use std::io::{stdout};

pub use crossterm::{
    ExecutableCommand, QueueableCommand, Result,
    terminal::{Clear, ClearType}, cursor,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

/// two character-wide basic TUI blocks, which may appear in the game
pub const HEAD_L: &str  = ": ";
pub const HEAD_R: &str  = " :";
pub const HEAD_U: &str  = "''";
pub const HEAD_D: &str  = "..";
pub const BEAN: &str    = "()";
pub const FENCE: &str   = "[]";
pub const EMPTY: &str   = "  ";
pub struct TUIBlock {
    pub fg: Color,
    pub bg: Color,
    pub content: &'static str, // shall be anyone declared above
}

// print a block after the curser
pub fn put_tui_block(block: &TUIBlock) -> Result<()> {
    stdout()
        .execute(SetForegroundColor(block.fg))?
        .execute(SetBackgroundColor(block.bg))?
        .execute(Print(block.content))?
        .execute(ResetColor)?;
    Ok(())
}