use crate::decoder::InstructionDecoder;
use crate::types::{DecodedInstruction, Dex, InstructionKind};
use solana_sdk::pubkey::Pubkey;

const PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

// SPL Token single-byte instruction discriminators
const TRANSFER: u8 = 3;
const MINT_TO: u8 = 7;
const BURN: u8 = 8;
const TRANSFER_CHECKED: u8 = 12;

// Transfer account layout:
// 0: source
// 1: destination
// 2: authority (signer)
const TRANSFER_SOURCE_INDEX: usize = 0;
const TRANSFER_DEST_INDEX: usize = 1;
const TRANSFER_AUTHORITY_INDEX: usize = 2;

// TransferChecked account layout:
// 0: source
// 1: mint
// 2: destination
// 3: authority (signer)
const CHECKED_SOURCE_INDEX: usize = 0;
const CHECKED_MINT_INDEX: usize = 1;
const CHECKED_DEST_INDEX: usize = 2;
const CHECKED_AUTHORITY_INDEX: usize = 3;

// MintTo account layout:
// 0: mint
// 1: destination
// 2: mint authority (signer)
const MINT_TO_MINT_INDEX: usize = 0;
const MINT_TO_AUTHORITY_INDEX: usize = 2;

// Burn account layout:
// 0: source account
// 1: mint
// 2: authority (signer)
const BURN_MINT_INDEX: usize = 1;
const BURN_AUTHORITY_INDEX: usize = 2;

pub struct SplTokenDecoder {
    program_id: Pubkey,
}

impl Default for SplTokenDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl SplTokenDecoder {
    pub fn new() -> Self {
        Self {
            program_id: PROGRAM_ID.parse().expect("valid SPL token program ID"),
        }
    }
}

impl InstructionDecoder for SplTokenDecoder {
    fn program_id(&self) -> Pubkey {
        self.program_id
    }

    fn dex(&self) -> Dex {
        Dex::SplToken
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

        match data[0] {
            TRANSFER => self.decode_transfer(accounts, instruction_accounts, data, signature, slot),
            TRANSFER_CHECKED => {
                self.decode_transfer_checked(accounts, instruction_accounts, data, signature, slot)
            }
            MINT_TO => self.decode_mint_to(accounts, instruction_accounts, data, signature, slot),
            BURN => self.decode_burn(accounts, instruction_accounts, data, signature, slot),
            _ => None,
        }
    }
}

impl SplTokenDecoder {
    fn decode_transfer(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        data: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        // Transfer data: [discriminator(1), amount(8)]
        if data.len() < 9 {
            return None;
        }

        let amount = u64::from_le_bytes(data[1..9].try_into().ok()?);

        let source_idx = instruction_accounts.get(TRANSFER_SOURCE_INDEX).copied()? as usize;
        let dest_idx = instruction_accounts.get(TRANSFER_DEST_INDEX).copied()? as usize;
        let auth_idx = instruction_accounts
            .get(TRANSFER_AUTHORITY_INDEX)
            .copied()? as usize;

        let source = accounts.get(source_idx).copied()?;
        let dest = accounts.get(dest_idx).copied()?;
        let authority = accounts.get(auth_idx).copied()?;

        tracing::debug!(
            dex = "spl_token",
            kind = "transfer",
            amount,
            source = %source,
            dest = %dest,
            signature,
            "Decoded SPL transfer"
        );

        Some(DecodedInstruction {
            dex: Dex::SplToken,
            kind: InstructionKind::Transfer,
            signature: signature.to_string(),
            slot,
            mint: source, // best-effort: token account, not mint
            input_mint: Some(source),
            output_mint: Some(dest),
            input_amount: Some(amount),
            output_amount: None,
            slippage_bps: None,
            authority,
        })
    }

    fn decode_transfer_checked(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        data: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        // TransferChecked data: [discriminator(1), amount(8), decimals(1)]
        if data.len() < 10 {
            return None;
        }

        let amount = u64::from_le_bytes(data[1..9].try_into().ok()?);

        let source_idx = instruction_accounts.get(CHECKED_SOURCE_INDEX).copied()? as usize;
        let mint_idx = instruction_accounts.get(CHECKED_MINT_INDEX).copied()? as usize;
        let dest_idx = instruction_accounts.get(CHECKED_DEST_INDEX).copied()? as usize;
        let auth_idx = instruction_accounts.get(CHECKED_AUTHORITY_INDEX).copied()? as usize;

        let source = accounts.get(source_idx).copied()?;
        let mint = accounts.get(mint_idx).copied()?;
        let dest = accounts.get(dest_idx).copied()?;
        let authority = accounts.get(auth_idx).copied()?;

        tracing::debug!(
            dex = "spl_token",
            kind = "transfer_checked",
            mint = %mint,
            amount,
            signature,
            "Decoded SPL transfer_checked"
        );

        Some(DecodedInstruction {
            dex: Dex::SplToken,
            kind: InstructionKind::TransferChecked,
            signature: signature.to_string(),
            slot,
            mint,
            input_mint: Some(source),
            output_mint: Some(dest),
            input_amount: Some(amount),
            output_amount: None,
            slippage_bps: None,
            authority,
        })
    }

    fn decode_mint_to(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        data: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        // MintTo data: [discriminator(1), amount(8)]
        if data.len() < 9 {
            return None;
        }

        let amount = u64::from_le_bytes(data[1..9].try_into().ok()?);

        let mint_idx = instruction_accounts.get(MINT_TO_MINT_INDEX).copied()? as usize;
        let auth_idx = instruction_accounts.get(MINT_TO_AUTHORITY_INDEX).copied()? as usize;

        let mint = accounts.get(mint_idx).copied()?;
        let authority = accounts.get(auth_idx).copied()?;

        tracing::debug!(
            dex = "spl_token",
            kind = "mint_to",
            mint = %mint,
            amount,
            signature,
            "Decoded SPL mint_to"
        );

        Some(DecodedInstruction {
            dex: Dex::SplToken,
            kind: InstructionKind::MintTo,
            signature: signature.to_string(),
            slot,
            mint,
            input_mint: None,
            output_mint: None,
            input_amount: Some(amount),
            output_amount: None,
            slippage_bps: None,
            authority,
        })
    }

    fn decode_burn(
        &self,
        accounts: &[Pubkey],
        instruction_accounts: &[u8],
        data: &[u8],
        signature: &str,
        slot: u64,
    ) -> Option<DecodedInstruction> {
        // Burn data: [discriminator(1), amount(8)]
        if data.len() < 9 {
            return None;
        }

        let amount = u64::from_le_bytes(data[1..9].try_into().ok()?);

        let mint_idx = instruction_accounts.get(BURN_MINT_INDEX).copied()? as usize;
        let auth_idx = instruction_accounts.get(BURN_AUTHORITY_INDEX).copied()? as usize;

        let mint = accounts.get(mint_idx).copied()?;
        let authority = accounts.get(auth_idx).copied()?;

        tracing::debug!(
            dex = "spl_token",
            kind = "burn",
            mint = %mint,
            amount,
            signature,
            "Decoded SPL burn"
        );

        Some(DecodedInstruction {
            dex: Dex::SplToken,
            kind: InstructionKind::Burn,
            signature: signature.to_string(),
            slot,
            mint,
            input_mint: None,
            output_mint: None,
            input_amount: Some(amount),
            output_amount: None,
            slippage_bps: None,
            authority,
        })
    }
}
