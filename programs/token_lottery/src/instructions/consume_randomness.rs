use anchor_lang::prelude::*;

use crate::error::Errors;
use crate::state::Lottery;
#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut,seeds=[b"lottery",signer.key().as_ref(),lottery.lottery_id.to_le_bytes().as_ref()],bump)]
    pub lottery: Account<'info, Lottery>,
}

pub fn consume_randomness_handler(
    ctx: Context<ConsumeRandomness>,
    randomness: [u8; 32],
) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    require!(lottery.is_active, Errors::LotteryClosed);
    let idx = ((u64::from_le_bytes(randomness[..8].try_into().unwrap()))
        % lottery.tickets_mints.len() as u64) as usize;
    lottery.randomness = Some(randomness);
    lottery.winner_mint = Some(lottery.tickets_mints.get(idx).unwrap().clone());
    Ok(())
}
