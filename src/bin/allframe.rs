//! AllFrame CLI binary
//!
//! This is a thin wrapper that calls the allframe-forge library's `run()`
//! function. It enables `cargo install allframe` to provide the `allframe` CLI
//! tool.

fn main() -> anyhow::Result<()> {
    allframe_forge::run()
}
