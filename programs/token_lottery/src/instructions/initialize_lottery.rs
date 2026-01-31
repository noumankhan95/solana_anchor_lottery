use anchor_lang::{accounts::signer, prelude::*};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::state::Lottery;

#[derive(Accounts)]
#[instruction(lottery_id:u64)]
pub struct InitializeLottery<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init,payer=signer,space=8+Lottery::INIT_SPACE,seeds=[b"lottery",signer.key().as_ref(),lottery_id.to_le_bytes().as_ref()],bump)]
    pub lottery_account: Account<'info, Lottery>,
    pub prize_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        token::mint = prize_mint,
        token::authority = lottery_account
    )]
    pub vault: Account<'info, Token>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn init_lottery(
    ctx: Context<InitializeLottery>,
    lottery_id: u64,
    ticket_price: u64,
    total_tickets: u64,
) -> Result<()> {
    let lottery_account = &mut ctx.accounts.lottery_account;
    lottery_account.authority = ctx.accounts.signer.key();
    lottery_account.ticket_price = ticket_price;
    lottery_account.lottery_id = lottery_id;
    lottery_account.is_active = true;
    lottery_account.bump = ctx.bumps.lottery_account;
    lottery_account.prize_mint = ctx.accounts.prize_mint.key();
    lottery_account.randomness = None;
    lottery_account.vault = ctx.accounts.vault.key();
    lottery_account.winner_mint = None;
    lottery_account.total_tickets = total_tickets;
    Ok(())
}
