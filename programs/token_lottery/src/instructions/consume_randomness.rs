use anchor_lang::prelude::*;

use crate::error::Errors;
use crate::state::Lottery;
#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut,seeds=[b"lottery",signer.key().as_ref(),lottery.lottery_id.to_le_bytes().as_ref()],bump=lottery.bump)]
    pub lottery: Account<'info, Lottery>,
    #[account(
        constraint = vrf.key() == lottery.vrf.unwrap()
    )]
    pub vrf: Account<'info, RandomnessRequest>,
}

pub fn consume_randomness_handler(ctx: Context<ConsumeRandomness>) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    let vrf = &ctx.accounts.vrf;

    require!(lottery.is_active, Errors::LotteryClosed);
    require!(lottery.randomness.is_none(), Errors::AlreadyDrawn);

    let randomness = vrf.get_randomness()?; // VERIFIED randomness
    let bytes = randomness.to_bytes();

    let idx = (u64::from_le_bytes(bytes[..8].try_into().unwrap())
        % lottery.tickets_mints.len() as u64) as usize;

    lottery.randomness = Some(bytes);
    lottery.winner_mint = Some(lottery.tickets_mints[idx]);

    Ok(())
}
