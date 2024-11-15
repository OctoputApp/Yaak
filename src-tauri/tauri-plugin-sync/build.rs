const COMMANDS: &[&str] = &["changes", "commit", "commits"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
