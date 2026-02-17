/// PDA seed prefixes
pub const SEED_EVENT:   &[u8] = b"event";
pub const SEED_TICKET:  &[u8] = b"ticket";
pub const SEED_MINT:    &[u8] = b"ticket_mint";
pub const SEED_ESCROW:  &[u8] = b"escrow";

pub const MAX_NAME_LEN:   usize = 50;
pub const MAX_DESC_LEN:   usize = 200;
pub const MAX_VENUE_LEN:  usize = 100;
pub const MAX_URI_LEN:    usize = 200;
pub const MAX_SYMBOL_LEN: usize = 10;

pub const MAX_TIERS:         u8  = 5;
pub const MAX_ROYALTY_BPS:   u16 = 2_000; // 20 % hard cap
pub const CHECK_IN_GRACE_S:  i64 = 3_600; // open gates 1 h early
pub const MAX_RESALE_MARKUP: u16 = 10_000; // 100 % above face value allowed by default


pub const DISCRIMINATOR: usize = 8;
pub const PUBKEY: usize = 32;
pub const U8:  usize = 1;
pub const U16: usize = 2;
pub const U32: usize = 4;
pub const U64: usize = 8;
pub const I64: usize = 8;
pub const BOOL: usize = 1;
pub const fn option(inner: usize) -> usize { 1 + inner }
pub const fn vec_of(n: usize, inner: usize) -> usize { 4 + n * inner }
pub const fn string(max: usize) -> usize { 4 + max }