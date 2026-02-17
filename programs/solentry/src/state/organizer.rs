use anchor_lang::prelude::*;
use crate::constants::*;


#[account]
pub struct OrganizerProfile {
    pub authority:         Pubkey,
    pub name:              String,   // ≤ 50 bytes
    pub website:           String,   // ≤ 100 bytes
    pub logo_uri:          String,   // ≤ 200 bytes
    pub total_events:      u32,
    pub total_tickets:     u64,
    pub total_revenue:     u64,
    pub total_royalties:   u64,
    pub is_verified:       bool,
    pub created_at:        i64,
    pub bump:              u8,
}

impl OrganizerProfile {
    pub const LEN: usize = DISCRIMINATOR
        + PUBKEY               // authority
        + string(MAX_NAME_LEN) // name
        + string(100)          // website
        + string(MAX_URI_LEN)  // logo_uri
        + U32                  // total_events
        + U64                  // total_tickets
        + U64                  // total_revenue
        + U64                  // total_royalties
        + BOOL                 // is_verified
        + I64                  // created_at
        + U8;                  // bump
}

#[account]
pub struct PlatformConfig {
    pub admin:              Pubkey,
    pub protocol_fee_bps:   u16,
    pub fee_receiver:       Pubkey,
    pub creation_paused:    bool,
    pub bump:               u8,
}

impl PlatformConfig {
    pub const LEN: usize = DISCRIMINATOR
        + PUBKEY + U16 + PUBKEY + BOOL + U8;
}