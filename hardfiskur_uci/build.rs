use vergen_gitcl::{Emitter, GitclBuilder};

fn main() -> anyhow::Result<()> {
    let gitcl = GitclBuilder::all_git()?;

    Emitter::default().add_instructions(&gitcl)?.emit()
}
