use std::mem;

use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use light_macros::pubkey;

declare_id!("EKfe7NDwaeh6Z8uFRXWTM7B1dsUbjQUxvocmCwBdcNK2");

pub const HEIGHT: usize = 26;
/// Number of changelog event batches we emit in one transaction.
pub const BATCHES: usize = 3;
/// Number of leaves per event batch.
pub const NR_LEAVES_BATCH: usize = 10;
// more than 10 doesn't fit in the noop instruction, so
// pub const NR_LEAVES_BATCH: usize = 11;

pub const NOOP_PROGRAM_ID: Pubkey = pubkey!("noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV");

#[program]
pub mod with_account {
    use super::*;

    pub fn append_leaves(ctx: Context<AppendLeaves>) -> Result<()> {
        let mut buffers = ctx.accounts.buffers.load_init()?;

        for i in 0..BATCHES {
            let mut changelogs = Vec::new();

            for j in 0..NR_LEAVES_BATCH {
                // For keeping the implementation ez, we keep one path per event so
                // far.
                let path = unsafe {
                    Vec::from_raw_parts(
                        buffers.paths_buffer[j].as_mut_ptr() as *mut PathNode,
                        HEIGHT,
                        HEIGHT,
                    )
                };
                let paths = vec![path];

                let changelog_event = ChangelogEventV1 {
                    id: [5u8; 32],
                    seq: j as u64,
                    index: j as u32,
                    paths,
                };
                changelogs.push(ChangelogEvent::V1(changelog_event));

                msg!("batch {}: leaf {}: pushed changelog", i, j);
            }

            let changelogs = Changelogs { changelogs };

            msg!("batch {}: about to serialize batch", i);
            changelogs.serialize(&mut buffers.serialization_buffer.as_mut_slice())?;
            let event = unsafe {
                Vec::from_raw_parts(
                    buffers.serialization_buffer.as_mut_ptr(),
                    buffers.serialization_buffer.len(),
                    buffers.serialization_buffer.len(),
                )
            };
            msg!("batch {}: serialized", i);

            msg!(
                "batch {}: serialization buffer len: {}",
                i,
                buffers.serialization_buffer.len()
            );

            emit_indexer_event(event, &ctx.accounts.log_wrapper, &ctx.accounts.user)?;

            msg!("batch {}: emitted event", i);

            // Clean up buffers.
            for j in 0..buffers.paths_buffer.len() {
                for k in 0..buffers.paths_buffer[j].len() {
                    buffers.paths_buffer[j][k] = 0;
                }
            }
            for j in 0..buffers.serialization_buffer.len() {
                buffers.serialization_buffer[j] = 0;
            }
        }

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
    msg!("checked noop");
    let instruction = Instruction {
        program_id: noop_program.key(),
        accounts: vec![],
        data,
    };
    msg!("instantiated instruction");
    invoke(
        &instruction,
        &[noop_program.to_account_info(), signer.to_account_info()],
    )?;
    msg!("invoked noop program");

    Ok(())
}

#[error_code]
pub enum MyErrorCode {
    #[msg("Invalid noop program publick key")]
    InvalidNoopPubkey,
}

#[account(zero_copy)]
pub struct Buffers {
    /// Buffer used by rkyv for changelog paths.
    // pub paths_buffer: [u8; (mem::size_of::<PathNode>() * HEIGHT * NR_LEAVES)
    //     + mem::size_of::<ChangelogEventV1>() * NR_LEAVES * mem::size_of::<Changelogs>()],
    pub paths_buffer: [[u8; mem::size_of::<PathNode>() * HEIGHT]; NR_LEAVES_BATCH],
    /// Buffer used by borsh to serialize the final event.
    // pub serialization_buffer: [u8; (mem::size_of::<PathNode>() * HEIGHT * NR_LEAVES)
    //     + mem::size_of::<ChangelogEventV1>() * NR_LEAVES * mem::size_of::<Changelogs>()],
    pub serialization_buffer: [u8; 10240],
}

#[derive(Accounts)]
pub struct AppendLeaves<'info> {
    pub user: Signer<'info>,
    #[account(zero)]
    pub buffers: AccountLoader<'info, Buffers>,
    /// CHECK: Checked manually in emit_indexer_event.
    pub log_wrapper: UncheckedAccount<'info>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
pub struct Changelogs {
    pub changelogs: Vec<ChangelogEvent>,
}

/// Event containing the Merkle path of the given
/// [`StateMerkleTree`](light_merkle_tree_program::state::StateMerkleTree)
/// change. Indexers can use this type of events to re-build a non-sparse
/// version of state Merkle tree.
#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
#[repr(C)]
pub enum ChangelogEvent {
    V1(ChangelogEventV1),
}

/// Node of the Merkle path with an index representing the position in a
/// non-sparse Merkle tree.
#[derive(AnchorDeserialize, AnchorSerialize, Debug, Eq, PartialEq)]
pub struct PathNode {
    pub node: [u8; 32],
    pub index: u32,
}

/// Version 1 of the [`ChangelogEvent`](light_merkle_tree_program::state::ChangelogEvent).
#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
pub struct ChangelogEventV1 {
    /// Public key of the tree.
    pub id: [u8; 32],
    // Merkle paths.
    pub paths: Vec<Vec<PathNode>>,
    /// Number of successful operations on the on-chain tree.
    pub seq: u64,
    /// Changelog event index.
    pub index: u32,
}

// /// Version 1 of the [`ChangelogEvent`](light_merkle_tree_program::state::ChangelogEvent).
// #[derive(AnchorDeserialize, AnchorSerialize, Debug)]
// pub struct ChangelogEventV1 {
//     pub meta: ChangelogEventMeta,
//     // Merkle paths.
//     pub paths: Vec<Vec<PathNode>>,
// }
