use anchor_lang::prelude::*;

declare_id!("3FWjsmMG13BLm5KQQsjZ9jozfSQC91gntgDuAMfNnkbJ");

#[program]
pub mod solentry {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
