use anchor_lang::prelude::*;

declare_id!("4FM5723KLZEfk6H4UN9xMTjt5Kw9pPYZNbmHYrNqrFEh");

#[program]
pub mod on_chain {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
