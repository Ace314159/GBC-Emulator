pub mod gbc;

use gbc::GBC;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();
    let gbc = GBC::new(&"Tetris.gb".to_string());
    loop {
        gbc.emulate();
    }
}
