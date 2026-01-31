use anchor_lang::prelude::*;

use crate::state::Lottery;

#[derive(Accounts)]
pub struct CloseLottery<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut,constraint=lottery.authority==signer.key(),close=signer)]
    pub lottery: Account<'info, Lottery>,
    pub system_program: Program<'info, System>,
}

pub fn close_lottery_handler(ctx: Context<CloseLottery>) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    lottery.is_active = false;
    Ok(())
}
