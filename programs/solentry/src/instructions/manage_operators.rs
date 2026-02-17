use anchor_lang::prelude::*;
use crate::{
    contexts::{AddOperator, RemoveOperator},
    errors::EventGateError,
    events::{OperatorAdded, OperatorRemoved},
};

const MAX_OPERATORS: usize = 10;

pub fn add_operator_handler(
    ctx: Context<AddOperator>,
    operator: Pubkey,
) -> Result<()> {
    let event = &mut ctx.accounts.event;
    let clock = Clock::get()?;

    require!(
        !event.gate_operators.contains(&operator),
        EventGateError::OperatorAlreadyAdded
    );
    require!(
        event.gate_operators.len() < MAX_OPERATORS,
        EventGateError::TooManyOperators
    );

    event.gate_operators.push(operator);

    emit!(OperatorAdded {
        event_pda: event.key(),
        operator,
        timestamp: clock.unix_timestamp,
    });

    msg!("OperatorAdded {}", operator);
    Ok(())
}

pub fn remove_operator_handler(
    ctx: Context<RemoveOperator>,
    operator: Pubkey,
) -> Result<()> {
    let event = &mut ctx.accounts.event;
    let clock = Clock::get()?;

    let idx = event
        .gate_operators
        .iter()
        .position(|k| *k == operator)
        .ok_or(EventGateError::OperatorNotFound)?;

    event.gate_operators.swap_remove(idx);

    emit!(OperatorRemoved {
        event_pda: event.key(),
        operator,
        timestamp: clock.unix_timestamp,
    });

    msg!("OperatorRemoved {}", operator);
    Ok(())
}