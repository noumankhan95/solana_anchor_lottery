use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::error::Errors;
use crate::state::Lottery;
#[derive(Accounts)]
pub struct ClaimWinner<'info> {
    #[account(mut)]
    pub winner: Signer<'info>,
    #[account(mut,seeds=[b"lottery",lottery.authority.key().as_ref(),lottery.lottery_id.to_le_bytes().as_ref()],bump=lottery.bump)]
    pub lottery: Account<'info, Lottery>,
    #[account(mut,constraint=vault.key()==lottery.vault)]
    pub vault: Account<'info, Token>,
    #[account(mut,constraint=winnings_mint.key()==lottery.winner_mint.unwrap())]
    pub winnings_mint: Account<'info, Mint>,

    #[account(init_if_needed,payer=winner,associated_token::mint=winnings_mint,associated_token::authority=winner)]
    pub winner_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn claim_winner(ctx: Context<ClaimWinner>) -> Result<()> {
    let lottery_acc = &ctx.accounts.lottery;
    require!(lottery_acc.winner_mint.is_some(), Errors::WinnerNotChosen);
    let transfer_acc = Transfer {
        authority: ctx.accounts.lottery.to_account_info(),
        from: ctx.accounts.lottery_ata.to_account_info(),
        to: ctx.accounts.winner_ata.to_account_info(),
    };
    let transfer_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_acc);
    token::transfer(transfer_ctx, amount);
    lottery_acc.is_active = false;
    Ok(())
}
