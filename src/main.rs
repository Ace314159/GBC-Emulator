pub mod gbc;

use gbc::GBC;

fn main() {
    std::env::set_current_dir("ROMs").unwrap();
    let mut gbc = GBC::new(&"cpu_instrs/cpu_instrs.gb".to_string());
    loop {
        gbc.emulate();
    }
}
