use crate::decoder::InstructionDecoder;
use crate::types::{DecodedInstruction, Dex, InstructionKind};
use solana_sdk::pubkey::Pubkey;

const PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

// sha256("global:<method>")[..8]
// Route family - only destinationMint in accounts (index 5)
const ROUTE_DISC: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
const ROUTE_V2_DISC: [u8; 8] = [187, 100, 250, 204, 49, 196, 175, 20];
const ROUTE_WITH_TOKEN_LEDGER_DISC: [u8; 8] = [150, 86, 71, 116, 167, 93, 14, 104];
const EXACT_OUT_ROUTE_DISC: [u8; 8] = [208, 51, 239, 151, 123, 43, 237, 92];

// SharedAccountsRoute family - both sourceMint (7) and destinationMint (8)
const SHARED_ACCOUNTS_ROUTE_DISC: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];
const SHARED_ACCOUNTS_ROUTE_V2_DISC: [u8; 8] = [209, 152, 83, 147, 124, 254, 216, 233];
const SHARED_ACCOUNTS_ROUTE_WITH_TOKEN_LEDGER_DISC: [u8; 8] = [230, 121, 143, 80, 119, 159, 106, 170];
const SHARED_ACCOUNTS_EXACT_OUT_DISC: [u8; 8] = [176, 209, 105, 168, 154, 125, 69, 62];

// Route: [1] = user, [5] = dest mint
const ROUTE_USER_INDEX: usize = 1;
const ROUTE_DEST_MINT_INDEX: usize = 5;

// SharedAccountsRoute: [2] = user, [7] = source mint, [8] = dest mint
const SHARED_USER_INDEX: usize = 2;
const SHARED_SOURCE_MINT_INDEX: usize = 7;
const SHARED_DEST_MINT_INDEX: usize = 8;

pub struct JupiterDecoder {
    program_id: Pubkey,
}

impl Default for JupiterDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl JupiterDecoder {
    pub fn new() -> Self {
        Self {
            program_id: PROGRAM_ID.parse().expect("valid jupiter program ID"),
        }
    }
}

fn is_sol(mint: &Pubkey) -> bool {
    let wsol: Pubkey = "So11111111111111111111111111111111111111112"
        .parse()
        .unwrap();
    *mint == solana_sdk::system_program::ID || *mint == wsol
}

impl InstructionDecoder for JupiterDecoder {
    fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn dex(&self) -> Dex {
        Dex::JupiterV6
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

        if disc == SHARED_ACCOUNTS_ROUTE_DISC
            || disc == SHARED_ACCOUNTS_ROUTE_V2_DISC
            || disc == SHARED_ACCOUNTS_ROUTE_WITH_TOKEN_LEDGER_DISC
            || disc == SHARED_ACCOUNTS_EXACT_OUT_DISC
        {
            return self.decode_shared_route(accounts, instruction_accounts, data, signature, slot);
        }

        if disc == ROUTE_DISC
            || disc == ROUTE_V2_DISC
            || disc == ROUTE_WITH_TOKEN_LEDGER_DISC
            || disc == EXACT_OUT_ROUTE_DISC
        {
            return self.decode_route(accounts, instruction_accounts, data, signature, slot);
        }

        None
    }
}

impl JupiterDecoder {
    fn decode_shared_route(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        data: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        let source_mint_idx = instruction_accounts.get(SHARED_SOURCE_MINT_INDEX).copied()? as usize;
        let dest_mint_idx = instruction_accounts.get(SHARED_DEST_MINT_INDEX).copied()? as usize;
        let user_idx = instruction_accounts.get(SHARED_USER_INDEX).copied()? as usize;

        let source_mint = accounts.get(source_mint_idx).copied()?;
        let dest_mint = accounts.get(dest_mint_idx).copied()?;
        let authority = accounts.get(user_idx).copied()?;

        let (in_amount, out_amount, slippage_bps) = Self::parse_amounts(data);

        let kind = if is_sol(&source_mint) { InstructionKind::Buy } else { InstructionKind::Sell };
        let mint = if is_sol(&source_mint) { dest_mint } else { source_mint };

        Some(DecodedInstruction {
            dex: Dex::JupiterV6,
            kind,
            signature: signature.to_string(),
            slot,
            mint,
            input_mint: Some(source_mint),
            output_mint: Some(dest_mint),
            input_amount: in_amount,
            output_amount: out_amount,
            slippage_bps,
            authority,
        })
    }

    fn decode_route(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        data: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        let dest_mint_idx = instruction_accounts.get(ROUTE_DEST_MINT_INDEX).copied()? as usize;
        let user_idx = instruction_accounts.get(ROUTE_USER_INDEX).copied()? as usize;

        let dest_mint = accounts.get(dest_mint_idx).copied()?;
        let authority = accounts.get(user_idx).copied()?;

        let (in_amount, out_amount, slippage_bps) = Self::parse_amounts(data);

        // No source mint in route accounts, infer direction from dest
        let kind = if is_sol(&dest_mint) { InstructionKind::Sell } else { InstructionKind::Buy };

        Some(DecodedInstruction {
            dex: Dex::JupiterV6,
            kind,
            signature: signature.to_string(),
            slot,
            mint: dest_mint,
            input_mint: None,
            output_mint: Some(dest_mint),
            input_amount: in_amount,
            output_amount: out_amount,
            slippage_bps,
            authority,
        })
    }

    // Args are variable-length (routePlan vec) but the fixed fields sit at the tail:
    // in_amount(8) + quoted_out(8) + slippage_bps(2) + platform_fee_bps(1) = 19 bytes
    fn parse_amounts(data: &[u8]) -> (Option<u64>, Option<u64>, Option<u32>) {
        if data.len() >= 19 {
            let tail = &data[data.len() - 19..];
            let in_amount = u64::from_le_bytes(tail[0..8].try_into().unwrap_or_default());
            let out_amount = u64::from_le_bytes(tail[8..16].try_into().unwrap_or_default());
            let slippage = u16::from_le_bytes(tail[16..18].try_into().unwrap_or_default());
            (Some(in_amount), Some(out_amount), Some(slippage as u32))
        } else {
            (None, None, None)
        }
    }
}
