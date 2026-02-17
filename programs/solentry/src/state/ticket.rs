use anchor_lang::prelude::*;
use crate::{
    constants::*,
    state::event::TierType,
};

#[account]
pub struct TicketAccount {

    pub event:              Pubkey,
    pub mint:               Pubkey,
    pub owner:              Pubkey,
    pub original_buyer:     Pubkey,


    pub ticket_number:      u64,
    pub tier_index:         u8,
    pub tier_type:          TierType,
    pub price_paid:         u64,


    pub metadata_uri:       String,    // ≤ 200 bytes

    pub is_checked_in:      bool,
    pub checked_in_at:      Option<i64>,
    pub checked_in_by:      Option<Pubkey>,
    pub poap_minted:        bool,

    pub is_listed:          bool,
    pub listed_price:       Option<u64>,
    pub resale_count:       u8,
    pub transfer_count:     u8,

    pub minted_at:          i64,
    pub last_transferred_at: Option<i64>,

    pub bump:               u8,
}

impl TicketAccount {
    pub const LEN: usize = DISCRIMINATOR
        + PUBKEY + PUBKEY + PUBKEY + PUBKEY    // event, mint, owner, original_buyer
        + U64                                  // ticket_number
        + U8 + U8                              // tier_index, tier_type
        + U64                                  // price_paid
        + string(MAX_URI_LEN)                  // metadata_uri
        + BOOL                                 // is_checked_in
        + option(I64)                          // checked_in_at
        + option(PUBKEY)                       // checked_in_by
        + BOOL                                 // poap_minted
        + BOOL                                 // is_listed
        + option(U64)                          // listed_price
        + U8 + U8                              // resale_count, transfer_count
        + I64                                  // minted_at
        + option(I64)                          // last_transferred_at
        + U8;                                  // bump
}


#[account]
pub struct ListingAccount {
    pub event:      Pubkey,
    pub ticket:     Pubkey,
    pub seller:     Pubkey,
    pub escrow_ata: Pubkey,
    pub price:      u64,
    pub listed_at:  i64,
    pub bump:       u8,
}

impl ListingAccount {
    /// 8 + 32*4 + 8 + 8 + 1 = 153
    pub const LEN: usize = DISCRIMINATOR
        + PUBKEY + PUBKEY + PUBKEY + PUBKEY
        + U64 + I64 + U8;
}