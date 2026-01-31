use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Lottery {
    pub authority: Pubkey,
    pub ticket_price: u64,
    pub lottery_id: u64,
    pub is_active: bool,
    pub prize_mint: Pubkey,
    pub vault: Pubkey,
    pub total_tickets: u64,
    pub winner_mint: Option<Pubkey>,
    pub randomness: Option<[u8; 32]>,
    pub bump: u8,
    #[max_len(1000)]
    pub tickets_mints: Vec<Pubkey>,
}
