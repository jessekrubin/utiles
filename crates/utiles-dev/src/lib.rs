use anyhow::Result;
pub fn quick_maths() -> Result<()> {
    let mut n = 2; // 2
    n += 2; // plus 2
    if n != 4 {
        anyhow::bail!("quick-maths failed");
    }
    Ok(())
}
