use super::coding_header::CODING_PAYLOAD_OFFSET;
use super::common_header::ShredVariant;
use super::data_header::{DataHeader, DATA_PAYLOAD_OFFSET};

pub fn extract_data_payload<'a>(
    raw: &'a [u8],
    header: &DataHeader,
    _variant: &ShredVariant,
) -> &'a [u8] {
    let start = DATA_PAYLOAD_OFFSET;
    let end = (header.size as usize).min(raw.len());
    if end <= start {
        return &[];
    }
    &raw[start..end]
}

pub fn extract_coding_payload<'a>(raw: &'a [u8], variant: &ShredVariant) -> &'a [u8] {
    if raw.len() <= CODING_PAYLOAD_OFFSET {
        return &[];
    }
    let end = raw.len().saturating_sub(variant.merkle_proof_bytes());
    if end <= CODING_PAYLOAD_OFFSET {
        return &[];
    }
    &raw[CODING_PAYLOAD_OFFSET..end]
}
