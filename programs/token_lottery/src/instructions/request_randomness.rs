use crate::state::Lottery;
use anchor_lang::prelude::*;
use switchboard_on_demand::RandomnessRequest;

#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub lottery: Account<'info, Lottery>,
    #[account(mut)]
    pub vrf: Account<'info, RandomnessRequest>,
}

pub fn consume_randomess_handler(ctx: Context<ConsumeRandomness>) -> Result<()> {
    ctx.accounts.vrf.request_randomness()?;
    Ok(())
}
