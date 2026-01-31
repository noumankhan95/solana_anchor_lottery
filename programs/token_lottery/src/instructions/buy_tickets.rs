use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
        CreateMetadataAccountsV3,
    },
    token::{self, Mint, TokenAccount},
};
use mpl_token_metadata::types::DataV2;

use crate::state::Lottery;

#[derive(Accounts)]
pub struct BuyTickets<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut)]
    pub lottery: Account<'info, Lottery>,
    #[account(init,payer=buyer,mint::decimals=0,mint::authority=lottery)]
    pub ticket_mint: Account<'info, Mint>,
    #[account(init,payer=buyer,associated_token::mint=ticket_mint,associated_token::authority=buyer)]
    pub associated_token_mint: Account<'info, TokenAccount>,
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub assoiciated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn buy_tokens(ctx: Context<BuyTickets>) -> Result<()> {
    let lottery = &ctx.accounts.lottery;
    token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                authority: lottery.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                to: ctx.accounts.associated_token_mint.to_account_info(),
            },
        ),
        amount,
    );
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
        ),
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
        ),
        Some(1),
    )?;
    lottery.tickets_mints.push(ctx.accounts.ticket_mint.key());
    Ok(())
}
