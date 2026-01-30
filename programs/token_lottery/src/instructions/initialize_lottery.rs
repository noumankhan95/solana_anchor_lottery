use anchor_lang::{accounts::signer, prelude::*};

use crate::state::Lottery;

#[derive(Accounts)]
#[instruction(lottery_id:u64)]
pub struct InitializeLottery<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init,payer=signer,space=8+Lottery::INIT_SPACE,seeds=[b"lottery",signer.key().as_ref(),lottery_id.to_le_bytes().as_ref()],bump)]
    pub lottery_account: Account<'info, Lottery>,
    pub system_program: Program<'info, System>,
}

pub fn init_lottery(
    ctx: Context<InitializeLottery>,
    lottery_id: u64,
    ticket_price: u64,
) -> Result<()> {
    let lottery_account = &mut ctx.accounts.lottery_account;
    lottery_account.authority = ctx.accounts.signer.key();
    lottery_account.ticket_price = ticket_price;
    lottery_account.lottery_id = lottery_id;
    lottery_account.is_active = true;
    Ok(())
}
