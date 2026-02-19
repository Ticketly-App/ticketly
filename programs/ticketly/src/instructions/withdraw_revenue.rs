use anchor_lang::prelude::*;
use crate::{
    errors::EventGateError,
    events::RevenueWithdrawn,
    state::event::EventAccount,
    constants::*,
};

pub fn handler(ctx: Context<WithdrawRevenue>, amount: Option<u64>) -> Result<()> {
    let clock = Clock::get()?;

    let event_info  = ctx.accounts.event.to_account_info();
    let rent        = Rent::get()?;
    let min_balance = rent.minimum_balance(EventAccount::LEN);
    let balance     = event_info.lamports();

    require!(balance > min_balance, EventGateError::NothingToWithdraw);

    let withdrawable = balance.checked_sub(min_balance)
        .ok_or(EventGateError::Overflow)?;

    let withdraw_amount = match amount {
        Some(a) => {
            require!(a <= withdrawable, EventGateError::InsufficientFunds);
            a
        }
        None => withdrawable,
    };

    **event_info.try_borrow_mut_lamports()? -= withdraw_amount;
    **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += withdraw_amount;

    emit!(RevenueWithdrawn {
        event_pda:  ctx.accounts.event.key(),
        authority:  ctx.accounts.authority.key(),
        amount:     withdraw_amount,
        timestamp:  clock.unix_timestamp,
    });

    msg!("RevenueWithdrawn amount={}", withdraw_amount);
    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawRevenue<'info> {
    #[account(
        mut,
        seeds = [SEED_EVENT, event.authority.as_ref(), &event.event_id.to_le_bytes()],
        bump  = event.bump,
        has_one = authority @ EventGateError::NotEventAuthority,
    )]
    pub event: Account<'info, EventAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,
}