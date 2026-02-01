use anchor_lang::{accounts::signer, prelude::*};
use anchor_spl::{
    metadata::create_metadata_accounts_v3,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

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
    pub vault: Account<'info, TokenAccount>,
    #[account(init,
        payer=signer,
        seeds=[b"collection_mint",lottery_id.to_le_bytes.as_ref()],bump,mint::decimals=0,mint::authority=collection_mint,mint::freeze_authority=collection_mint
    )]
    pub collection_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = collection_mint,
        associated_token::authority = payer,
    )]
    pub collection_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Created by Metaplex
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn init_lottery(
    ctx: Context<InitializeLottery>,
    lottery_id: u64,
    ticket_price: u64,
    total_tickets: u64,
) -> Result<()> {
    let seeds: &[&[&[u8]]] = &[
        b"collection_mint",
        &lottery_id.to_le_bytes,
        &[ctx.bumps.collection_mint],
    ];

    anchor_spl::token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                authority: ctx.accounts.collection_mint.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                to: ctx.accounts.collection_ata.to_account_info(),
            },
        )
        .with_signer(seeds),
        1,
    )?;
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.collection_metadata.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer,
        ),
        DataV2 {
            name: "Lottery Tickets".to_string(),
            symbol: "TICKET".to_string(),
            uri: "https://example.com/collection.json".to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        true,
        true,
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

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
