use anchor_lang::prelude::*;
use crate::constants::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TierType {
    GeneralAdmission = 0,
    EarlyBird        = 1,
    Vip              = 2,
    Vvip             = 3,
    Custom           = 4,
}

impl Default for TierType {
    fn default() -> Self { TierType::GeneralAdmission }
}

impl TierType {
    pub fn label(&self) -> &'static str {
        match self {
            TierType::GeneralAdmission => "GA",
            TierType::EarlyBird        => "Early Bird",
            TierType::Vip              => "VIP",
            TierType::Vvip             => "VVIP",
            TierType::Custom           => "Custom",
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct GpsCoords {
    pub lat_micro: i32,
    pub lng_micro: i32,
}

impl GpsCoords {
    pub const LEN: usize = 4 + 4; // i32 + i32
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct TicketTier {
    pub tier_type:      TierType,
    pub price:          u64,
    pub supply:         u32,
    pub minted:         u32,
    pub checked_in:     u32,
    pub is_on_sale:     bool,
    pub sale_start:     i64,
    pub sale_end:       i64,
}

impl TicketTier {
    /// 1 + 8 + 4 + 4 + 4 + 1 + 8 + 8 = 38
    pub const LEN: usize = U8 + U64 + U32 + U32 + U32 + BOOL + I64 + I64;
}


#[account]
pub struct EventAccount {

    pub authority:          Pubkey,
    pub event_id:           u64,

    pub name:               String,    // ≤ 50 bytes
    pub description:        String,    // ≤ 200 bytes
    pub venue:              String,    // ≤ 100 bytes
    pub metadata_uri:       String,    // ≤ 200 bytes
    pub symbol:             String,    // ≤ 10 bytes
    pub gps:                GpsCoords,

    pub event_start:        i64,
    pub event_end:          i64,
    pub created_at:         i64,

    pub ticket_tiers:       Vec<TicketTier>,
    pub total_minted:       u64,
    pub total_checked_in:   u64,
    pub total_revenue:      u64,

    pub resale_allowed:     bool,
    pub max_resale_price:   Option<u64>,
    pub royalty_bps:        u16,
    pub royalty_receiver:   Pubkey,
    pub total_royalties:    u64,

    pub gate_operators:     Vec<Pubkey>,
    pub whitelist_gated:    bool,


    pub poap_enabled:       bool,
    pub poap_metadata_uri:  String,    // ≤ 200 bytes
    pub total_poaps_minted: u64,

    pub is_active:          bool,
    pub is_cancelled:       bool,

    pub bump:               u8,
}

impl EventAccount {
    pub const LEN: usize = DISCRIMINATOR
        + PUBKEY                                        // authority
        + U64                                           // event_id
        + string(MAX_NAME_LEN)                          // name
        + string(MAX_DESC_LEN)                          // description
        + string(MAX_VENUE_LEN)                         // venue
        + string(MAX_URI_LEN)                           // metadata_uri
        + string(MAX_SYMBOL_LEN)                        // symbol
        + GpsCoords::LEN                                // gps
        + I64 + I64 + I64                               // event_start, event_end, created_at
        + vec_of(MAX_TIERS as usize, TicketTier::LEN)   // ticket_tiers
        + U64 + U64 + U64                               // total_minted, total_checked_in, total_revenue
        + BOOL                                          // resale_allowed
        + option(U64)                                   // max_resale_price
        + U16                                           // royalty_bps
        + PUBKEY                                        // royalty_receiver
        + U64                                           // total_royalties
        + vec_of(10, PUBKEY)                            // gate_operators
        + BOOL                                          // whitelist_gated
        + BOOL                                          // poap_enabled
        + string(MAX_URI_LEN)                           // poap_metadata_uri
        + U64                                           // total_poaps_minted
        + BOOL + BOOL                                   // is_active, is_cancelled
        + U8;                                           // bump
}