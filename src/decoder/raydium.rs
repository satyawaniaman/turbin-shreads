use crate::decoder::InstructionDecoder;
use crate::types::{DecodedInstruction, Dex, InstructionKind};
use solana_sdk::pubkey::Pubkey;

// Raydium AMM

const AMM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

const AMM_SWAP_BASE_IN: u8 = 9;
const AMM_SWAP_BASE_OUT: u8 = 11;
const AMM_INITIALIZE2: u8 = 1;

// Swap accounts: [5] = pool coin, [6] = pool pc, [17] = user
const AMM_POOL_COIN_INDEX: usize = 5;
const AMM_POOL_PC_INDEX: usize = 6;
const AMM_USER_OWNER_INDEX: usize = 17;

const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";

pub struct RaydiumAmmDecoder {
    program_id: Pubkey,
}

impl Default for RaydiumAmmDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl RaydiumAmmDecoder {
    pub fn new() -> Self {
        Self {
            program_id: AMM_PROGRAM_ID.parse().expect("valid raydium AMM program ID"),
        }
    }
}

impl InstructionDecoder for RaydiumAmmDecoder {
    fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn dex(&self) -> Dex {
        Dex::RaydiumAmm
    }

    fn decode(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        data: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        if data.is_empty() {
            return None;
        }

        let instruction_type = data[0];

        if instruction_type == AMM_INITIALIZE2 {
            return self.decode_pool_init(accounts, instruction_accounts, signature, slot);
        }

        if instruction_type != AMM_SWAP_BASE_IN && instruction_type != AMM_SWAP_BASE_OUT {
            return None;
        }

        if data.len() < 17 {
            return None;
        }

        let amount_a = u64::from_le_bytes(data[1..9].try_into().ok()?);
        let amount_b = u64::from_le_bytes(data[9..17].try_into().ok()?);

        let (input_amount, output_amount) = if instruction_type == AMM_SWAP_BASE_IN {
            (amount_a, amount_b)
        } else {
            (amount_b, amount_a)
        };

        let pool_coin_idx = instruction_accounts.get(AMM_POOL_COIN_INDEX).copied()? as usize;
        let pool_pc_idx = instruction_accounts.get(AMM_POOL_PC_INDEX).copied()? as usize;
        let user_idx = instruction_accounts.get(AMM_USER_OWNER_INDEX).copied()? as usize;

        let pool_coin = accounts.get(pool_coin_idx).copied()?;
        let pool_pc = accounts.get(pool_pc_idx).copied()?;
        let authority = accounts.get(user_idx).copied()?;

        Some(DecodedInstruction {
            dex: Dex::RaydiumAmm,
            kind: InstructionKind::Swap,
            signature: signature.to_string(),
            slot,
            mint: pool_coin,
            input_mint: Some(pool_coin),
            output_mint: Some(pool_pc),
            input_amount: Some(input_amount),
            output_amount: Some(output_amount),
            slippage_bps: None,
            authority,
        })
    }
}

impl RaydiumAmmDecoder {
    fn decode_pool_init(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        // initialize2 accounts: [8] = coin mint, [9] = pc mint, [17] = user
        let coin_mint_idx = instruction_accounts.get(8).copied()? as usize;
        let pc_mint_idx = instruction_accounts.get(9).copied()? as usize;
        let user_idx = instruction_accounts.get(17).copied()? as usize;

        let coin_mint = accounts.get(coin_mint_idx).copied()?;
        let pc_mint = accounts.get(pc_mint_idx).copied()?;
        let authority = accounts.get(user_idx).copied()?;

        let wsol: Pubkey = WSOL_MINT.parse().unwrap();
        let sol = solana_sdk::system_program::ID;

        let mint = if coin_mint == wsol || coin_mint == sol { pc_mint } else { coin_mint };

        Some(DecodedInstruction {
            dex: Dex::RaydiumAmm,
            kind: InstructionKind::PoolInit,
            signature: signature.to_string(),
            slot,
            mint,
            input_mint: Some(coin_mint),
            output_mint: Some(pc_mint),
            input_amount: None,
            output_amount: None,
            slippage_bps: None,
            authority,
        })
    }
}

// Raydium CPMM

const CPMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";

const CPMM_SWAP_BASE_INPUT_DISC: [u8; 8] = [143, 190, 90, 218, 196, 30, 51, 222];
const CPMM_SWAP_BASE_OUTPUT_DISC: [u8; 8] = [55, 217, 98, 86, 163, 74, 180, 173];
const CPMM_INITIALIZE_DISC: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];

// Swap accounts: [0] = payer, [10] = input mint, [11] = output mint
const CPMM_PAYER_INDEX: usize = 0;
const CPMM_INPUT_MINT_INDEX: usize = 10;
const CPMM_OUTPUT_MINT_INDEX: usize = 11;

pub struct RaydiumCpmmDecoder {
    program_id: Pubkey,
}

impl Default for RaydiumCpmmDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl RaydiumCpmmDecoder {
    pub fn new() -> Self {
        Self {
            program_id: CPMM_PROGRAM_ID.parse().expect("valid raydium CPMM program ID"),
        }
    }
}

impl InstructionDecoder for RaydiumCpmmDecoder {
    fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn dex(&self) -> Dex {
        Dex::RaydiumCpmm
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

        if disc == CPMM_INITIALIZE_DISC {
            return self.decode_pool_init(accounts, instruction_accounts, signature, slot);
        }

        let is_base_input = disc == CPMM_SWAP_BASE_INPUT_DISC;
        let is_base_output = disc == CPMM_SWAP_BASE_OUTPUT_DISC;

        if !is_base_input && !is_base_output {
            return None;
        }

        if data.len() < 24 {
            return None;
        }

        let amount_a = u64::from_le_bytes(data[8..16].try_into().ok()?);
        let amount_b = u64::from_le_bytes(data[16..24].try_into().ok()?);

        let (input_amount, output_amount) = if is_base_input {
            (amount_a, amount_b)
        } else {
            (amount_b, amount_a)
        };

        let payer_idx = instruction_accounts.get(CPMM_PAYER_INDEX).copied()? as usize;
        let input_mint_idx = instruction_accounts.get(CPMM_INPUT_MINT_INDEX).copied()? as usize;
        let output_mint_idx = instruction_accounts.get(CPMM_OUTPUT_MINT_INDEX).copied()? as usize;

        let authority = accounts.get(payer_idx).copied()?;
        let input_mint = accounts.get(input_mint_idx).copied()?;
        let output_mint = accounts.get(output_mint_idx).copied()?;

        let wsol_mint: Pubkey = WSOL_MINT.parse().unwrap();
        let sol_mint = solana_sdk::system_program::ID;

        let kind = if input_mint == sol_mint || input_mint == wsol_mint {
            InstructionKind::Buy
        } else {
            InstructionKind::Sell
        };

        let mint = if input_mint == sol_mint || input_mint == wsol_mint {
            output_mint
        } else {
            input_mint
        };

        Some(DecodedInstruction {
            dex: Dex::RaydiumCpmm,
            kind,
            signature: signature.to_string(),
            slot,
            mint,
            input_mint: Some(input_mint),
            output_mint: Some(output_mint),
            input_amount: Some(input_amount),
            output_amount: Some(output_amount),
            slippage_bps: None,
            authority,
        })
    }
}

impl RaydiumCpmmDecoder {
    fn decode_pool_init(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        // initialize accounts: [0] = creator, [5] = token_0_mint, [6] = token_1_mint
        let creator_idx = instruction_accounts.first().copied()? as usize;
        let mint0_idx = instruction_accounts.get(5).copied()? as usize;
        let mint1_idx = instruction_accounts.get(6).copied()? as usize;

        let authority = accounts.get(creator_idx).copied()?;
        let mint0 = accounts.get(mint0_idx).copied()?;
        let mint1 = accounts.get(mint1_idx).copied()?;

        let wsol: Pubkey = WSOL_MINT.parse().unwrap();
        let sol = solana_sdk::system_program::ID;

        let mint = if mint0 == wsol || mint0 == sol { mint1 } else { mint0 };

        Some(DecodedInstruction {
            dex: Dex::RaydiumCpmm,
            kind: InstructionKind::PoolInit,
            signature: signature.to_string(),
            slot,
            mint,
            input_mint: Some(mint0),
            output_mint: Some(mint1),
            input_amount: None,
            output_amount: None,
            slippage_bps: None,
            authority,
        })
    }
}
