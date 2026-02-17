use anchor_lang::prelude::*;

#[error_code]
pub enum EventGateError {
    #[msg("Event name exceeds 50 characters")]
    NameTooLong,
    #[msg("Description exceeds 200 characters")]
    DescriptionTooLong,
    #[msg("Venue string exceeds 100 characters")]
    VenueTooLong,
    #[msg("Metadata URI exceeds 200 characters")]
    UriTooLong,
    #[msg("Symbol exceeds 10 characters")]
    SymbolTooLong,
    #[msg("Must define 1-5 ticket tiers")]
    InvalidTierCount,
    #[msg("Tier supply must be at least 1")]
    InvalidSupply,
    #[msg("Ticket price must be ≥ 0 lamports")]
    InvalidPrice,
    #[msg("Royalty basis-points must be ≤ 2 000 (20 %)")]
    InvalidRoyalty,
    #[msg("event_start must be in the future")]
    StartInPast,
    #[msg("event_end must be after event_start")]
    EndBeforeStart,

    #[msg("Event is not active")]
    EventNotActive,
    #[msg("Event has already ended")]
    EventEnded,
    #[msg("Check-in opens 1 hour before event start")]
    CheckInTooEarly,
    #[msg("Event is already cancelled")]
    EventAlreadyCancelled,
    #[msg("Cannot cancel: at least one attendee has already checked in")]
    CannotCancelAfterCheckIn,

    #[msg("This tier is sold out")]
    TierSoldOut,
    #[msg("Tier index out of bounds")]
    InvalidTierIndex,
    #[msg("This tier is not currently on sale")]
    TierNotOnSale,
    #[msg("Tier sale has not started yet")]
    TierSaleNotStarted,
    #[msg("Tier sale window has ended")]
    TierSaleEnded,

    #[msg("Ticket has already been checked in")]
    AlreadyCheckedIn,
    #[msg("Ticket has not been checked in yet")]
    NotCheckedIn,
    #[msg("Ticket does not belong to this event")]
    TicketEventMismatch,
    #[msg("Caller does not own this ticket")]
    NotTicketOwner,
    #[msg("Ticket is currently listed for resale — cancel listing first")]
    TicketIsListed,
    #[msg("Ticket is not listed for resale")]
    TicketNotListed,

    #[msg("Token mint does not match ticket record")]
    MintMismatch,
    #[msg("Token account has zero balance")]
    ZeroBalance,
    #[msg("Token account authority mismatch")]
    TokenOwnerMismatch,

    #[msg("Resale is not enabled for this event")]
    ResaleNotAllowed,
    #[msg("Listing price exceeds the event resale price cap")]
    PriceExceedsCap,
    #[msg("Buyer is already the owner of this ticket")]
    BuyerIsOwner,

    #[msg("POAP minting is not enabled for this event")]
    PoapNotEnabled,
    #[msg("A POAP has already been minted for this ticket")]
    PoapAlreadyMinted,

    #[msg("This event requires a whitelist entry to purchase")]
    WhitelistEntryRequired,
    #[msg("Whitelist entry belongs to a different event")]
    WhitelistEventMismatch,
    #[msg("Caller is not on the whitelist for this event")]
    NotWhitelisted,
    #[msg("Whitelist allocation fully used")]
    AllocationExhausted,
    #[msg("Whitelist gating is not enabled for this event")]
    WhitelistNotEnabled,

    #[msg("Signer is not the event authority")]
    NotEventAuthority,
    #[msg("Signer is not an authorised gate operator")]
    NotGateOperator,
    #[msg("Gate operator is already registered")]
    OperatorAlreadyAdded,
    #[msg("Gate operator not found")]
    OperatorNotFound,
    #[msg("Max gate operators reached (10)")]
    TooManyOperators,

    #[msg("No withdrawable balance in event PDA")]
    NothingToWithdraw,
    #[msg("Requested withdrawal exceeds available balance")]
    InsufficientFunds,

    #[msg("Arithmetic overflow or underflow")]
    Overflow,
}