use anchor_lang::prelude::*;
mod state;
mod instructions;
declare_id!("FjuGSfv4un5gnM4mmY7GVWhfhckLqrjqfsY1xfdF8VcN");

#[program]
pub mod token_lottery {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
