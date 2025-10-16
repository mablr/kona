//! Jovian L1 Block Info transaction types.

use alloc::vec::Vec;
use alloy_primitives::{Address, B256, Bytes};

use crate::{DecodeError, L1BlockInfoIsthmus};

/// Represents the fields within an Jovian L1 block info transaction.
///
/// Jovian Binary Format
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
/// | 2       | DAFootprintGasScalar     |
/// +---------+--------------------------+
#[derive(Debug, Clone, Hash, Eq, PartialEq, Default, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct L1BlockInfoJovian {
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
    /// The DA footprint gas scalar
    pub da_footprint_gas_scalar: u16,
}

impl L1BlockInfoJovian {
    /// The default DA footprint gas scalar
    /// <https://github.com/ethereum-optimism/specs/blob/main/specs/protocol/jovian/l1-attributes.md#overview>
    pub const DEFAULT_DA_FOOTPRINT_GAS_SCALAR: u16 = 400;

    /// The type byte identifier for the L1 scalar format in Jovian.
    pub const L1_SCALAR: u8 = 2;

    /// The length of an L1 info transaction in Jovian.
    pub const L1_INFO_TX_LEN: usize = 4 + 32 * 5 + 4 + 8 + 2;

    /// The 4 byte selector of "setL1BlockValuesJovian()"
    /// Those are the first 4 calldata bytes -> `<https://github.com/ethereum-optimism/specs/blob/main/specs/protocol/jovian/l1-attributes.md#overview>`
    pub const L1_INFO_TX_SELECTOR: [u8; 4] = [0x3d, 0xb6, 0xbe, 0x2b];

    /// Converts this Jovian info to an Isthmus info (parent hardfork).
    ///
    /// This is useful for encoding/decoding the common fields that are shared
    /// between Isthmus and Jovian.
    fn to_isthmus(&self) -> L1BlockInfoIsthmus {
        L1BlockInfoIsthmus {
            number: self.number,
            time: self.time,
            base_fee: self.base_fee,
            block_hash: self.block_hash,
            sequence_number: self.sequence_number,
            batcher_address: self.batcher_address,
            blob_base_fee: self.blob_base_fee,
            blob_base_fee_scalar: self.blob_base_fee_scalar,
            base_fee_scalar: self.base_fee_scalar,
            operator_fee_scalar: self.operator_fee_scalar,
            operator_fee_constant: self.operator_fee_constant,
        }
    }

    /// Creates a Jovian info from an Isthmus info and Jovian-specific fields.
    fn from_isthmus(isthmus: L1BlockInfoIsthmus, da_footprint_gas_scalar: u16) -> Self {
        Self {
            number: isthmus.number,
            time: isthmus.time,
            base_fee: isthmus.base_fee,
            block_hash: isthmus.block_hash,
            sequence_number: isthmus.sequence_number,
            batcher_address: isthmus.batcher_address,
            blob_base_fee: isthmus.blob_base_fee,
            blob_base_fee_scalar: isthmus.blob_base_fee_scalar,
            base_fee_scalar: isthmus.base_fee_scalar,
            operator_fee_scalar: isthmus.operator_fee_scalar,
            operator_fee_constant: isthmus.operator_fee_constant,
            da_footprint_gas_scalar,
        }
    }

    /// Encodes the Jovian-specific fields into a buffer.
    ///
    /// This should be called after encoding the base Isthmus fields.
    pub(crate) fn encode_jovian_fields(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.da_footprint_gas_scalar.to_be_bytes().as_ref());
    }

    /// Decodes the Jovian-specific fields from calldata.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `r` is at least 178 bytes long.
    pub(crate) fn decode_jovian_fields(r: &[u8]) -> u16 {
        // SAFETY: 2 bytes are copied directly into the array
        let mut da_footprint_gas_scalar = [0u8; 2];
        da_footprint_gas_scalar.copy_from_slice(&r[176..178]);
        u16::from_be_bytes(da_footprint_gas_scalar)
    }

    /// Encodes the [`L1BlockInfoJovian`] object into Ethereum transaction calldata.
    pub fn encode_calldata(&self) -> Bytes {
        // Start with Isthmus fields (which includes Ecotone base fields)
        let isthmus = self.to_isthmus();
        let mut buf = isthmus.encode_calldata().to_vec();

        // Replace the selector with Jovian selector
        buf[0..4].copy_from_slice(&Self::L1_INFO_TX_SELECTOR);

        // Add Jovian-specific fields
        self.encode_jovian_fields(&mut buf);

        buf.into()
    }

    /// Decodes the [`L1BlockInfoJovian`] object from ethereum transaction calldata.
    pub fn decode_calldata(r: &[u8]) -> Result<Self, DecodeError> {
        if r.len() != Self::L1_INFO_TX_LEN {
            return Err(DecodeError::InvalidJovianLength(Self::L1_INFO_TX_LEN, r.len()));
        }

        // SAFETY: For all below slice operations, the full
        //         length is validated above to be `178`.

        // Decode base Isthmus fields (which includes Ecotone base fields)
        // We can reuse Isthmus decoding for the first 176 bytes
        let isthmus = L1BlockInfoIsthmus::decode_calldata(&r[..L1BlockInfoIsthmus::L1_INFO_TX_LEN])
            .map_err(|_| DecodeError::InvalidJovianLength(Self::L1_INFO_TX_LEN, r.len()))?;

        // Decode Jovian-specific fields
        let da_footprint_gas_scalar = Self::decode_jovian_fields(r);

        Ok(Self::from_isthmus(isthmus, da_footprint_gas_scalar))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_decode_calldata_jovian_invalid_length() {
        let r = vec![0u8; 1];
        assert_eq!(
            L1BlockInfoJovian::decode_calldata(&r),
            Err(DecodeError::InvalidJovianLength(L1BlockInfoJovian::L1_INFO_TX_LEN, r.len()))
        );
    }

    #[test]
    fn test_l1_block_info_jovian_roundtrip_calldata_encoding() {
        let info = L1BlockInfoJovian {
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
            da_footprint_gas_scalar: 12,
        };

        let calldata = info.encode_calldata();
        let decoded_info = L1BlockInfoJovian::decode_calldata(&calldata).unwrap();

        assert_eq!(info, decoded_info);
    }
}
