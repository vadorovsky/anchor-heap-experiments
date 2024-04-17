use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use light_macros::pubkey;

declare_id!("EnusnErAr7SfFWTJvUXMNdXcXaXkWnpN6p217YZ5T8ou");

pub const HEIGHT: usize = 26;
pub const NR_LEAVES: usize = 8;
// more than 8 causes memory allocation error
// pub const NR_LEAVES: usize = 9;

pub const NOOP_PROGRAM_ID: Pubkey = pubkey!("noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV");

#[program]
pub mod without_account {
    use super::*;

    pub fn append_leaves(ctx: Context<AppendLeaves>) -> Result<()> {
        let mut changelog_events = Vec::new();
        // for (i, leaf) in LEAVES.iter().enumerate() {
        for i in 0..NR_LEAVES {
            let changelog_event = ChangelogEvent {
                id: [5u8; 32],
                seq: i as u64,
                index: i as u32,
            };
            changelog_events.push(changelog_event);
        }
        let changelogs = Changelogs {
            changelogs: changelog_events,
        };

        let mut changelogs_bytes = changelogs.try_to_vec()?;

        for i in 0..NR_LEAVES {
            // Fake Merkle path.
            // 32 (root) + 32 * HEIGHT (Merkle path) + 8 (index)
            changelogs_bytes.extend_from_slice(&[6u8; 32 + 32 * HEIGHT + 8]);
            msg!("{i}: changelog_bytes size: {}", changelogs_bytes.len());
        }

        emit_indexer_event(
            changelogs_bytes,
            &ctx.accounts.log_wrapper,
            &ctx.accounts.user,
        )?;

        Ok(())
    }
}

#[inline(never)]
pub fn emit_indexer_event<'info>(
    data: Vec<u8>,
    noop_program: &AccountInfo<'info>,
    signer: &AccountInfo<'info>,
) -> Result<()> {
    if noop_program.key() != NOOP_PROGRAM_ID {
        return err!(MyErrorCode::InvalidNoopPubkey);
    }
    let instruction = Instruction {
        program_id: noop_program.key(),
        accounts: vec![],
        data,
    };
    invoke(
        &instruction,
        &[noop_program.to_account_info(), signer.to_account_info()],
    )?;

    Ok(())
}

#[error_code]
pub enum MyErrorCode {
    #[msg("Invalid noop program publick key")]
    InvalidNoopPubkey,
}

#[derive(Accounts)]
pub struct AppendLeaves<'info> {
    pub user: Signer<'info>,
    /// CHECK: Checked manually in emit_indexer_event.
    pub log_wrapper: UncheckedAccount<'info>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct Changelogs {
    pub changelogs: Vec<ChangelogEvent>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct ChangelogEvent {
    /// Public key of the tree.
    pub id: [u8; 32],
    /// Number of successful operations on the on-chain tree.
    pub seq: u64,
    /// Changelog event index.
    pub index: u32,
}
