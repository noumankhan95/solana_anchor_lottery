use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct BuyTickets<'info> {
    pub buyer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn buy_tokens(ctx: Context<BuyTickets>) -> Result<()> {
    Ok(())
}
