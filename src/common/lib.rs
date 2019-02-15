extern crate hex;

pub enum TXOut {
    ToScript {
        // keys: Vec<public_key>,
        script: Vec<u8>
    },
    ToScriptHash(),
    ToKey()
}

pub mod checkpoints;
