//! Isthmus L1 Block Info transaction types.

use alloc::vec::Vec;
use alloy_primitives::{Address, B256, Bytes};

use crate::{DecodeError, L1BlockInfoEcotone};

/// Represents the fields within an Isthnus L1 block info transaction.
///
/// Isthmus Binary Format
/// +---------+--------------------------+
/// | Bytes   | Field                    |
/// +---------+--------------------------+
/// | 4       | Function signature       |
/// | 4       | BaseFeeScalar            |
/// | 4       | BlobBaseFeeScalar        |
/// | 8       | SequenceNumber           |
/// | 8       | Timestamp                |
/// | 8       | L1BlockNumber            |
/// | 32      | BaseFee                  |
/// | 32      | BlobBaseFee              |
/// | 32      | BlockHash                |
/// | 32      | BatcherHash              |
/// | 4       | OperatorFeeScalar        |
/// | 8       | OperatorFeeConstant      |
/// +---------+--------------------------+
#[derive(Debug, Clone, Hash, Eq, PartialEq, Default, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct L1BlockInfoIsthmus {
    /// The current L1 origin block number
    pub number: u64,
    /// The current L1 origin block's timestamp
    pub time: u64,
    /// The current L1 origin block's basefee
    pub base_fee: u64,
    /// The current L1 origin block's hash
    pub block_hash: B256,
    /// The current sequence number
    pub sequence_number: u64,
    /// The address of the batch submitter
    pub batcher_address: Address,
    /// The current blob base fee on L1
    pub blob_base_fee: u128,
    /// The fee scalar for L1 blobspace data
    pub blob_base_fee_scalar: u32,
    /// The fee scalar for L1 data
    pub base_fee_scalar: u32,
    /// The operator fee scalar
    pub operator_fee_scalar: u32,
    /// The operator fee constant
    pub operator_fee_constant: u64,
}

impl L1BlockInfoIsthmus {
    /// The type byte identifier for the L1 scalar format in Isthmus.
    pub const L1_SCALAR: u8 = 2;

    /// The length of an L1 info transaction in Isthmus.
    pub const L1_INFO_TX_LEN: usize = 4 + 32 * 5 + 4 + 8;

    /// The 4 byte selector of "setL1BlockValuesIsthmus()"
    pub const L1_INFO_TX_SELECTOR: [u8; 4] = [0x09, 0x89, 0x99, 0xbe];

    /// Converts this Isthmus info to an Ecotone info (base hardfork).
    ///
    /// This is useful for encoding/decoding the common fields that are shared
    /// between Ecotone and Isthmus.
    fn to_ecotone(&self) -> L1BlockInfoEcotone {
        L1BlockInfoEcotone {
            number: self.number,
            time: self.time,
            base_fee: self.base_fee,
            block_hash: self.block_hash,
            sequence_number: self.sequence_number,
            batcher_address: self.batcher_address,
            blob_base_fee: self.blob_base_fee,
            blob_base_fee_scalar: self.blob_base_fee_scalar,
            base_fee_scalar: self.base_fee_scalar,
            empty_scalars: false,
            l1_fee_overhead: alloy_primitives::U256::ZERO,
        }
    }

    /// Creates an Isthmus info from an Ecotone info and Isthmus-specific fields.
    fn from_ecotone(
        ecotone: L1BlockInfoEcotone,
        operator_fee_scalar: u32,
        operator_fee_constant: u64,
    ) -> Self {
        Self {
            number: ecotone.number,
            time: ecotone.time,
            base_fee: ecotone.base_fee,
            block_hash: ecotone.block_hash,
            sequence_number: ecotone.sequence_number,
            batcher_address: ecotone.batcher_address,
            blob_base_fee: ecotone.blob_base_fee,
            blob_base_fee_scalar: ecotone.blob_base_fee_scalar,
            base_fee_scalar: ecotone.base_fee_scalar,
            operator_fee_scalar,
            operator_fee_constant,
        }
    }

    /// Encodes the Isthmus-specific fields into a buffer.
    ///
    /// This should be called after encoding the base Ecotone fields.
    pub(crate) fn encode_isthmus_fields(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.operator_fee_scalar.to_be_bytes().as_ref());
        buf.extend_from_slice(self.operator_fee_constant.to_be_bytes().as_ref());
    }

    /// Decodes the Isthmus-specific fields from calldata.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `r` is at least 176 bytes long.
    pub(crate) fn decode_isthmus_fields(r: &[u8]) -> (u32, u64) {
        // SAFETY: 4 bytes are copied directly into the array
        let mut operator_fee_scalar = [0u8; 4];
        operator_fee_scalar.copy_from_slice(&r[164..168]);
        let operator_fee_scalar = u32::from_be_bytes(operator_fee_scalar);

        // SAFETY: 8 bytes are copied directly into the array
        let mut operator_fee_constant = [0u8; 8];
        operator_fee_constant.copy_from_slice(&r[168..176]);
        let operator_fee_constant = u64::from_be_bytes(operator_fee_constant);

        (operator_fee_scalar, operator_fee_constant)
    }

    /// Encodes the [`L1BlockInfoIsthmus`] object into Ethereum transaction calldata.
    pub fn encode_calldata(&self) -> Bytes {
        // Start with Ecotone base fields
        let ecotone = self.to_ecotone();
        let mut buf = ecotone.encode_base_fields();

        // Replace the selector with Isthmus selector
        buf[0..4].copy_from_slice(&Self::L1_INFO_TX_SELECTOR);

        // Add Isthmus-specific fields
        self.encode_isthmus_fields(&mut buf);

        buf.into()
    }

    /// Decodes the [`L1BlockInfoIsthmus`] object from ethereum transaction calldata.
    pub fn decode_calldata(r: &[u8]) -> Result<Self, DecodeError> {
        if r.len() != Self::L1_INFO_TX_LEN {
            return Err(DecodeError::InvalidIsthmusLength(Self::L1_INFO_TX_LEN, r.len()));
        }

        // SAFETY: For all below slice operations, the full
        //         length is validated above to be `176`.

        // Decode base Ecotone fields
        let ecotone = L1BlockInfoEcotone::decode_base_fields(r);

        // Decode Isthmus-specific fields
        let (operator_fee_scalar, operator_fee_constant) = Self::decode_isthmus_fields(r);

        Ok(Self::from_ecotone(ecotone, operator_fee_scalar, operator_fee_constant))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_decode_calldata_isthmus_invalid_length() {
        let r = vec![0u8; 1];
        assert_eq!(
            L1BlockInfoIsthmus::decode_calldata(&r),
            Err(DecodeError::InvalidIsthmusLength(L1BlockInfoIsthmus::L1_INFO_TX_LEN, r.len()))
        );
    }

    #[test]
    fn test_l1_block_info_isthmus_roundtrip_calldata_encoding() {
        let info = L1BlockInfoIsthmus {
            number: 1,
            time: 2,
            base_fee: 3,
            block_hash: B256::from([4; 32]),
            sequence_number: 5,
            batcher_address: Address::from_slice(&[6; 20]),
            blob_base_fee: 7,
            blob_base_fee_scalar: 8,
            base_fee_scalar: 9,
            operator_fee_scalar: 10,
            operator_fee_constant: 11,
        };

        let calldata = info.encode_calldata();
        let decoded_info = L1BlockInfoIsthmus::decode_calldata(&calldata).unwrap();

        assert_eq!(info, decoded_info);
    }
}
