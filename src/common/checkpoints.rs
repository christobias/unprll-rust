use crypto::hash::Hash256;

pub struct Checkpoint {
    pub height: u64,
    pub hash: Hash256
}

struct Checkpoints {
    checkpoints: Vec<Checkpoint>
}

impl Checkpoints {
    pub fn new() -> Checkpoints {
        Checkpoints {
            checkpoints: Vec::<Checkpoint>::new()
        }
    }
    pub fn add_checkpoint(height: u64, hash: String) {

    }
}
