use crypto::hash::Hash256;

pub enum TXOutTarget {
    ToKey {
        // key: public_key
    }
}

pub enum TXIn {
    Gen {
        height: u64
    },
    FromKey {
        amount: u64,
        key_offsets: Vec<u64>,
        // key_image: key_image
    }
}

pub struct TXOut {
    pub amount: u64,
    pub target: TXOutTarget
}

pub struct TransactionPrefix {
    pub version: usize,
    pub unlock_delta: u16,
    pub inputs: Vec<TXIn>,
    pub outputs: Vec<TXOut>,
    pub extra: Vec<u8>
}

pub struct Transaction {
    pub prefix: TransactionPrefix,
    pub signatures: Vec<Vec<Hash256>>
}
