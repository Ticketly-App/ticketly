use anchor_lang::prelude::*;

#[error_code]
pub enum EventGateError {
    // Validation 
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
    #[msg("Ticket price must be greater than 0 lamports")]
    InvalidPrice,
    #[msg("Royalty basis-points must be ≤ 2 000 (20 %)")]
    InvalidRoyalty,
    #[msg("event_start must be in the future")]
    StartInPast,
    #[msg("event_end must be after event_start")]
    EndBeforeStart,

    // Event state 
    #[msg("Event is not active")]
    EventNotActive,
    #[msg("Event has already ended")]
    EventEnded,
    #[msg("Event has not started yet; check-in opens 1 hour before start")]
    CheckInTooEarly,
    #[msg("Event is already cancelled")]
    EventAlreadyCancelled,
    #[msg("Cannot cancel: check-ins have already begun")]
    CannotCancelAfterCheckIn,

    // Ticket state 
    #[msg("Tier is sold out")]
    TierSoldOut,
    #[msg("Invalid tier index")]
    InvalidTierIndex,
    #[msg("Ticket has already been checked in")]
    AlreadyCheckedIn,
    #[msg("Ticket does not belong to this event")]
    TicketEventMismatch,
    #[msg("Caller does not own this ticket")]
    NotTicketOwner,
    #[msg("Ticket is currently listed for resale")]
    TicketIsListed,
    #[msg("Ticket is not listed for resale")]
    TicketNotListed,

    // Token / mint 
    #[msg("Token mint does not match ticket record")]
    MintMismatch,
    #[msg("Token account has zero balance")]
    ZeroBalance,
    #[msg("Token account authority mismatch")]
    TokenOwnerMismatch,

    // Marketplace
    #[msg("Resale is not enabled for this event")]
    ResaleNotAllowed,
    #[msg("Listing price exceeds the event resale price cap")]
    PriceExceedsCap,
    #[msg("Buyer is the current owner of the ticket")]
    BuyerIsOwner,

    //Access control 
    #[msg("Signer is not the event authority")]
    NotEventAuthority,
    #[msg("Signer is not an authorised gate operator")]
    NotGateOperator,
    #[msg("Gate operator is already registered")]
    OperatorAlreadyAdded,
    #[msg("Gate operator not found in list")]
    OperatorNotFound,
    #[msg("Too many gate operators (max 10)")]
    TooManyOperators,

    // Arithmetic 
    #[msg("Arithmetic overflow / underflow")]
    Overflow,
}