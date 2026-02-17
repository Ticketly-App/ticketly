use anchor_lang::prelude::*;
use crate::constants::*;


#[account]
pub struct WhitelistEntry {
    pub event:         Pubkey,
    pub wallet:        Pubkey,
    pub allocation:    u8,
    pub purchased:     u8,
    pub added_at:      i64,
    pub bump:          u8,
}

impl WhitelistEntry {
    /// 8 + 32 + 32 + 1 + 1 + 8 + 1 = 83
    pub const LEN: usize = DISCRIMINATOR
        + PUBKEY + PUBKEY + U8 + U8 + I64 + U8;
}