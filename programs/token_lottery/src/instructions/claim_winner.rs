use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::error::Errors;
use crate::state::Lottery;

#[derive(Accounts)]
pub struct ClaimWinner<'info> {
    #[account(mut)]
    pub winner: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"lottery",
            lottery.authority.as_ref(),
            lottery.lottery_id.to_le_bytes().as_ref()
        ],
        bump = lottery.bump
    )]
    pub lottery: Account<'info, Lottery>,

    #[account(
        constraint = winner_ticket_ata.owner == winner.key(),
        constraint = winner_ticket_ata.amount == 1
    )]
    pub winner_ticket_ata: Account<'info, TokenAccount>,

    pub ticket_mint: Account<'info, Mint>,

    /// CHECK: Metaplex metadata PDA
    #[account(
        seeds = [
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            ticket_mint.key().as_ref()
        ],
        bump,
        seeds::program = mpl_token_metadata::ID
    )]
    pub ticket_metadata: UncheckedAccount<'info>,

    #[account(
        seeds = [b"collection_mint", lottery.lottery_id.to_le_bytes().as_ref()],
        bump = lottery.collection_bump
    )]
    pub collection_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
}

pub fn claim_winner(ctx: Context<ClaimWinner>) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;

    require!(lottery.is_active, Errors::LotteryClosed);

    let winner_mint = lottery.winner_mint.ok_or(Errors::WinnerNotChosen)?;

    // Ensure the provided ticket is the winning one
    require!(
        ctx.accounts.winner_ticket_ata.mint == winner_mint,
        Errors::InvalidWinningTicket
    );

    // Deserialize metadata safely
    let metadata =
        mpl_token_metadata::accounts::Metadata::from_account_info(&ctx.accounts.ticket_metadata)?;

    let collection = metadata.collection.ok_or(Errors::InvalidTicket)?;

    require!(collection.verified, Errors::InvalidTicket);
    require!(
        collection.key == ctx.accounts.collection_mint.key(),
        Errors::InvalidTicket
    );

    // Transfer SOL prize
    let prize = lottery.pot;
    require!(prize > 0, Errors::NoPrizeAvailable);

    **lottery.to_account_info().try_borrow_mut_lamports()? -= prize;
    **ctx
        .accounts
        .winner
        .to_account_info()
        .try_borrow_mut_lamports()? += prize;

    lottery.pot = 0;
    lottery.is_active = false;

    Ok(())
}
