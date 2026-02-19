use anchor_lang::prelude::*;


#[event]
pub struct EventCreated {
    pub authority:   Pubkey,
    pub event_pda:   Pubkey,
    pub event_id:    u64,
    pub name:        String,
    pub event_start: i64,
    pub event_end:   i64,
}

#[event]
pub struct EventUpdated {
    pub event_pda: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EventCancelled {
    pub event_pda: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}


#[event]
pub struct TicketMinted {
    pub event_pda:     Pubkey,
    pub ticket_pda:    Pubkey,
    pub mint:          Pubkey,
    pub owner:         Pubkey,
    pub ticket_number: u64,
    pub tier_index:    u8,
    pub paid_lamports: u64,
    pub timestamp:     i64,
}

#[event]
pub struct TicketCheckedIn {
    pub event_pda:     Pubkey,
    pub ticket_pda:    Pubkey,
    pub attendee:      Pubkey,
    pub operator:      Pubkey,
    pub ticket_number: u64,
    pub timestamp:     i64,
}

#[event]
pub struct TicketTransferred {
    pub event_pda:      Pubkey,
    pub ticket_pda:     Pubkey,
    pub from:           Pubkey,
    pub to:             Pubkey,
    pub ticket_number:  u64,
    pub transfer_count: u8,
    pub timestamp:      i64,
}


#[event]
pub struct TicketListed {
    pub event_pda:  Pubkey,
    pub ticket_pda: Pubkey,
    pub seller:     Pubkey,
    pub price:      u64,
    pub timestamp:  i64,
}

#[event]
pub struct TicketSold {
    pub event_pda:        Pubkey,
    pub ticket_pda:       Pubkey,
    pub seller:           Pubkey,
    pub buyer:            Pubkey,
    pub price:            u64,
    pub royalty_lamports: u64,
    pub resale_count:     u8,
    pub timestamp:        i64,
}

#[event]
pub struct ListingCancelled {
    pub event_pda:  Pubkey,
    pub ticket_pda: Pubkey,
    pub seller:     Pubkey,
    pub timestamp:  i64,
}


#[event]
pub struct OperatorAdded {
    pub event_pda: Pubkey,
    pub operator:  Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct OperatorRemoved {
    pub event_pda: Pubkey,
    pub operator:  Pubkey,
    pub timestamp: i64,
}


#[event]
pub struct PoapMinted {
    pub event_pda:      Pubkey,
    pub ticket_pda:     Pubkey,
    pub poap_pda:       Pubkey,
    pub holder:         Pubkey,
    pub edition_number: u64,
    pub timestamp:      i64,
}


#[event]
pub struct WhitelistEntryAdded {
    pub event_pda:  Pubkey,
    pub wallet:     Pubkey,
    pub allocation: u8,
    pub timestamp:  i64,
}

#[event]
pub struct WhitelistEntryRemoved {
    pub event_pda: Pubkey,
    pub wallet:    Pubkey,
    pub timestamp: i64,
}


#[event]
pub struct RevenueWithdrawn {
    pub event_pda: Pubkey,
    pub authority: Pubkey,
    pub amount:    u64,
    pub timestamp: i64,
}


#[event]
pub struct OrganizerVerified {
    pub organizer_pda: Pubkey,
    pub authority:     Pubkey,
    pub timestamp:     i64,
}