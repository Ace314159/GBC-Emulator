pub mod gbc;

use gbc::GBC;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();
    let mut gbc = GBC::new(&"cpu_instrs/01-special.gb".to_string());
    loop {
        gbc.emulate();
    }
}
