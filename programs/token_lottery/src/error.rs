use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Lottery Is Closed")]
    LotteryClosed,
    #[msg("Winner Not Chosen")]
    WinnerNotChosen,
}
