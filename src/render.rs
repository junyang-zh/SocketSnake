/// render: utils drawing the gameplay on the terminal
pub mod render {

    use std::io::{stdout};

    use crossterm::{
        ExecutableCommand,
        Result,
        terminal::Clear,
        style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    };

    /// two character-wide basic blocks, which may appear in the game
    pub const HEAD_L: &str  = ": ";
    pub const HEAD_R: &str  = " :";
    pub const HEAD_U: &str  = "''";
    pub const HEAD_D: &str  = "..";
    pub const BEAN: &str    = "()";
    pub const FENCE: &str   = "[]";
    pub struct TUIBlock {
        fg : Color,
        bg : Color,
        content : &'static str,
    }

    // print a block after the curser
    pub fn put_tui_block(block : &TUIBlock) -> Result<()> {
        stdout()
            .execute(SetForegroundColor(block.fg))?
            .execute(SetBackgroundColor(block.bg))?
            .execute(Print(block.content))?
            .execute(ResetColor)?;
        
        Ok(())
    }

    #[test]
    fn test_put() {
        put_tui_block(&TUIBlock { fg: Color::White, bg: Color::Cyan, content: BEAN } ).unwrap();
    }

}