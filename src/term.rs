use std::io::{self, Write};

pub fn clear_screen() {
    print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
    io::stdout().flush().unwrap();
}
