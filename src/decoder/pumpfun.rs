use crate::decoder::InstructionDecoder;
use crate::types::{DecodedInstruction, Dex, InstructionKind};
use solana_sdk::pubkey::Pubkey;

const PROGRAM_ID: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

// sha256("global:<method>")[..8]
const BUY_DISC: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
const BUY_EXACT_SOL_IN_DISC: [u8; 8] = [56, 252, 116, 8, 158, 223, 205, 95];
const SELL_DISC: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
const CREATE_DISC: [u8; 8] = [24, 30, 200, 40, 5, 28, 7, 119];
const CREATE_V2_DISC: [u8; 8] = [214, 144, 76, 236, 95, 139, 49, 180];

// buy/sell/buy_exact_sol_in accounts: [2] = mint, [6] = user
const MINT_INDEX: usize = 2;
const USER_INDEX: usize = 6;

// create v1 accounts: [0] = mint, [7] = user
const CREATE_MINT_INDEX: usize = 0;
const CREATE_USER_INDEX: usize = 7;

// create_v2 accounts (Token2022): [0] = mint, [5] = user
const CREATE_V2_MINT_INDEX: usize = 0;
const CREATE_V2_USER_INDEX: usize = 5;

pub struct PumpFunDecoder {
    program_id: Pubkey,
}

impl Default for PumpFunDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl PumpFunDecoder {
    pub fn new() -> Self {
        Self {
            program_id: PROGRAM_ID.parse().expect("valid pump.fun program ID"),
        }
    }
}

impl InstructionDecoder for PumpFunDecoder {
    fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn dex(&self) -> Dex {
        Dex::PumpFun
    }

    fn decode(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        data: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        if data.len() < 8 {
            return None;
        }

        let disc = &data[..8];

        if disc == CREATE_DISC {
            return self.decode_create(accounts, instruction_accounts, CREATE_MINT_INDEX, CREATE_USER_INDEX, signature, slot);
        }
        if disc == CREATE_V2_DISC {
            return self.decode_create(accounts, instruction_accounts, CREATE_V2_MINT_INDEX, CREATE_V2_USER_INDEX, signature, slot);
        }

        if data.len() < 24 {
            return None;
        }

        let (kind, input_amount, output_amount) = if disc == BUY_DISC {
            let token_amount = u64::from_le_bytes(data[8..16].try_into().ok()?);
            let max_sol = u64::from_le_bytes(data[16..24].try_into().ok()?);
            (InstructionKind::Buy, Some(max_sol), Some(token_amount))
        } else if disc == BUY_EXACT_SOL_IN_DISC {
            let sol_in = u64::from_le_bytes(data[8..16].try_into().ok()?);
            let min_tokens = u64::from_le_bytes(data[16..24].try_into().ok()?);
            (InstructionKind::Buy, Some(sol_in), Some(min_tokens))
        } else if disc == SELL_DISC {
            let token_amount = u64::from_le_bytes(data[8..16].try_into().ok()?);
            let min_sol = u64::from_le_bytes(data[16..24].try_into().ok()?);
            (InstructionKind::Sell, Some(token_amount), Some(min_sol))
        } else {
            return None;
        };

        let mint_idx = instruction_accounts.get(MINT_INDEX).copied()? as usize;
        let user_idx = instruction_accounts.get(USER_INDEX).copied()? as usize;
        let mint = accounts.get(mint_idx).copied()?;
        let authority = accounts.get(user_idx).copied()?;

        let sol_mint = solana_sdk::system_program::ID;
        let (input_mint, output_mint) = match kind {
            InstructionKind::Buy => (Some(sol_mint), Some(mint)),
            InstructionKind::Sell => (Some(mint), Some(sol_mint)),
            _ => (None, None),
        };

        Some(DecodedInstruction {
            dex: Dex::PumpFun,
            kind,
            signature: signature.to_string(),
            slot,
            mint,
            input_mint,
            output_mint,
            input_amount,
            output_amount,
            slippage_bps: None,
            authority,
        })
    }
}

impl PumpFunDecoder {
    fn decode_create(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        mint_account_index: usize,
        user_account_index: usize,
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        let mint_idx = instruction_accounts.get(mint_account_index).copied()? as usize;
        let user_idx = instruction_accounts.get(user_account_index).copied()? as usize;
        let mint = accounts.get(mint_idx).copied()?;
        let authority = accounts.get(user_idx).copied()?;

        Some(DecodedInstruction {
            dex: Dex::PumpFun,
            kind: InstructionKind::Create,
            signature: signature.to_string(),
            slot,
            mint,
            input_mint: None,
            output_mint: None,
            input_amount: None,
            output_amount: None,
            slippage_bps: None,
            authority,
        })
    }
}
