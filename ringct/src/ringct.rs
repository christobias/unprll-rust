use std::iter::Sum;

use failure::format_err;
use serde::{
    Serialize,
    Deserialize
};

use crypto::{
    CNFastHash,
    curve25519_dalek::traits::MultiscalarMul,
    Digest,
    ecc::{
        self,
        BASEPOINT,
        Point
    },
    Hash256,
    PublicKey,
    SecretKey
};

use crate::{
    bulletproof::{
        self,
        Bulletproof
    },
    Matrix,
    MASK_BASEPOINT,
    mlsag::{
        self,
        Signature as MLSAGSignature
    }
};

mod ecdh_utils {
    use crypto::{
        CNFastHash,
        Digest,
        ecc,
        SecretKey
    };

    pub fn ecdh_hash(key: &SecretKey) -> SecretKey {
        let mut hasher = CNFastHash::new();
        hasher.input(b"amount");
        hasher.input(key.as_bytes());

        ecc::hash_to_scalar(hasher.result())
    }
}

#[derive(Serialize, Deserialize)]
/// ECDH encoded tuple of amount and mask
pub struct ECDHTuple {
    /// Encoded Amount in bytes
    pub amount: Vec<u8>,
    /// Encoded Mask
    pub mask: SecretKey
}

/// Pedersen Commitments
///
/// `C = aG + bH`
#[derive(Clone, Serialize, Deserialize)]
pub struct Commitment {
    /// The amount being transacted `b`
    amount: SecretKey,
    /// The blinding factor `a`
    mask: SecretKey
}

impl Commitment {
    /// Generate a commitment to the given value using a random mask
    pub fn commit_to_value(value: u64) -> Commitment {
        Commitment {
            amount: SecretKey::from(value),
            mask: SecretKey::random(&mut rand::rngs::OsRng)
        }
    }

    /// Returns the result of the commitment
    ///
    /// Computes `C` where `C = aG + bH`
    pub fn to_public(self) -> Point {
        Point::multiscalar_mul(
            &[self.mask, self.amount],
            &[BASEPOINT, *MASK_BASEPOINT]
        ).mul_by_cofactor()
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum RingCTType {
    Null = 0,
    Bulletproof = 3,
    Bulletproof2 = 4
}

#[derive(Serialize, Deserialize)]
pub struct RingCTBase {
    pub signature_type: RingCTType,
    pub message_hash: Hash256,
    pub mix_ring: Matrix<(PublicKey, PublicKey)>,
    pub output_commitments: Vec<(PublicKey, PublicKey)>,
    pub ecdh_exchange: Vec<ECDHTuple>,
    pub fee: u64
}

#[derive(Serialize, Deserialize)]
pub struct RingCTSignature {
    pub base: RingCTBase,
    pub bulletproofs: Vec<Bulletproof>,
    pub input_commitments: Vec<PublicKey>,
    pub mlsag: Vec<MLSAGSignature>
}

fn get_pre_mlsag_hash(signature: &RingCTSignature) -> Vec<u8> {
    let mut hasher = CNFastHash::new();

    hasher.input(&[signature.base.signature_type.clone() as u8]);
    if signature.base.signature_type != RingCTType::Null {
        // Base
        hasher.input(bincode_epee::serialize(&signature.base.fee).unwrap());
        for ecdh in &signature.base.ecdh_exchange {
            hasher.input(ecdh.mask.as_bytes());
            hasher.input(&ecdh.amount);
        }
        for (_, commitment) in &signature.base.output_commitments {
            hasher.input(commitment.as_bytes());
        }

        let base_hash = hasher.result_reset();

        let mut bulletproof_hash = CNFastHash::new();
        for proof in &signature.bulletproofs {
            bulletproof_hash.input(proof.A.compress().to_bytes());
            bulletproof_hash.input(proof.S.compress().to_bytes());
            bulletproof_hash.input(proof.T_1.compress().to_bytes());
            bulletproof_hash.input(proof.T_2.compress().to_bytes());
            bulletproof_hash.input(proof.tau_x.to_bytes());
            bulletproof_hash.input(proof.mu.to_bytes());
            for l in &proof.L {
                bulletproof_hash.input(l.compress().to_bytes());
            }
            for r in &proof.R {
                bulletproof_hash.input(r.compress().to_bytes());
            }
            bulletproof_hash.input(proof.a.to_bytes());
            bulletproof_hash.input(proof.b.to_bytes());
            bulletproof_hash.input(proof.t.to_bytes());
        }

        hasher.input(signature.base.message_hash.data());
        hasher.input(base_hash);
        hasher.input(bulletproof_hash.result());
    }

    hasher.result().to_vec()
}

pub fn verify_multiple(signatures: &[RingCTSignature]) -> Result<(), failure::Error> {
    for signature in signatures {
        // Currently we've only got Bulletproof outputs
        if let RingCTType::Null = signature.base.signature_type {
            return Err(format_err!("Invalid signature type"));
        }

        if signature.base.output_commitments.len() != signature.bulletproofs[0].V.len() {
            return Err(format_err!("Inconsistent lengths of output commitments and bulletproof commitments"));
        }
        if signature.input_commitments.len() != signature.mlsag.len() {
            return Err(format_err!("Inconsistent lengths of input commitments and MLSAGs"));
        }

        // pre_mlsag_hash
        let message = get_pre_mlsag_hash(signature);

        signature.mlsag.iter()
            .zip(signature.base.mix_ring.iter())
            .zip(signature.input_commitments.iter())
            .filter_map(|((mlsag, mix_ring), input_commitment)| {                
                Some((mlsag, mix_ring, input_commitment.decompress()?))
            }).map(|(mlsag, mix_ring, input_commitment)| {
                // For each value at corresponding indices,

                let mlsag_matrix = mix_ring.iter()
                    .filter_map(move |(destination, commitment)| {
                        Some(vec![*destination, (commitment.decompress()? - input_commitment).compress()])
                    }).collect::<Vec<_>>();

                mlsag::verify(&message, &mlsag_matrix, mlsag, 1)
            }).try_for_each(|res| res)?;

        // IDEA: Aggregate all the commitments across signatures
        //       into one sum and check?

        // Add all the input commitments
        let sum_ins = Point::sum(
            signature.input_commitments.iter()
                .filter_map(|x| {
                    x.decompress()
                })
        );

        // Add the output commitments
        let sum_outs = Point::sum(
            signature.base.output_commitments.iter()
            .filter_map(|(_, commit)| {
                commit.decompress()
            }).chain(std::iter::once(
                // And the transaction fee
                SecretKey::from(signature.base.fee) * *MASK_BASEPOINT
            ))
        );
        // Check if they're equal
        if sum_ins != sum_outs {
            return Err(format_err!("Sum of inputs does not equal sum of outputs"));
        }
    }

    // Check the range proofs
    bulletproof::verify_multiple(&signatures.iter()
        .flat_map(|x| x.bulletproofs.iter())
        .collect::<Vec<_>>()
    )
}

#[cfg(test)]
mod test {
    #[cfg(test)]
    use std::convert::TryFrom;

    use crypto::{
        KeyImage,
        ScalarExt
    };

    use super::*;
    use crate::ringct;

    #[test]
    fn it_verifies_ringct_signatures_correctly() {
        // The following is from mainnet transaction <cf8e4ffccd7f3604b4ec4be689a7d3669a8ea8bfa5e40d7bacf44a864ee75365>
        // https://explorer.unprll.cash/tx/cf8e4ffccd7f3604b4ec4be689a7d3669a8ea8bfa5e40d7bacf44a864ee75365
        //
        // Manually expanded from the daemon via expand_transactions_2()
        let sig = RingCTSignature {
            base: RingCTBase {
                ecdh_exchange: [
                    (
                        "8b70d371ece03467089fb77f9f3808c13bb458ebdde64d579fdb95a8ac7abf07",
                        "49296989d79031e6"
                    ),
                    (
                        "5187251a60bd0849944cbe4db3cc64a09bab9f873fef052dfe5d1d40ea92210c",
                        "eed70dfdf7545d9d"
                    )
                ].iter().map(|(mask, amount)| {
                    ECDHTuple {
                        mask: SecretKey::from_slice(&hex::decode(mask).unwrap()),
                        amount: hex::decode(amount).unwrap()
                    }
                }).collect(),
                fee: 196510000,
                message_hash: Hash256::try_from("22fe728e1bb1e0fc69e7ca9aba33ea97b9d25cda66fa11517921fa638035f024").unwrap(),
                mix_ring: [
                    [
                        ["bd910048aefadc6340d12e609f9af97e8efbc733b3a2c15a9e1ebbed065deafa", "632c2832313c97266b03a60f68562953e937cef0ee389c8a663ac6970f19acae"],
                        ["8a956d598bb7e2b810a9e61a2b2b2d20428067d2244e326a94ec7daadac5dce4", "cf767f131e62590406379eb5afacf0a8f28ae310a4196c8e1cbf2283d594d067"],
                        ["0f655c450692de71932b1145176668e53792a3a2bb903f14296ed65688c25c8f", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["fb3bb02bb5c6a66ae7eff41ca636e7e7d496f2b9ffeeddfdcb4e7626c740009f", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["85a222a60cc4315a97a77f333e1b50959d99d05c7d26235b14a21d9a5753bdec", "ad289c407d9ee5e7ab2ee93d7632e5febaa83c4a7a237ca2aab5c123b5670cb1"],
                        ["dd2a83280186005a3199a9517700714a9927412eb514c9f274f59369f3c68be5", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["8e6ff568bb1105d4298b155d43372ace99bf02b4bb68c8e575c27bf374fd8cb7", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["a6f62d31370f34cfef468e8ff51f830e286a36a9e66b6fd526d7f8e106ff7667", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["d4f89ed548a901e1a3c0c3be91eea51bad5068a97db0f4ce8eed0893efe4a863", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["d823126ef838f11a5c364ba1f5de2c4ff18bd0504839c12cb86c9947b3546f1a", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["036c6f6a6df4a1573a5a1f632b33dfe820eba6d53994fc22a7bda0d085e8bb91", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["ea013bfcb43fce7b0c50cc62f7f6e61bb507a5076d0ad4635386402fafc57a73", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["8536f9bcc33ccb04b6f3169197b8180b2f8c0137ad71e7e2dd982ff6f8ebed10", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"]
                    ],
                    [
                        ["256da66fc3cbcfe74c73cebf621b9e054a7738f0f90bcfd7641fead5833b73aa", "470b2cafdbf235b517255189f7d09c9622d2e2f336b68be93d931e44fa78b661"],
                        ["24d40b8fc3d305a3625ef85813cdc236c9ac088110393e8cec4a5709f83c445a", "470b2cafdbf235b517255189f7d09c9622d2e2f336b68be93d931e44fa78b661"],
                        ["3c3981a56465a4995c34080f6f6836edfe2c047333f5d7f0be012d3e8eafa34a", "470b2cafdbf235b517255189f7d09c9622d2e2f336b68be93d931e44fa78b661"],
                        ["f518a879e3023c31a51791263ef1d22c01aa799e7ef27a558a12e96ed9c6b305", "470b2cafdbf235b517255189f7d09c9622d2e2f336b68be93d931e44fa78b661"],
                        ["6320260eb93c721feaf574dce8bb8cab2e8006fe5e0148c02cdc19bdca4bc4d6", "470b2cafdbf235b517255189f7d09c9622d2e2f336b68be93d931e44fa78b661"],
                        ["aafbb25432dbfb34ee509df84220489a52cad09db1c6537eec2978e1b64adcc2", "a6877365b46431f67dadccbe5eefba5ca8f48442be3c99c4ebd35c67063b4c78"],
                        ["980f772c172f36329f8ec9380bbca98de5d8afd89a15f28b33aa70385cb35b1e", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["d58e4c58abd19a04988057f244c257273513ab96d912fac5da97f7a86aa0f434", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["e79137953c41231a616683bcec93c3252c5ae0753359d58fc16523e50f54c89f", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["32f4ce51f105c06ad06c800d1f47bdead7f00d0f2f55b1058538ace68a8eff4d", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["10a35ebea881e828f74c1b96f6073dce8aacd3c0eb4d855ea1e2398f8f4455f8", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["a82a1009d52d69b6c8af08d1f47a20d355931d77ef7689541687a907263e5d6e", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["217b1ecf353b4bcdf1a3d2bae11b13c0fce5c92b1f3a02b9b6fb69ae8e7584f0", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"]
                    ],
                    [
                        ["28049579fa4ead6a07400091433f1cd4a65c1c4375ec56d396c93ef58fa3e79f", "be8a922ed7db95c9e21d257fafaac96ac1d611b20a745f4f2372c446fa29a67f"],
                        ["7dafa8ce69e5714f844202a257209fa53fe684694afdf6f205bcbb7914f14832", "470b2cafdbf235b517255189f7d09c9622d2e2f336b68be93d931e44fa78b661"],
                        ["2abc2dfb0b1a814883a9245545c96668879ae9229d71ae406ca93d20a5b37452", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["28251a56d7e89d80839fe31106dc19f80086bdadbd251b94c3097c790042e2a6", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["cb927dec724ccc84f8d143b94664310161d64c74782a31814e4c6361da8edbe9", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["717337412ca4c113c5f0935a1d8d6fa0499360c3eb02d10f67d858f2507103f4", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["744fa6040b24c3164b1ab5cc5146cf3fe601d8b7501fcbd62d905889adee5b65", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["37c27e6dbb7fb3f6b2a151c463029d5302827985063266260355db2abb3dc5a8", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["fb2d1e46554ae510f7ecf6ec5b08bb9e156a69effbfa9ca071d1047103616d2c", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["9365a9b15d428382680e010ba92d7e63ed26c4c225420a823418a6249c370f3f", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["b60be86a6acfa7bb04f06157f01601e2d5d4791b742de32201c797998961fe4f", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["7bb700c07d5e83557ba4537489d419e28d3a8877b6ccea83e19dd3432b8be54f", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["1b67aaf08896faa2100aaac66c0fcf7ce3e9bc550c95fff79f2a5fc0cb8abd71", "a341979d197fcd0d8e5e054b4aee67e41d7d469db80ba0d726675eafb2e3728c"]
                    ],
                    [
                        ["518914d45b0b6a3b79b13f303e9e38fb81ff1721db6a1c6afa9ee18b1886d699", "14aaa214b40e5b28f8999e94a4ec66b302c206fde24ac0d0eee089db75b3868b"],
                        ["3ee28c015f30f78c5dd5536ef65915620e43346b3f199b5d8ccef7b261ff57f3", "470b2cafdbf235b517255189f7d09c9622d2e2f336b68be93d931e44fa78b661"],
                        ["5d0782caab31e9e04a5e89fd082f6656759880c25437c6ee97ac9be5671362ca", "470b2cafdbf235b517255189f7d09c9622d2e2f336b68be93d931e44fa78b661"],
                        ["583efbef734a18932b6efded05bb6d6c28f19eda0c1dbfa1ae0c81de91b21aa1", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["3fa1eed1a73a4f66097b7606dd90ead586d2b8c65f1acd7586889615fd4edd67", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["34c1047f2a5b61c091a61bc39646c9535d43fd4eaf8bcc410a37e6534ffb8a9e", "c4af8bf185f8c968f1e233a22587c96c1dbb05f4b8018326195f06158fd33ba3"],
                        ["6bcf70501dca827e3bc0c4cc55d0f029377835d28c725d5f3ca88490fa6eb951", "37148f07974f28de7665f00526f0a81842605ffe07bb2bb850c72e9f76748244"],
                        ["20a7b5697d0006be4135000b3f94d5c195b65f905647e18b0de4a0f57ce8d83c", "7abc117a311e332cb3b6e2069e0c635cf43e3dd33e7d05d82fc727ab4d10a5eb"],
                        ["287810700b200fa756fb947a3a7cd71ac47cc9feb4ec546bb0ba239f113b2b11", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["5b0271265717452f77f8b77f95a193f340efba0af636fb06be312c33f986faa1", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["cbc7fb0312c37e6eecc921bdba42dbd03af539fb3686b0792751e68cb58421d2", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["c7134ce5d35e37b00d0d5c8f0efff1d8c39789d38553160fafd4df42adf1b6fe", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["7cbda0e9eee404c4f65fa9cd21a87bab090106f28155054525f28c2a5bf4d755", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"]
                    ],
                    [
                        ["7a5a40f556c870cb09b23009d237fb906e331c684d52f2f1dd0479b9dbfa925d", "470b2cafdbf235b517255189f7d09c9622d2e2f336b68be93d931e44fa78b661"],
                        ["3531a2072c0df32927562ffc7bd1bc80f0fed985e2b5a151b4926377b7d49417", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["0fa0a8a5471b44e50cbe0d6fd0c82c65b15875654da41efc870907b01310ece9", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["0d2388b148143b2c3e1a6d15069e9a5da813defd9286093dd8421037dc4eceb8", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["cdc9ea98e4cf096e1d9acde997cbbbcc3cc1e5fab3573fff3984de3dbc1907e2", "8f41661bc0cb4aa96ce1c34050d709413f9dcc5e1d7a11044c8e92398b13bd8a"],
                        ["05134ad8be18b2bb173d72733b27f7f5116734fd70c688c4d070df0ff0f5db68", "07fddb3000eda7db425527713fa4a4131afedfe830df6f3c341aabd0c15eda54"],
                        ["b2aed98092fbe3adccc2e1062aec705f0587dd52de141d273cbb84aa1f53886d", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["e22515d2d4d51fa674186bc59a1549587903fa7d37ce9dcb8d614f8db2097315", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["9e90e06f9d4b7048702625cd690f27d4ea59aba6fffccf448026ceb89710f598", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["34e65a6186abe7bd4160e41c3cd789e74cd92eaa74d2a97b02ed28dc88d16bfa", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["5788d5e8ee0af493f008569755968d9bea31c70a8836a4fbdc4e523652f93a6f", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["24813c14d6ce316b5cc9831d672ba37b06b8c833b64313a822c34d99ecab5e23", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["701cc5f4a23b89bcffaf98b7c322dee834ec18e3718fc2f0bdd4296c97eb74a2", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"]
                    ],
                    [
                        ["f332a082d7f8c46a3c83f382adb7ed00d17d3d37effca766a81c291f3a1696e8", "a1730c6f881535548992257ffb94ba7955bcb90c91a2da76759cbef72dd1310a"],
                        ["6331fcdf5dba3b61d209252545148576eb690aa7dcabcf525df951530ecb9611", "9b90c02d16f3568861be1a0118de61b30c824c6b76bd0938ad8a9d38c211e56b"],
                        ["249a39240b9fb9fd4b33e1f837fd081b2ee201e1f012be75d299635e1008f492", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["a7e3cff6557629e241f935b9ddc7ce4017a9ffebcd62671e5a8dc84a48a110e2", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["f1af575b2e7f30a59949fea2f9bc9f16a2cef807ca0eeeed20c68e3321e70086", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["98005c03f95a1cc55760f76f0311bb087ade0132ac299de806a8be1ca6fc2e81", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["a838d06fe50c88f8bc0daa26e97ad6fff128c424e56b4bc6b79636ad4ce1f3c0", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["f5ffcede7efffa41ddd9d394c50e377fa93ca2148a179a969bd9e39b59397c8f", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["f3574d0eb133a4778f9327240ca8802a9fcb3bd089bb8c0ff4744ce1a645a057", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["307c8147cffe92adf817ae36b5fcd4d2f187d894348159722d5477a35aac3146", "1fabf71564d08447469021298fb3fb847ae6d5f92c35ceb9edb1f0dc84c612c6"],
                        ["4be4e2747050c98c8e56529c39ac568dc7eecb802700cebc77dbd6feef4a0d02", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["037ac8664d1047fef2a6570957a265a6b960c6180166bd4e9994b54c340dae29", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["9135c51f02bfb5231fde65b73f0e9a130a4018055f8fcd63b75ca4bc52caec53", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"]
                    ],
                    [
                        ["a2a110f7b1bd2d0106a9e9901e9ee123ccc2a3e7325e6b695c819c4f01aec375", "7dca67252c866f362f382d6a8283f87a35e1300cb36fb394c4adc53272e5dfcc"],
                        ["1a61e38ab0a40b1fadfcd6adc18e523ed2c7bea0f21c5bd38d3cb6a5e3bb5df5", "46765d922ff869acfcf17869a6e2db359de506ac7a5e5770331aa4d8206459da"],
                        ["6a73c611530e76cfba7ba2aaaa026b84ae3343a279052721d096cdcd8a94175a", "19dee6b699515b3c5b4ea23a88a04111066b7ca1885867ab7a77ea9b7aaf3aa2"],
                        ["35df44c9705301b72c421c692ad8936043830028ba87281bf858d237f89865f0", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["9de94d7d99eefb5cee7d0c38306650f835f34147e35d21a5655eda6f7d85e2aa", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["ec03f5ff8489420334476a06b7789c9a77beb801aa3e1b331f11afc9416f8415", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["a7597256196538571f4b0c27f0d90a7324f0d2b88468a1e479396366e68f5059", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["546574e7c6ec455416d7edb68f5739916710e5607e435a7f33b30a8e26810768", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["ca4504d05f3b7939a5dfeb5003c1cba3b1526dddebb654c03d7da71badfaaff1", "f47046f05f0398b5bf801a4dd13cbe25428126774da7e4e841c598a8a74858f7"],
                        ["ddd3934b598856606f605f25516c8f20564b51a5c39b9dfc0883401c91a8df58", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["0e9149891de90c5ea996feaf811c31067a7f1c5f5affce3fdca8f13f3ca36814", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["01e60ccb6b4560d3f8c07ea9abda36118c3b43e8d8a5c0a0dedcc305eb452a48", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["84762915a3fc686ea4bfcea9b436dcbb39656c43bc5307c3d3ba0516b35093a7", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"]
                    ],
                    [
                        ["85631286fb025715051bd4a519a828fc4842055fe38a0dd2fdde913b5b709f4e", "8dff4d007f9b0ea362290172c28fa1f66558124a808f189a37adaff5aff467d8"],
                        ["8c923a45890ba82371267da3fd2164b5e991053a497fd8ee0b68c2ce7ed4c4e5", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["1ce459c59d038e1735a91e4b820b716914cf0e8e253396794cd5fbd060cc7d29", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["bd0227a4fb9395dd7f1dd72ac16ac04f60edac66f3e49c1f29ae86ef35f8aaff", "bae29fc22534d1be13296f27fc748c4c5b5156380b342a64d6f492b897248dd9"],
                        ["47ae2057c835154989594980e883eaa661ccb2a864ecd14dadd9aa8f45af1082", "02ed4029ea054f8676569c9f9db1773fb047f964aed54ed8579c5a7781c77dbd"],
                        ["5e7def15f08d0a9b31d74495d201531959e0dcdafe067dc45a387deada32674a", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["6c1194bebbb68d1f16a07317d93e22fb1ab915d36b4ac7d7c7ed6a0d2a4f20e1", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["1f2a7c0b95a87295d6055cc184e2c6412fa2fe22568f284a0c6a0b50802a6fe9", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["f30e3c7f41928d2f7a7aef97d728c322c1897d36617834617dc96b006091c57c", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["48f98c9e141a5a559d864445b102d0e9a5c5dd38cc016cef37f731a51a39392c", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["25186b6e530a6719476abf72334568671f08c3e9eb229b2b8ab8bdc1def8f438", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["b2c3c8213bb8b4377ab91837e61cc9aa2ec01e79dafb9933678a9278cb47a62e", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"],
                        ["322d1a9b5657f36cc4153d1ca634bae717ecac9f070c0ac5b271526b38c8a63e", "899b8f9bb9d27a451ed53878316660066c6debc4188b530332a7c38733009da9"]
                    ]
                ].iter().map(|r| {
                    r.iter().map(|[destination, commitment]| {
                        (PublicKey::from_slice(&hex::decode(destination).unwrap()), PublicKey::from_slice(&hex::decode(commitment).unwrap()))
                    }).collect()
                }).collect(),
                output_commitments: [
                    "5324fa962edab083eef717f8dd9f2cced683671cf5f28081c83ee1171c054869",
                    "62c50265df62e8b6c78a1e320366684ab5873565ce0e17eaa4e1a28bab9d70f7"
                ].iter().map(|x| {
                    (PublicKey::from_slice(Hash256::null_hash().data()), PublicKey::from_slice(&hex::decode(x).unwrap()))
                }).collect(),
                signature_type: RingCTType::Bulletproof2
            },
            bulletproofs: vec![
                Bulletproof {
                    V: [
                        "5324fa962edab083eef717f8dd9f2cced683671cf5f28081c83ee1171c054869",
                        "62c50265df62e8b6c78a1e320366684ab5873565ce0e17eaa4e1a28bab9d70f7"
                    ].iter()
                        .map(|x| hex::decode(x).unwrap())
                        .map(|x| PublicKey::from_slice(&x).decompress().unwrap())
                        // NOTE: Remember to multiply by eight inverse in actual code
                        .map(|x| x * SecretKey::from(8u64).invert())
                        .collect(),
                    A: PublicKey::from_slice(&hex::decode("85df863be3a385365b82cfbef09aaa87267522265e9dc7d8f5cf32440bcf3996").unwrap()).decompress().unwrap(),
                    S: PublicKey::from_slice(&hex::decode("51d1d9f2ba89de8cb5608c98c795cb6079a0b4aafb60ce5c444159d8edb8db6c").unwrap()).decompress().unwrap(),
                    T_1: PublicKey::from_slice(&hex::decode("d6939befc6a1d735fa4a13e0c4f69bc1e72bdacab6f60c260fa763c6f412f474").unwrap()).decompress().unwrap(),
                    T_2: PublicKey::from_slice(&hex::decode("21331553a5d2a385aeeec00d7f252b86bd6a676f63e21a16d4173f0f0ac795e3").unwrap()).decompress().unwrap(),
                    tau_x: SecretKey::from_slice(&hex::decode("057d34ae685f3b753eba9be6bb3fb88fe2335aed10bbf027beac6d071593f600").unwrap()),
                    mu: SecretKey::from_slice(&hex::decode("b5b36890fe4006fedf8d5d8d5b7a33b71b60411d229c96d8fdec8c8db20b2902").unwrap()),
                    L: [
                        "6a5d60a0ece269606913ad09434be74852ef65c8248c111921cd5ca25eed2324",
                        "148f1631c946ac671b66ff79ab02cdf44b13259d8173c4039fbf1f6d04342b42",
                        "75d59916898656b9c13929f2abf386263b241c42a6c9bfaf404ebbdf20ee0e4d",
                        "193a55a65d50a19f4f367873bccfb869bc32b0aca8982d5654dd8a10b14805d2",
                        "50bf471652814b69b4f690f510fb0bd4cc0d1181ddd86805c82d8b6fd6b3b391",
                        "20d6ebc217ac4ee405cdb23fe48f87ece14d1cb19845af38a054a8d2ae6aec95",
                        "84b7729f5fe410c5dc00dcb1fbe1218f118e5d92eec81943b5546cac74653043"
                    ].iter()
                        .map(|x| hex::decode(x).unwrap())
                        .map(|x| PublicKey::from_slice(&x).decompress().unwrap())
                        .collect(),
                    R: [
                        "dfc25c10bd846911a1ce4bbe5a7bb877757937a30781605117aebb4890f22936",
                        "f2e77c4c16a18c46567a5f5d6e0d0765206cc435bbf59bcb66ae5e926d6844db",
                        "2f877df20b0fe87e422da9c233c39890665a326f50c6861bbf17e61b068a6b4e",
                        "980e92d495c02d86c92372efe423f0c9e29ef1e94bfa4ca6688bf17d045f0819",
                        "e14916dff97b534d7c30d911bdf2e0586ed676f8be3b200ae18b264ecd0d94da",
                        "fe8a374b1322d6bd325ec963b5e9e3a7f0151dd712c7c81d208c0c5429983dcd",
                        "48901c3dd352422a24056e2a0f72e41a8c1eeee56fa600f83a1027d068e2c8e0"
                    ].iter()
                        .map(|x| hex::decode(x).unwrap())
                        .map(|x| PublicKey::from_slice(&x).decompress().unwrap())
                        .collect(),
                    a: SecretKey::from_slice(&hex::decode("6d97e02c1942f18a900854d337d428b92416af2680335f8fc7fd003320a19700").unwrap()),
                    b: SecretKey::from_slice(&hex::decode("d0dea26ace229cace97f2f477f4cf770871d784720cac53ecb47ead8021a5b09").unwrap()),
                    t: SecretKey::from_slice(&hex::decode("bfa2af387659ddb7fb4418fb8094a99f394012c5fe300c7cf8bf15cc91fd2d04").unwrap())
                }
            ],
            input_commitments: [
                "e610a35489aa345c2deb5367b7d9faaa2af752bf05784d479e1cb5d8c3b32245",
                "7f408048011dd1709b000caff58d6bec33d532cb7fa936387a70c39ff2ec1ff0",
                "859f74bac0f35ac8c91132482ca4b3c6a07acb10f4e4bffe986ac90da6a2ba31",
                "d4bbb2845ffdf790eac489639fff72c900a02c96815ee6980e63638c06849100",
                "98bbd075a01121dc6228e1e30755ac668f288bb785d7056f21da53b37550ab51",
                "27f65172de0601c976ab6bf3fd4afab69e996e585e87bfd52c88330f1c9c6198",
                "2bfdd11d15dce87e1dc27c2ff06a20b4699b0228e42f1f4ddaad0ca856a199a2",
                "c7edfee963658fb8ac67f1609d158731888bffec5a855f8803d9937bf4d02b7e"
            ].iter().map(|x| {
                PublicKey::from_slice(&hex::decode(x).unwrap())
            }).collect(),
            mlsag: vec![
                MLSAGSignature {
                    s: [
                        [ "a82b8c9e1f4f87c4f6dcbbb68cf2433a1460144d1a2fca6c1f7223532808010f", "f770fb8fa66fe8723df7aec0228bf07fbbf7d2d7fee32bdebec885a27f342806" ],
                        [ "0d833346786fa642a5fc19c3d5a1cab1cc4ec9a6ed7365205901fb15d7714f08", "01d1798f76fd5ccf81e5f9eae16d86254e5e29e713d1dbe90094eec42ff1bf00" ],
                        [ "dfc9b5f13cfbcd44b079c9b7e56e0626f1e6dfc51b2927743d5bc7794e22d30e", "43d9b093023a3ed939b6fd065d202c8a5bb01b7c1f0de09d1b435717e8c03205" ],
                        [ "7189af23d93df82e12438c1e056a73cf77835be86f555b15dab63f776347cf07", "7f9615ad7a907198014b09d8a7dc50646a1caa1651bbce73100e920c6bf6e506" ],
                        [ "d4e7e32460a9a795ad9dfc4b85cfaf53897f0a707252f688f040e756c65b3508", "baa56bcf9c1aa44ec2cd6d1af94b3928a2ee38321685cf36f28218ad6f04040b" ],
                        [ "dc4ba4b256247d65d1f26776b3b527df9055c3175c0fcdf4c75e619e10567e08", "c93d9a669fc5ea0fd303e6adbb1062b916cbc268ac09ceb584be375b22f6140c" ],
                        [ "bec275aeec7827a7ad912fd2b8356238fceeeb4927909419b12a23db0359dd02", "0296a057c1a233a95c76f0dd6b3331f6b30d7e019201135b30e7341db516c706" ],
                        [ "b876c72dfbe374b4286236e7db73d7295bc4618e9d56c42e37f8f23035103d05", "10af6dc897d9b4ed1b35e7148f5b1c7ade2fe338c11da274f6b34966e36dac09" ],
                        [ "a031d0177dc6c4499fe79d58269f55995f77b22d3d12d2755f3d129d8066380a", "205ec26a6776d3fc61f583a647b23f4cbf5e007c007e0bf536dd328e6c088505" ],
                        [ "ec10c9e699760cd7e90a418c9a78c60ccc963db9664e7307863398ed231b3e03", "5e1bd1847374874745fc20844c59cc9542483e3a4d86c96bbec7f99cba960204" ],
                        [ "3a73aad03d21266d5c35470c177261c4ab8cd429b1e55e1e94d62b5649964d0b", "b1b96066a43e01dab3d870c8f90dc668e1423d8f6f3fa527163007003d23bd07" ],
                        [ "52ad2218db2c2196d29fd031324003fff36e697936b2e7a0c8b0b4568a629a04", "39922996abfe9fc0eb39deb57cc34bd98b3b0bb1f08c091dff07ceb7f80ae000" ],
                        [ "c07f51a1357cf7ca8677d87d2e3829fb3972caa7dcc733a7ba29959df0fbb30a", "ad64bb1c2dc933d78972cf8195547abfab3219cd07ba1a37f8680c59813aa403" ]
                    ].iter().map(|r| {
                        r.iter().map(|c| {
                            SecretKey::from_slice(&hex::decode(&c).unwrap())
                        }).collect()
                    }).collect(),
                    c: SecretKey::from_slice(&hex::decode("345fa251944a661131a79c979deb84f07ce9ba6186fbd3dbb661397416179102").unwrap()),
                    key_images: vec![KeyImage::from_slice(&hex::decode("c31459c2a5f57af8e0d576b5006f00d97e289af42cf20a5fdbffec32186d7c31").unwrap())]
                },
                MLSAGSignature {
                    s: [
                        [ "315d6abb7ba78d35d19876e4c40674c5b753988a2f7a992f2fd44bcaebaae304", "d121142ce50fe1412ab5bf76b732daf2223c4efe3bdab37b59f37c40d3adb805"],
                        [ "f28b4c54620c0376e62c42073bd7c60dbe5259b49c1b2474b8ad0d2bf7ef3201", "dba9ddc6ee01b7f35b1c115a93ac76a54188f538f2cfa30ed28beba733e0080f"],
                        [ "a536f28d7e211e4aaa842e335059ea977c470efbb2492b21415cd7a321969b09", "2deba068c3b73cabc607e989a97727560f17d6c9f8b209e4804f386f45083903"],
                        [ "1e3e138adc1463339bad80701d0b72367e569f0afa784ac511c8ea2c9ef3db00", "160c129be97965670d821a2c65a6f1ef27459cbc0372c40a50afea10d0c13d03"],
                        [ "b6db10b587068f25cc963b6ef09ca05f7edf8de9b770e13ebb196408743f940b", "b756a5988192d3370d8cb9661570afea7711e4c72f5a32fd654e5af4311dae06"],
                        [ "5fd6ab5c9f9dd6235424f743e562969ba70b11f3a3f7c92c26c756b8e8bb8600", "dcfd628025ffb50ee721037e91f5eabf4dad52458d35d6d965e3d485b909500e"],
                        [ "1a8e8feff5da4f97a0308d1024a9b713449aeddfa1cb303469ee6126ffc01b08", "b3e77cc40a352a87f96c7e620e388b9c1ad5a2dbdf2296459b4646c1c0d9120f"],
                        [ "198ec14b4e1f46d1730557a42dd8c227a253b58de1c7ae37826e697ba71ff10e", "0b44c814457522e36b36004be300bcf9a037398781f64a2c83b35d15811d1801"],
                        [ "d8a13908b578c0c7677acc1bd4d559758052290cc7e7df09171c0aa66170890f", "906de07d04361248fdd1b82ba874ffb64828f65b26158b24dae5fa71235d7507"],
                        [ "d05cefcc3b26e8c2580fb30c662ccd6f9bb8b06e3ff22b2627106b419c7fbd01", "1865578797f094a9eb75e9c05cd33e3b1d0f5d7fd11f54d08db7bb914b11ed03"],
                        [ "6a8eba8bbe7c1250441ddf5a6a25c620339482b424f735f8798736becacfe403", "93e648899ddfba440b792e41096905e78de3ee66eae5a6a74e831c1ee205ce0a"],
                        [ "f5ece5a35a7bff2ff0050fca4762b4d71d9536aaa42399c1c831a97342495f03", "ac862259e8b5e78658e55d8b16bde92da2361521b1d557ff57cbf09f28683e0d"],
                        [ "45a0ebc752592967d55656c44323dd63bc135bb91903925a25a2baff24951f05", "8148f003c17969100093790bdd16a2a0b41efe782e9eadfe274c29f9dfb1920b"]
                    ].iter().map(|r| {
                        r.iter().map(|c| {
                            SecretKey::from_slice(&hex::decode(&c).unwrap())
                        }).collect()
                    }).collect(),
                    c: SecretKey::from_slice(&hex::decode("9f77970fa63ec1ce9a2724d597433f75eadfaec8e482436d88e8e0223dfb1406").unwrap()),
                    key_images: vec![KeyImage::from_slice(&hex::decode("ac65fc7660f378fba77cee0fc5eb218df71a1820db1b1617092f345ea35392db").unwrap())]
                },
                MLSAGSignature {
                    s: [
                        [ "3acabca04a99a2685662ebc9f9302a1815949614202bc052c1b5aa59530f1f0c", "f02a6e42066287002de9af80d077c265c99706d1f61bdb3902d94a4442641a03"],
                        [ "2a0c12d3cd857b2398bfe530df2583c35aa24e7dc58f1404bb84d95bcfe48c0c", "008aa9804573fea9e6a9ef341943d1a3aa9bb402ea51a818d34e86e2c6101403"],
                        [ "2abc286355e8ed1ae9dc9280741d8ba7ecc699aba1ea1239fc2f6c327b26ab02", "1bda8de74bf61162bd9f63d02ce89b94b7fc1f40b17ffbb2e5a3f920e5d71301"],
                        [ "829c663f6822d18cf96c32d4e4512d96ba1122ef9cb01e760450e4f8b85f1c05", "f67a20a0ec32c03f750694cd559aa20a12f9d5eebdc23b00a95b571c1b289a0b"],
                        [ "5e1fdf83b9de23fea80b3e8fb1c69728bc5edbe2a5f63b12dfa365c204e92604", "463675b3a5caac669c6f8ed7949f79c28e0af45f2d5dd75342e503e4884b9a03"],
                        [ "208ca3b9ad86f7acb34da5db509376ece7034ee8de8c84a7fdcc215d84bb0f02", "1dbac4f22f8b20a880186ba6bc96f738fd7629b72b9e992cb265804ae6fb3804"],
                        [ "04072e9f9935938ec4727970f99827cd00f4f05cc6ca75fba468f66b86edfc02", "c767ec0f378f9b88de5bcd7a0a0935f3a792b484f54f906b096c9a2b28e86302"],
                        [ "f15280e6055939e4caa420fe1c416e112787078bb79a4d339e2fb3b52dba3301", "51b8eb59955a0e898209e7085b90062cad5d70e4625040c7775193d8b16d530a"],
                        [ "6ce06f9f71536a00e65fd8defec241fe0ff81095f20cf71534255374f1789607", "a6665e958b49c52acbe677c63447222278d0094ae36d94598dc6274352d4cb04"],
                        [ "8b65f7680f748f5cb3dd1dd11ddc005dff1462095a4191422a38c8ed928d8b0a", "ff63a993f4a1cb11d1373407e61b91bc15135b6d7aeb164560d4cc8d0defca0a"],
                        [ "e419951265ef46c7e9ef4975ca5036897c985c8b2c36f757e0fdaaa6f29cba09", "cdbb6f8e2ce49d5eda46719152b825dc4f186ac1841f75bd3c839755c3aa600f"],
                        [ "7958cc38efc895664fc6c4c6966a1f7923df2df61c41080cd260ec15048b160f", "9e397fc6256d6827a65eca2cf504bef16e870c5035a21a9cfac56aec137ef80c"],
                        [ "1778e19417e89dbfc17a9167bad6f96e8339beb639785927920ba2af3457c800", "0c96a3a1e957f709a9de4684cbcf8697cd663fdd92e9dce904e39c0691d88e09"]
                    ].iter().map(|r| {
                        r.iter().map(|c| {
                            SecretKey::from_slice(&hex::decode(&c).unwrap())
                        }).collect()
                    }).collect(),
                    c: SecretKey::from_slice(&hex::decode("0c553a1d688e8ac929b5da160f88cff4027898d76ae2b2f37fd8ba3ce99bb70f").unwrap()),
                    key_images: vec![KeyImage::from_slice(&hex::decode("a16627b3882df6f26bf4d3b9ee483bfe64c9d6e5632e95c2b2d8417c454b4e92").unwrap())]
                },
                MLSAGSignature {
                    s: [
                        [ "5ece2e371f66d3af1feaf4706a539cd0608d151a5619b0d711b1eb8676dd270a", "2e983a1a055d0db230005ec9292186b8b3f117cfa523b63e80b306d326c6bc04"],
                        [ "0881840161ed53b91d1346cf007ee7ecde6eb64acbe58a8b5404f032026e9b0b", "0a8477f793f4ba65ba129bc941bd1fe6fc276a57876700f3fa82e09c29aa0407"],
                        [ "2b58cc90d7e341bf931fa1677e616bdfa7729bc3843d80cef03566fb209bec06", "bc6f8f44ce8abc2fd295eb291ee5a1b4ed655c51ba38039b2a5f5b0140420e0b"],
                        [ "0eab1c5e9d747c640e8f434d3f075b6174d373212cac17ce22bfdc85c09a2805", "af46bcb26bca3b86a7c88469336b84738dce8a7caf71c3a8a7b3a16ab7fa6a03"],
                        [ "dfe703e1c7689c891c6e9699e3fc6ecf2425636fd96ee6fa6a261746507c0607", "4f863dd0c3912632cce142d09de1be1758b5e96910bd112fd8bbed2ce428ba08"],
                        [ "8adfc12b274873ea6c48070cacf182b9347a861432976b6e4e65a1adfac0f000", "7ee656cd155081901d50aab89e3a6f71d3af3871535c751e61056d352a9b3f0f"],
                        [ "0582317607cc9c490b3a5438e7a1817915b3060b229e80be4e479d116594a303", "960e6f7e0acdc12b08624476b1d67dd4cb1ea3a212ad8a2769b987ea978cca0f"],
                        [ "9066084e58b69ad5bef05f8b9bd96433477ab8d1d13a584638a512d447727a08", "29db773d07556a1d3f190b02bbd9168f8a43dd1e5019692cacdf138e1e8be40a"],
                        [ "5ebc0c535ed34838b205e4ce7842da9760b0778b5fafa74d45fc93454890890f", "02a1a5dbb5a927218933a0afa4346801c036fcdf12bf3b93b84ddf9e08d2870e"],
                        [ "4b4a86408180a3e649fee41e56c071064f566e42113c281155a35a51944c2103", "573f220e8b8e8efe66ec83c0e6f1d6d5f8249ac5955c89f4503a9b77f465c508"],
                        [ "c9e4729b89b0c7650a264e03a2bd9b0ac9877ed59e1cb1cf3d360351d944970a", "db2532d644476d8f9b9913042772273f40f95190af854114a990a04b9004b40f"],
                        [ "3fa854b1eedd777f7a2643f0b1cb2b756616a8095c94a7765fd2bf4912d0ff0b", "95d9936367ff387805e8fdf278eb6a15568eecaedb81f1fadd1af40d01968a08"],
                        [ "cc3727255438471ab4f87ef844dc806b2a914c80d9630ecd80e989f71d293e05", "24895b1032fba1748b91e7ded2cafe4c65b4a935f06d70bbc693b15178ff1a0f"]
                    ].iter().map(|r| {
                        r.iter().map(|c| {
                            SecretKey::from_slice(&hex::decode(&c).unwrap())
                        }).collect()
                    }).collect(),
                    c: SecretKey::from_slice(&hex::decode("06fee335e6b5a5c2d62ee5bab4b17970df52e6a20ab4b74784963178c3af2604").unwrap()),
                    key_images: vec![KeyImage::from_slice(&hex::decode("7c33c004d26fcd67000adf36b3622a73bd9ca18b4e8b45b8e6d44a6d037f78fd").unwrap())]
                },
                MLSAGSignature {
                    s: [
                        [ "28490cc708671cd01174339005ca421e5db9724aaad323cb6d0f95429d82060d", "6ef3fc5d7d1ef7d7f49931fc34753428b22fe320b00c9d015388b4b00fb25b03"],
                        [ "b947f0f08a0225958bff746311a549bcfdbf8272ad9b975aae15476c282a1308", "e30f23dfdd7ed13380adc3eb683c42c68416737374bc8e24e91158ec4562690f"],
                        [ "943392597d2e25d6c456d12ff892f29a36b94d8b91f1b89657a213e8dacf8b02", "d5ed18cd44d82838e0976f63270d64889b73166092c67a8acbbbd29c492dc102"],
                        [ "c24fb9832bdb9bc8bf19e12947760e33f4040e69f44a1eef7d65d19743aa9708", "2f7a53fa985ce94c953c029cdeedec092aa27a914ebcdfb173378a1795e0d40e"],
                        [ "be0cbc304edf74227717d68f033cef1c86f072bc19d248b71c7d2cd14de5d605", "4df93c415b0cfc46ad8dd5199c79401563d634cfd51c2ac6c3c17e4af2ee7001"],
                        [ "04b287094488e6ee3b9e9980ea031aec952ad7ad1f58aaf90246df3d36323c06", "4d2f599f59cb59fed338cf80bd316ad44e2afcfb6232ef2ca5beb199057d4f08"],
                        [ "de4ab1840ef8d2fb5e514a06bf0b81eb677b7d759aca3b7e3b7a37f1a4837c02", "bc1a3efdb1a8afaa2078f36a362c174fe6ea3a11848fe3d47ef1d81b11da2301"],
                        [ "0e3d2c56d64b7c8db6cfa3bfb8433efe00ecd8030bc04896db2c5af704d2670b", "d05ba1ceaaeb6a0fe4109d802f1da45d6ed936342407bb360d98d9ecc5d10001"],
                        [ "c8d0a7ffe8cea60900ec71d00a670a3cf6d1959f19e5bc2eaaa90fb5b3abff0b", "b51ad16f7c128cb6e6eebccfbe6e163e8d1b347c2301526897a755d202084005"],
                        [ "f0055fe455a6ee52631e4de0a88786dd049eccf61b350402a1b4a95c40afe30f", "0ecf18c97be4bae0bf64de2b466ed5ab46e697862968a1248bf73027ba0e7400"],
                        [ "1ab9402af727db7409e146269830f6fb0c2c97fceb70457c7860c5a3ef4bed07", "51b12dd0e2358b52841c27d59778f62bbaed52950a8c24f1800832fb6974300a"],
                        [ "6e005e55b90eb71dc7ac5e6068879e3bac845eadec1664702c1d0b3c25626908", "445ea6a457818d27c8adf881dc406702148dbf5208155df7d26609c11c75b109"],
                        [ "1ae33daa44bb76034b5176d5cb4bb4b0d0fe1d032039b6b9f15ea1dcf0084002", "d92b69bc8cad22c2db6b29f1056c8e23d58619626f649f695dbaed8f4ff4b109"]
                    ].iter().map(|r| {
                        r.iter().map(|c| {
                            SecretKey::from_slice(&hex::decode(&c).unwrap())
                        }).collect()
                    }).collect(),
                    c: SecretKey::from_slice(&hex::decode("320323b2248da5eb620fb63d94b2620a5384186a31009551d78cfd5c6014f90e").unwrap()),
                    key_images: vec![KeyImage::from_slice(&hex::decode("4e7472ea5f5e492c0e0f78d975127efb65b3a85baeee976e0526a9d8148a303a").unwrap())]
                },
                MLSAGSignature {
                    s: [
                        [ "61a7dec95369345115f8f4f5d197ef2304946817a364ba269951de61f4f5d904", "08678201e492218544e41f86979918882c2cc8c6e4ce05d0067cd39756d39b0e"],
                        [ "a411d26b6d69ae664ce48b9125856bc8cb2d13dd9f1355bdc4ed99a75da32006", "c934385ee902d35cfadbe148061afbf7e8faf94e36017e81a03f07f7a4b7c809"],
                        [ "f67cba3979f0c9cc9be22aa730082377b24276e4388494b3af4fd90df8a94607", "ddea967b140f785aebd5d1b0ea95cbb781628e2865659176b0d88f55553d970c"],
                        [ "1999e660d08da9c421251c535f1f7b3fe51701938a3bfb4f8c98a99c1abcf90f", "6a5268f5be7609af1c24a72159d7747a840bfff78d06830db8f50f20ec247f0c"],
                        [ "8cc9433a2c628578206e26424b20c9812d8fae6378b9403858f90abd298bac09", "4c9679907d985291f953e263e8786e9018a19746f0997e9b69d3885cff61ec05"],
                        [ "9b372afef15a9124e3fc111730902e274830beaeda1c66ef4f02a948bfbb160b", "48856d50d242c03fad2e3c4332d02aa0a78f7b2f792737f9c7520f0687bb9509"],
                        [ "daa101ed1064d1cac7c4518a308af4aac8c1216e5d8ebde7c987c9d3fae2e20a", "8f3e81fe2e2d57edee484e8cea22771e7dbfe822931604bfe9e2d19181f1ff07"],
                        [ "129222078e0d6eb42dd9e280de11681de46fb5dbab043a0a3a10dabfb117690f", "6568609b71569681781c9e36876de017ae256046f1b1e8ba6f525a596759870b"],
                        [ "c03681a67e3e1b16630dd771e50c78a3e26254a88ec52761aa61d76f9fe6570a", "180c55852a34f09ce30c8e8beb08b1630dd36d1f765ce39057bc22f728dc4003"],
                        [ "bf948dcb3ebaf936e0393d901f9062c2944875de7760d61c834c1dfb5c4cf702", "e3d8def3d81d33536e347bb36e79148e3acfb4be4a788891ad8e6c20ac43b30a"],
                        [ "88c93713e35603c32c881b171554b1ac7437afa4c4961e8b4f654c10714e4e03", "779d3d2bc12c0d25f45a689b165d66b0a37036e44b4a0131c20b6e551bd56e0c"],
                        [ "aabbdcba3678dba8ed8a14f98d5f9b7bb304f8ccab3c4de8f49b0f2902e73803", "a507b0a19d24101b350df4fcb7bffa2f8ab6ae45bd1271fd55504a7cb8910b09"],
                        [ "04423292ed6377c51f39307da4e112fdd378edd2e79b93d2a8ff454b1564930f", "2be79e2a5d18f68b14f6d128f972967053e6e7ff53401e0e500807ffd2a16c0a"]
                    ].iter().map(|r| {
                        r.iter().map(|c| {
                            SecretKey::from_slice(&hex::decode(&c).unwrap())
                        }).collect()
                    }).collect(),
                    c: SecretKey::from_slice(&hex::decode("85c4bd08c3fe7126d5b3716d1b590c6e3b3305dbcd77a18906109e8a17c3d800").unwrap()),
                    key_images: vec![KeyImage::from_slice(&hex::decode("4ca352a03037b52c8fb0bb22f0d9131b07a700a733f768f7fd4287813e8cff84").unwrap())]
                },
                MLSAGSignature {
                    s: [
                        [ "0ed00360e1348ba3da63a65615ad59f422d4b3bdd5767740da5767f2b786cb04", "5b21846cbe4e41868f666fa5bbc4ccffb6e7582c175afa6253b2f8514f647a00"],
                        [ "c6a0eb69e3aec07bd4020268b7031295f584bf6a2a5e1c9981cf343b407e730d", "538227d3b9da7a1845fc5cfa3a40bbeaa06fa868dcca371e3a02b5dd627a180f"],
                        [ "8e3f36d523d862b8ca2bc0d2512d979d319583772505b7877e15392d1744b60e", "cacd33ff212b67d6c46ec7c5579ff683100f25b199d3735b3f67e09b82654d04"],
                        [ "e1c69c1589d1e1e956218b539c2031097a886cdd37feab2329e38ac464b7a304", "1a13549fe0b34845e743a090920271a77a7e6642fd15a9e7fa4f3504e3054804"],
                        [ "fbc9874f5a228a23d1d3ab5343db3d8c111af4326212b9b96a8e8181f737d907", "98181eba9c4edf69d168296296ab59d263cf0befc13848f308288fdf4c15640f"],
                        [ "f027ae9dcfbd36b90443c7194755437107a46baea4cdddc1dcf454195c5d560e", "0d1d96ab7ff6ef7da04c142e9dfba3f59bd22ba50cc0eb98e1ebaf3a45942508"],
                        [ "514b790ba9d9375f8f0b7509f57914ec94a4d6d34eadb4f220cf9377d3eb1f0e", "d2ff9c862430c7b9d36f44b5d0282d2e43bf26dda84a9c234748de2b400cab01"],
                        [ "23cc17c881ae1dbcca631d38acc87235f1bc72a9cf067c7e0dd5741252f46201", "bffc514a57475589ddfd06c762b3c9237a4936e4df93eb7b744b9e1d391db203"],
                        [ "82c779d2a4f62d2ae83028f606498a4a4dc7ba602da2d53450dc53495ae38c09", "777a64f3fe31d4205193d717f08b599777b40501633fc72e4276e2821381ed07"],
                        [ "fab4f48961a8525bae26c4c4c710c50d95ac612389699237ec8d50654404ca0b", "73c3e170d72dbb210c010e1f8ab19776976b9aa131cc6145d6aa44c540a29306"],
                        [ "4114016e5cd144ead2fd9da981ded96899acd41fbb75f9a54b2581eda1d5060b", "9deb6d66de9a0845dcd5ec28e5a28c83e3db9103b118de2e7677f994fd84df07"],
                        [ "f839516f9f77845f769a475880495617ba77255df031b6d96ddca7cb90d8f20e", "e7680f05713e8586869b501554498da2d85ad7a1d85e95a7f518dc51c6ddaa02"],
                        [ "1158323b876361c6aaf654b4183539ccc40adb2ac57a3eaf728fbfccfa57460b", "67490a8c556b4fa7ebaf64acdd7ad122714a65c01230cd39605cca9d66407f05"]
                    ].iter().map(|r| {
                        r.iter().map(|c| {
                            SecretKey::from_slice(&hex::decode(&c).unwrap())
                        }).collect()
                    }).collect(),
                    c: SecretKey::from_slice(&hex::decode("81775d06446993e763cbe1ac37030ac6a283b2a401f4c36bd1f724f7915bd50c").unwrap()),
                    key_images: vec![KeyImage::from_slice(&hex::decode("47b7a021c02eda05eb6b2b04d8aca0e225ecd6f49b5e1ec24b67dc11dc91bcfd").unwrap())]
                },
                MLSAGSignature {
                    s: [
                        [ "1f88610297faeb227832c20bafc2dd0c1a32555c375a02702da69e065f575f0a", "ff6f15db2a41a87bafefbd33601b472f6b2c9f4b89d7f9d206b3c6a62045f900"],
                        [ "674df6303a6eb9ec48419976cea379e42283595054f5ea3e0e1b286d278feb04", "67cacff9a07b0dd0d0f0d8cf79d9bfba215b2e6b24b129120beebc77e4bd7b0a"],
                        [ "a2c3e7e8187aa33960a87da0ae00720d64b151cf1c95d139ff17d4df4aa54705", "364830c06509c736a2f61c7dbf0fbfc0d000bb67e809e3a274b8625269853c0b"],
                        [ "1b21d0c4608af39b401eaba9eb70acd78317b8fc6a5dda9a1efd89a220e1470b", "f15196491fc1a615497148f9b711f646b9a7e09859423c8244b599db0a626502"],
                        [ "87a4e5c41a6bbc255e40a0b479415d6868e7f1f755acf6afdc962db5de9ff60b", "d31e6beacae666a2d5113622da9641e7ce5b3ac1c232fe414ea5154bd54d8a0b"],
                        [ "2df5642a5962308537763b88c3c9590b676e65cad47830b450daa7810b2dc90f", "8a6265787cf5a22595fdd2394a1b2b590c5e4abdc9557733bc70a5daa3aaf303"],
                        [ "a7fd0521849b26e0939339fcb3b1f792a83944caad8fff6738ad205e5d310e0a", "f6cfa7587ab501ca802b39d7668476ef9bc5d6231bd0ae661310a2db3633a608"],
                        [ "09d4ef1b12c59142ad3931ed5d3348be243f8e446254d25f16d4b170b3073608", "654606340252cd967dc94b6ab87686e10ab092962c773c2e7888e02459a2040b"],
                        [ "0e4674cf42696308fa0729dfc8d06fa585bfb3092bc3b4f5f5b4790ea0af8d02", "3e34896511b6239cf740ce7546e8f180cfc55f41fcaca0bd134a0b2490cf6f09"],
                        [ "fa3b5161ceaa438e153cd9b3a241d769094f375817816bdfd0dc8609c80c6204", "ee202594c70a70d77e9926ff89f46f081ce5212633515f1e618dfa5bf9733c00"],
                        [ "5adbe7989a85ccccd4125e9deab1346ed29a94f5d77a4b0ade86b9193a607902", "fae6243b65a87593f9772ea59f0bd06ea786cdc401939368209a78c03b306b00"],
                        [ "1f7cee23ce62a0b4fc6b121cd883e1306d60b51075f0d93877d8efb06a36790b", "2a13b8049e191264fd3fc6318295d7bb58c076fd27820b43df749cf2658ae901"],
                        [ "ec2efe94ba7539b78647f13b8105bb41fd89e3ea360c28f0a30f9b86d3260d0b", "9a1a4f0d780f33866aa758627855a062144c1ad3e0e72bc49eb9c7b49a746f0b"]
                    ].iter().map(|r| {
                        r.iter().map(|c| {
                            SecretKey::from_slice(&hex::decode(&c).unwrap())
                        }).collect()
                    }).collect(),
                    c: SecretKey::from_slice(&hex::decode("b4654dfe8dc883747ad343fb850738de31f0eb32737f0b53813782674b0b6c0e").unwrap()),
                    key_images: vec![KeyImage::from_slice(&hex::decode("008dd11210f1dd0fa344eadc1b0a628520f0c86bc9ef148ae407737f839be3e3").unwrap())]
                }
            ]
        };

        ringct::verify_multiple(&[sig]).unwrap();
    }
}