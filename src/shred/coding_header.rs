use super::common_header::COMMON_HEADER_SIZE;

// Coding-shred-specific header, immediately after the common header.
//
// Layout (6 bytes):
//   [0..2]  num_data_shreds   -number of data shreds in this FEC set (u16 LE)
//   [2..4]  num_coding_shreds -number of coding shreds in this FEC set (u16 LE)
//   [4..6]  position          -this shard's position in the coding set (u16 LE)
pub const CODING_HEADER_SIZE: usize = 6;
pub const CODING_HEADER_OFFSET: usize = COMMON_HEADER_SIZE;
pub const CODING_PAYLOAD_OFFSET: usize = COMMON_HEADER_SIZE + CODING_HEADER_SIZE; // byte 89

#[derive(Debug, Clone)]
pub struct CodingHeader {
    pub num_data_shreds: u16,
    pub num_coding_shreds: u16,
    pub position: u16,
}

pub fn parse_coding_header(data: &[u8]) -> Option<CodingHeader> {
    if data.len() < CODING_HEADER_OFFSET + CODING_HEADER_SIZE {
        return None;
    }

    let offset = CODING_HEADER_OFFSET;
    let num_data_shreds = u16::from_le_bytes(data[offset..offset + 2].try_into().ok()?);
    let num_coding_shreds = u16::from_le_bytes(data[offset + 2..offset + 4].try_into().ok()?);
    let position = u16::from_le_bytes(data[offset + 4..offset + 6].try_into().ok()?);

    Some(CodingHeader {
        num_data_shreds,
        num_coding_shreds,
        position,
    })
}
