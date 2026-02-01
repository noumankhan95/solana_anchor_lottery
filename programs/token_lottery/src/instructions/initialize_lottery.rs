use anchor_lang::{accounts::signer, prelude::*};
use anchor_spl::token::{Mint, TokenAccount, TokenInterface};

use crate::state::Lottery;
use anchor_spl::metadata::{
    create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
    CreateMetadataAccountsV3,
};
use mpl_token_metadata::types::{CollectionDetails, DataV2};
#[derive(Accounts)]
#[instruction(lottery_id:u64)]
pub struct InitializeLottery<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init,payer=signer,space=8+Lottery::INIT_SPACE,seeds=[b"lottery",signer.key().as_ref(),lottery_id.to_le_bytes().as_ref()],bump)]
    pub lottery_account: Account<'info, Lottery>,

    #[account(init,
        payer=signer,
        seeds=[b"collection_mint",lottery_id.to_le_bytes().as_ref()],bump,mint::decimals=0,mint::authority=lottery_account,mint::freeze_authority=lottery_account
    )]
    pub collection_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = collection_mint,
        associated_token::authority = signer,
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
    let lottery_seeds: &[&[&[u8]]] = &[&[
        b"lottery",
        ctx.accounts.signer.key.as_ref(),
        &lottery_id.to_le_bytes(),
        &[ctx.bumps.lottery_account],
    ]];

    // Mint 1 collection NFT
    anchor_spl::token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.collection_mint.to_account_info(),
                to: ctx.accounts.collection_ata.to_account_info(),
                authority: ctx.accounts.lottery_account.to_account_info(),
            },
            lottery_seeds,
        ),
        1,
    )?;

    // Create metadata
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.collection_metadata.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                mint_authority: ctx.accounts.lottery_account.to_account_info(),
                update_authority: ctx.accounts.lottery_account.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            lottery_seeds,
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

    // Create master edition
    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.collection_master_edition.to_account_info(),
                metadata: ctx.accounts.collection_metadata.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                mint_authority: ctx.accounts.lottery_account.to_account_info(),
                update_authority: ctx.accounts.lottery_account.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            lottery_seeds,
        ),
        Some(1),
    )?;

    // Store lottery state
    let lottery = &mut ctx.accounts.lottery_account;
    lottery.authority = ctx.accounts.signer.key();
    lottery.ticket_price = ticket_price;
    lottery.lottery_id = lottery_id;
    lottery.total_tickets = total_tickets;
    lottery.is_active = true;
    lottery.bump = ctx.bumps.lottery_account;
    lottery.collection_bump = ctx.bumps.collection_mint;
    lottery.randomness = None;
    lottery.winner_mint = None;
    lottery.ticket_count = 0;
    lottery.pot = 0;

    Ok(())
}
