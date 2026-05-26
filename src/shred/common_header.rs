pub const COMMON_HEADER_SIZE: usize = 83;
pub const SIGNATURE_SIZE: usize = 64;

#[derive(Debug, Clone)]
pub struct CommonHeader {
    pub signature: [u8; SIGNATURE_SIZE],
    pub variant: ShredVariant,
    pub slot: u64,
    pub index: u32,
    pub version: u16,
    pub fec_set_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShredVariant {
    LegacyData,
    LegacyCoding,
    MerkleData {
        proof_size: u8,
        chained: bool,
        resigned: bool,
    },
    MerkleCoding {
        proof_size: u8,
        chained: bool,
        resigned: bool,
    },
}

impl ShredVariant {
    pub fn is_data(&self) -> bool {
        matches!(
            self,
            ShredVariant::LegacyData | ShredVariant::MerkleData { .. }
        )
    }

    pub fn is_coding(&self) -> bool {
        matches!(
            self,
            ShredVariant::LegacyCoding | ShredVariant::MerkleCoding { .. }
        )
    }

    pub fn merkle_proof_bytes(&self) -> usize {
        match self {
            ShredVariant::MerkleData {
                proof_size,
                chained,
                resigned,
            }
            | ShredVariant::MerkleCoding {
                proof_size,
                chained,
                resigned,
            } => {
                *proof_size as usize * 20
                    + if *chained { 32 } else { 0 }
                    + if *resigned { 64 } else { 0 }
            }
            _ => 0,
        }
    }
}

fn parse_variant_byte(byte: u8) -> Option<ShredVariant> {
    match byte {
        0xa5 => return Some(ShredVariant::LegacyData),
        0x5a => return Some(ShredVariant::LegacyCoding),
        _ => {}
    }

    let proof_size = byte & 0x0F;

    match byte & 0xF0 {
        0x40 => Some(ShredVariant::MerkleCoding {
            proof_size,
            chained: false,
            resigned: false,
        }),
        0x60 => Some(ShredVariant::MerkleCoding {
            proof_size,
            chained: true,
            resigned: false,
        }),
        0x70 => Some(ShredVariant::MerkleCoding {
            proof_size,
            chained: true,
            resigned: true,
        }),
        0x80 => Some(ShredVariant::MerkleData {
            proof_size,
            chained: false,
            resigned: false,
        }),
        0x90 => Some(ShredVariant::MerkleData {
            proof_size,
            chained: true,
            resigned: false,
        }),
        0xb0 => Some(ShredVariant::MerkleData {
            proof_size,
            chained: true,
            resigned: true,
        }),
        _ => None,
    }
}

pub fn parse_common_header(data: &[u8]) -> Option<CommonHeader> {
    if data.len() < COMMON_HEADER_SIZE {
        return None;
    }

    let mut signature = [0u8; SIGNATURE_SIZE];
    signature.copy_from_slice(&data[..SIGNATURE_SIZE]);

    let variant = parse_variant_byte(data[64])?;
    let slot = u64::from_le_bytes(data[65..73].try_into().ok()?);
    let index = u32::from_le_bytes(data[73..77].try_into().ok()?);
    let version = u16::from_le_bytes(data[77..79].try_into().ok()?);
    let fec_set_index = u32::from_le_bytes(data[79..83].try_into().ok()?);

    Some(CommonHeader {
        signature,
        variant,
        slot,
        index,
        version,
        fec_set_index,
    })
}
