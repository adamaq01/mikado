use std::error::Error;
use vergen_gitcl::{Build, Emitter, Gitcl};

fn main() -> Result<(), Box<dyn Error>> {
    let build = Build::builder().build_date(true).build();
    let gitcl = Gitcl::builder()
        .describe(false, false, None)
        .sha(false)
        .build();

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&gitcl)?
        .emit()?;

    Ok(())
}
