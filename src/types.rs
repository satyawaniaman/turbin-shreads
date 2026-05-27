use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Dex {
    PumpFun,
    JupiterV6,
    RaydiumAmm,
    RaydiumCpmm,
    SplToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum InstructionKind {
    Buy,
    Sell,
    Create,
    Swap,
    Transfer,
    TransferChecked,
    MintTo,
    Burn,
    PoolInit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedInstruction {
    pub dex: Dex,
    pub kind: InstructionKind,
    pub signature: String,
    pub slot: u64,
    #[serde(with = "pubkey_serde")]
    pub mint: Pubkey,
    #[serde(with = "option_pubkey_serde")]
    pub input_mint: Option<Pubkey>,
    #[serde(with = "option_pubkey_serde")]
    pub output_mint: Option<Pubkey>,
    pub input_amount: Option<u64>,
    pub output_amount: Option<u64>,
    pub slippage_bps: Option<u32>,
    #[serde(with = "pubkey_serde")]
    pub authority: Pubkey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShredInfo {
    pub slot: u64,
    pub index: u32,
    pub version: u16,
    pub fec_set_index: u32,
    pub is_data: bool,
    pub payload_size: usize,
}

mod pubkey_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use solana_sdk::pubkey::Pubkey;

    pub fn serialize<S>(key: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&key.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

mod option_pubkey_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use solana_sdk::pubkey::Pubkey;

    pub fn serialize<S>(key: &Option<Pubkey>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match key {
            Some(k) => serializer.serialize_some(&k.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Pubkey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}
