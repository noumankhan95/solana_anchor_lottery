use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Lottery {
    pub authority: Pubkey,
    pub ticket_price: u64,
    pub lottery_id: u64,
    pub is_active: bool,
}
