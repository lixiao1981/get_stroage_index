//! RLP codec for Erigon PlainState account encoding
//!
//! Erigon uses a custom RLP encoding with field omission optimization:
//! - Zero values may be omitted to save space
//! - Fields are encoded in order: nonce, balance, storage_root, code_hash, incarnation

use crate::error::{Error, Result};
use crate::model::PlainAccount;
use alloy_primitives::{FixedBytes, U256};
use alloy_rlp::Decodable;

/// Helper trait to advance buffer
trait BufExt {
    fn advance(&mut self, n: usize);
}

impl BufExt for &[u8] {
    fn advance(&mut self, n: usize) {
        *self = &self[n..];
    }
}

impl Decodable for PlainAccount {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        // Erigon encodes accounts as RLP lists with field omission
        // Format: [nonce, balance, storage_root?, code_hash?, incarnation?]
        
        let header = alloy_rlp::Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::UnexpectedString);
        }

        let payload_len = header.payload_length;
        let start_len = buf.len();
        
        // Decode nonce (field 0)
        let nonce = u64::decode(buf)?;

        // Decode balance (field 1) - decode as bytes then convert to U256
        let balance_header = alloy_rlp::Header::decode(buf)?;
        let balance_bytes = &buf[..balance_header.payload_length];
        let balance = U256::from_be_slice(balance_bytes);
        buf.advance(balance_header.payload_length);

        // Calculate how much we've consumed
        let consumed = start_len - buf.len();
        
        // Decode optional fields for contracts
        let storage_root = if consumed < payload_len {
            let header = alloy_rlp::Header::decode(buf)?;
            if header.payload_length == 0 {
                None
            } else if header.payload_length == 32 {
                let bytes = &buf[..32];
                buf.advance(32);
                Some(FixedBytes::<32>::from_slice(bytes))
            } else {
                None
            }
        } else {
            None
        };

        let consumed = start_len - buf.len();
        let code_hash = if consumed < payload_len {
            let header = alloy_rlp::Header::decode(buf)?;
            if header.payload_length == 0 {
                None
            } else if header.payload_length == 32 {
                let bytes = &buf[..32];
                buf.advance(32);
                Some(FixedBytes::<32>::from_slice(bytes))
            } else {
                None
            }
        } else {
            None
        };

        let consumed = start_len - buf.len();
        let incarnation = if consumed < payload_len {
            Some(u64::decode(buf)?)
        } else {
            None
        };

        Ok(PlainAccount {
            nonce,
            balance,
            storage_root,
            code_hash,
            incarnation,
        })
    }
}

/// Decode account data from RLP bytes
pub fn decode_account(data: &[u8]) -> Result<PlainAccount> {
    PlainAccount::decode(&mut &data[..]).map_err(|e| Error::rlp_decode("unknown", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_decode_eoa() {
        // RLP encoded EOA: [nonce=5, balance=1000]
        // Simplified example - actual encoding would be more complex
        let data = hex!("c20564");
        
        // This is a placeholder - real test needs actual Erigon RLP data
        // For now, we test the structure
    }

    #[test]
    fn test_decode_contract() {
        // Test decoding contract account with all fields
        // Placeholder for actual Erigon RLP data
    }

    #[test]
    fn test_decode_zero_balance() {
        // Test field omission for zero values
        // Placeholder for actual test
    }
}
