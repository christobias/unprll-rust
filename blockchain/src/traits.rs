/// Trait to define the emission curve of a coin
pub trait EmissionCurve {
    /// Returns the block reward for a block given a set of existing conditions
    fn get_block_reward(&self, version: u8) -> u64;
}
