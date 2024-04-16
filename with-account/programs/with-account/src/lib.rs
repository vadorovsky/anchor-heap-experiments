use anchor_lang::prelude::*;

declare_id!("EKfe7NDwaeh6Z8uFRXWTM7B1dsUbjQUxvocmCwBdcNK2");

#[program]
pub mod with_account {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
