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
#[derive(Copy, Clone, PartialEq)]
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

/// the buffer that is sended to the clients, width, height
pub type YardBuf = Vec<Vec<TUIBlock>>;

pub fn width(b: &YardBuf) -> usize {
    b[0].len()
}

pub fn height(b: &YardBuf) -> usize {
    b.len()
}

pub struct TUIHelper {
    pub is_init: bool,
    pub buf: YardBuf,
}

impl TUIHelper {
    pub fn new() -> TUIHelper {
        return TUIHelper { is_init: false, buf: YardBuf::new() };
    }

    pub fn print_yard(&self) -> Result<()> {
        stdout()
            .execute(Clear(ClearType::All))?
            .execute(cursor::MoveTo(0, 0))?;
        let fence_block = TUIBlock { fg: Color::Black, bg: Color::Grey, content: FENCE };
        for _i in 0..(width(&self.buf) + 2) {
            put_tui_block(&fence_block)?;
        }
        stdout().queue(Print("\n"))?;
        for r in &self.buf {
            put_tui_block(&fence_block)?;
            for c in r {
                put_tui_block(c)?;
            }
            put_tui_block(&fence_block)?;
            stdout().queue(Print("\n"))?;
        }
        for _i in 0..(width(&self.buf) + 2) {
            put_tui_block(&fence_block)?;
        }
        stdout().queue(Print("\n"))?;
        Ok(())
    }

    pub fn refresh_yard(&mut self, nbuf: YardBuf) -> Result<()> {
        if !self.is_init || width(&nbuf) != width(&self.buf) || height(&nbuf) != height(&self.buf) {
            self.buf = nbuf;
            self.is_init = true;
            return self.print_yard();
        }
        for r in 0..height(&self.buf) {
            for c in 0..width(&self.buf) {
                if self.buf[r][c] != nbuf[r][c] {
                    stdout().execute(cursor::MoveTo(
                        (c * 2 + 2).try_into().unwrap(),
                        (r + 1).try_into().unwrap(),
                    ))?;
                    put_tui_block(&nbuf[r][c])?;
                }
            }
        }
        self.buf = nbuf;
        Ok(())
    }
}