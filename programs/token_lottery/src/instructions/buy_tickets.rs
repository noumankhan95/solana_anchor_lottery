use crate::error::Errors;
use crate::state::Lottery;

use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3,
        set_and_verify_sized_collection_item, CreateMasterEditionV3, CreateMetadataAccountsV3,
        SetAndVerifySizedCollectionItem,
    },
    token::{mint_to, Mint, Token, TokenAccount},
};

use mpl_token_metadata::types::DataV2;

#[derive(Accounts)]
pub struct BuyTickets<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

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
        init,
        payer = buyer,
        seeds = [
            b"ticket_mint",
            lottery.key().as_ref(),
            &lottery.ticket_count.to_le_bytes()
        ],
        bump,
        mint::decimals = 0,
        mint::authority = lottery,
        mint::freeze_authority = lottery
    )]
    pub ticket_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = buyer,
        associated_token::mint = ticket_mint,
        associated_token::authority = buyer
    )]
    pub ticket_ata: Account<'info, TokenAccount>,

    /// CHECK: Metaplex PDA
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
    pub ticket_metadata: UncheckedAccount<'info>,

    /// CHECK: Metaplex PDA
    #[account(
        mut,
        seeds = [
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            ticket_mint.key().as_ref(),
            b"edition"
        ],
        bump,
        seeds::program = mpl_token_metadata::ID
    )]
    pub ticket_master_edition: UncheckedAccount<'info>,

    #[account(
        seeds = [b"collection_mint", lottery.lottery_id.to_le_bytes().as_ref()],
        bump = lottery.collection_bump
    )]
    pub collection_mint: Account<'info, Mint>,

    /// CHECK: Collection metadata PDA
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Collection master edition PDA
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn buy_tokens(ctx: Context<BuyTickets>) -> Result<()> {
    let lottery = &mut ctx.accounts.lottery;

    require!(lottery.is_active, Errors::LotteryClosed);
    require!(
        lottery.ticket_count < lottery.total_tickets,
        Errors::LimitExceeded
    );

    // 1️⃣ Transfer SOL for ticket price
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.lottery.to_account_info(),
            },
        ),
        lottery.ticket_price,
    )?;

    lottery.pot += lottery.ticket_price;

    // PDA signer for mint authority
    let lottery_seeds: &[&[&[u8]]] = &[&[
        b"lottery",
        lottery.authority.as_ref(),
        &lottery.lottery_id.to_le_bytes(),
        &[lottery.bump],
    ]];

    // 2️⃣ Mint ticket NFT
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.ticket_mint.to_account_info(),
                to: ctx.accounts.ticket_ata.to_account_info(),
                authority: ctx.accounts.lottery.to_account_info(),
            },
            lottery_seeds,
        ),
        1,
    )?;

    // 3️⃣ Create ticket metadata
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.ticket_metadata.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                mint_authority: ctx.accounts.lottery.to_account_info(),
                update_authority: ctx.accounts.lottery.to_account_info(),
                payer: ctx.accounts.buyer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            lottery_seeds,
        ),
        DataV2 {
            name: format!("Ticket #{}", lottery.ticket_count),
            symbol: "TICKET".to_string(),
            uri: "https://example.com/ticket.json".to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        true,
        true,
        None,
    )?;

    // 4️⃣ Create master edition (NFT = 1/1)
    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.ticket_master_edition.to_account_info(),
                metadata: ctx.accounts.ticket_metadata.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                mint_authority: ctx.accounts.lottery.to_account_info(),
                update_authority: ctx.accounts.lottery.to_account_info(),
                payer: ctx.accounts.buyer.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            lottery_seeds,
        ),
        Some(1),
    )?;

    // 5️⃣ Verify ticket belongs to collection
    set_and_verify_sized_collection_item(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            SetAndVerifySizedCollectionItem {
                metadata: ctx.accounts.ticket_metadata.to_account_info(),
                collection_authority: ctx.accounts.lottery.to_account_info(),
                payer: ctx.accounts.buyer.to_account_info(),
                update_authority: ctx.accounts.lottery.to_account_info(),
                collection_mint: ctx.accounts.collection_mint.to_account_info(),
                collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
                collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
            },
            lottery_seeds,
        ),
        None,
    )?;

    lottery.ticket_count += 1;
    Ok(())
}
