use crate::state::Lottery;
use anchor_lang::prelude::*;
use switchboard_on_demand::RandomnessRequest;

#[derive(Accounts)]
pub struct RequestRandomness<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        constraint = signer.key() == lottery.authority
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(
        init,
        payer = signer,
        space = 8 + RandomnessRequest::SIZE,
        owner = switchboard_on_demand::ID
    )]
    pub vrf: Account<'info, RandomnessRequest>,
    pub system_program: Program<'info, System>,
}

pub fn consume_randomess_handler(ctx: Context<RequestRandomness>) -> Result<()> {
    let vrf = &mut ctx.accounts.vrf;

    vrf.request_randomness()?;

    ctx.accounts.lottery.vrf = Some(vrf.key());
    Ok(())
}
