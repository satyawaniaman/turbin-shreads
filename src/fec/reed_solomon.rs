use reed_solomon_erasure::galois_8::ReedSolomon;

pub fn recover_shards(
    num_data: usize,
    num_coding: usize,
    shards: &mut [Option<Vec<u8>>],
) -> anyhow::Result<()> {
    if num_data == 0 || num_coding == 0 {
        anyhow::bail!("Invalid FEC parameters: num_data={num_data}, num_coding={num_coding}");
    }

    let rs = ReedSolomon::new(num_data, num_coding)
        .map_err(|e| anyhow::anyhow!("Failed to create ReedSolomon encoder: {e}"))?;

    rs.reconstruct(shards)
        .map_err(|e| anyhow::anyhow!("Reed-Solomon recovery failed: {e}"))?;

    Ok(())
}
