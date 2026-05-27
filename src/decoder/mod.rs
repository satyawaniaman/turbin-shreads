pub mod jupiter;
pub mod pumpfun;
pub mod raydium;
pub mod spl_token;

use crate::types::{DecodedInstruction, Dex};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

pub trait InstructionDecoder: Send + Sync {
    fn program_id(&self) -> Pubkey;
    fn dex(&self) -> Dex;
    fn decode(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        data: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction>;
}

pub struct DecoderRegistry {
    decoders: HashMap<Pubkey, Vec<Box<dyn InstructionDecoder>>>,
}

impl DecoderRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            decoders: HashMap::new(),
        };

        registry.register(Box::new(pumpfun::PumpFunDecoder::new()));
        registry.register(Box::new(jupiter::JupiterDecoder::new()));
        registry.register(Box::new(raydium::RaydiumAmmDecoder::new()));
        registry.register(Box::new(raydium::RaydiumCpmmDecoder::new()));
        registry.register(Box::new(spl_token::SplTokenDecoder::new()));

        tracing::info!(
            decoders = registry.decoders.len(),
            "DecoderRegistry initialized"
        );

        registry
    }

    fn register(&mut self, decoder: Box<dyn InstructionDecoder>) {
        let pid = decoder.program_id();
        self.decoders.entry(pid).or_default().push(decoder);
    }

    pub fn decode_transaction(
        &self,
        accounts: &[Pubkey],
        instructions: &[(u8, Vec<u8>, Vec<u8>)],
        signature: &str,
        slot: u64,
    ) -> Vec<DecodedInstruction> {
        let mut results = Vec::new();

        for (program_id_index, ix_accounts, data) in instructions {
            let program_idx = *program_id_index as usize;
            let Some(&program_id) = accounts.get(program_idx) else {
                continue;
            };

            if let Some(decoders) = self.decoders.get(&program_id) {
                for decoder in decoders {
                    if let Some(decoded) =
                        decoder.decode(accounts, ix_accounts, data, signature, slot)
                    {
                        results.push(decoded);
                    }
                }
            }
        }

        results
    }

    pub fn program_ids(&self) -> Vec<Pubkey> {
        self.decoders.keys().copied().collect()
    }
}

impl Default for DecoderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
