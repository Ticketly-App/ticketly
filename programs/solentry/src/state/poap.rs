use anchor_lang::prelude::*;
use crate::constants::*;


#[account]
pub struct PoapRecord {
    pub ticket:          Pubkey,
    pub event:           Pubkey,
    pub mint:            Pubkey,
    pub holder:          Pubkey,
    pub event_name:      String,   // ≤ 50 bytes
    pub attended_at:     i64,
    pub edition_number:  u64,
    pub metadata_uri:    String,   // ≤ 200 bytes
    pub bump:            u8,
}

impl PoapRecord {
    pub const LEN: usize = DISCRIMINATOR
        + PUBKEY + PUBKEY + PUBKEY + PUBKEY  // ticket, event, mint, holder
        + string(MAX_NAME_LEN)               // event_name
        + I64                                // attended_at
        + U64                                // edition_number
        + string(MAX_URI_LEN)                // metadata_uri
        + U8;                                // bump
}