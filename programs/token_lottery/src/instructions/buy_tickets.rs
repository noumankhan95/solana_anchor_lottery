use crate::error::Errors;
use crate::state::Lottery;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
        CreateMetadataAccountsV3,
    },
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use mpl_token_metadata::types::DataV2;

#[derive(Accounts)]
pub struct BuyTickets<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(
  mut,
  constraint = buyer_ata.owner == buyer.key(),
  constraint = buyer_ata.mint == lottery.prize_mint
)]
    pub buyer_ata: Account<'info, TokenAccount>,
    #[account(mut,seeds=[b"lottery",lottery.authority.key().as_ref(),lottery.lottery_id.to_le_bytes().as_ref()],bump=lottery.bump)]
    pub lottery: Account<'info, Lottery>,
    #[account(mut,constraint=vault.key()==lottery.vault)]
    pub vault: Account<'info, TokenAccount>,
    #[account(init,payer=buyer,mint::decimals=0,mint::authority=lottery)]
    pub ticket_mint: Account<'info, Mint>,
    #[account(init,payer=buyer,associated_token::mint=ticket_mint,associated_token::authority=buyer)]
    pub associated_ticket_mint_account: Account<'info, TokenAccount>,
    #[account(
  mut,
  seeds = [
    b"metadata",
    mpl_token_metadata::ID.as_ref(),
    ticket_mint.key().as_ref()
  ],
  bump,
  seeds::program = mpl_token_metadata::ID
)]
    pub metadata: UncheckedAccount<'info>,
    #[account(mut,seeds = [
  b"metadata",
  mpl_token_metadata::ID.as_ref(),
  ticket_mint.key().as_ref(),
  b"edition"
],bump)]
    pub master_edition: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn buy_tokens(ctx: Context<BuyTickets>) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;
    require!(
        lottery.tickets_mints.len() <= (lottery.total_tickets) as usize,
        Errors::LimitExceeded
    );
    let transfer_acc = Transfer {
        from: ctx.accounts.buyer_ata.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let transfer_cpi = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_acc);
    token::transfer(transfer_cpi, lottery.ticket_price)?;
    require!(lottery.is_active, Errors::LotteryClosed);
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"lottery",
        lottery.authority.key().as_ref(),
        lottery.lottery_id.to_le_bytes().as_ref(),
        &[lottery.bump],
    ]];
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                authority: lottery.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                to: ctx
                    .accounts
                    .associated_ticket_mint_account
                    .to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        1,
    )?;
    let data = DataV2 {
        name: "Lottery Ticket".to_string(),
        symbol: "LOTTO".to_string(),
        uri: "https://example.com/ticket.json".to_string(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };
    create_metadata_accounts_v3(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                mint_authority: lottery.to_account_info(),
                payer: ctx.accounts.buyer.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                update_authority: lottery.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        data,
        true,
        true,
        None,
    )?;

    create_master_edition_v3(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.master_edition.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                mint_authority: lottery.to_account_info(),
                payer: ctx.accounts.buyer.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                update_authority: lottery.to_account_info(),
            },
        )
        .with_signer(signer_seeds),
        Some(1),
    )?;
    lottery.tickets_mints.push(ctx.accounts.ticket_mint.key());
    Ok(())
}
