use anchor_lang::prelude::*;

declare_id!("EnusnErAr7SfFWTJvUXMNdXcXaXkWnpN6p217YZ5T8ou");

#[program]
pub mod without_account {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
