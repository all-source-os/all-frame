const COMMANDS: &[&str] = &[
    "allframe_list",
    "allframe_call",
    "allframe_stream",
    "allframe_stream_cancel",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
