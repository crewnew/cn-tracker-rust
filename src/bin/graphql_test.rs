use timetrackrs::graphql::*;

fn main() -> anyhow::Result<()> {
    dbg!(get_user_rules()?);

    Ok(())
}
