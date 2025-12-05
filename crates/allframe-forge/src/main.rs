//! AllFrame CLI binary
//!
//! This is a thin wrapper that calls the library's `run()` function.

fn main() -> anyhow::Result<()> {
    allframe_forge::run()
}
