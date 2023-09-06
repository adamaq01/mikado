use std::error::Error;
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    EmitBuilder::builder()
        .git_describe(false, false, None)
        .git_sha(false)
        .build_date()
        .emit()?;
    Ok(())
}
