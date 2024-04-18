use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, log::sol_log_compute_units, program::invoke},
};
use light_macros::pubkey;

declare_id!("EnusnErAr7SfFWTJvUXMNdXcXaXkWnpN6p217YZ5T8ou");

pub const HEIGHT: usize = 26;
/// Number of leaves per batch. More than 10 doesn't fit in the log_wrapper
/// instruction.
pub const NR_LEAVES: usize = 10;
pub const NR_BATCHES: usize = 63;

pub const NOOP_PROGRAM_ID: Pubkey = pubkey!("noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV");

#[program]
pub mod without_account {
    use super::*;

    pub fn append_leaves(ctx: Context<AppendLeaves>) -> Result<()> {
        for i in 0..NR_BATCHES {
            #[cfg(target_os = "solana")]
            let pos = GLOBAL_ALLOCATOR.get_heap_pos();
            test_event_emmitance(&ctx)?;
            #[cfg(target_os = "solana")]
            GLOBAL_ALLOCATOR.free_heap(pos);
            msg!("inserted batch {}", i);
        }
        Ok(())
    }
}

pub fn test_event_emmitance(ctx: &Context<AppendLeaves>) -> Result<()> {
    // let mut paths_buffer: Vec<Vec<PathNode>>
    let mut changelog_events = Vec::new();

    for i in 0..NR_LEAVES {
        let mut paths = Vec::with_capacity(1);
        let mut path = Vec::with_capacity(1);
        path.push(PathNode {
            node: [11u8; 32],
            index: i as u32,
        });
        paths.push(path);
        let changelog_event = ChangelogEventV1 {
            id: [5u8; 32],
            paths,
            seq: i as u64,
            index: i as u32,
        };
        changelog_events.push(ChangelogEvent::V1(changelog_event));
        #[cfg(target_os = "solana")]
        GLOBAL_ALLOCATOR.log_total_heap(format!("{}: appending changelog event", i).as_str());
    }
    let changelogs = Changelogs {
        changelogs: changelog_events,
    };
    #[cfg(target_os = "solana")]
    GLOBAL_ALLOCATOR.log_total_heap("before changelogs_bytes");

    // let mut changelogs_bytes = vec![6u8; 10240];
    // .serialize()
    let changelog_bytes = changelogs.try_to_vec()?;
    #[cfg(target_os = "solana")]
    GLOBAL_ALLOCATOR.log_total_heap("after emit_indexer_event");
    sol_log_compute_units();

    emit_indexer_event(
        changelog_bytes,
        &ctx.accounts.log_wrapper,
        &ctx.accounts.user,
    )?;
    sol_log_compute_units();
    Ok(())
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

#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
#[repr(C)]
pub enum ChangelogEvent {
    V1(ChangelogEventV1),
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct Changelogs {
    pub changelogs: Vec<ChangelogEvent>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Eq, PartialEq)]
pub struct PathNode {
    pub node: [u8; 32],
    pub index: u32,
}

#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
pub struct ChangelogEventV1 {
    /// Public key of the tree.
    pub id: [u8; 32],
    /// Merkle paths.
    pub paths: Vec<Vec<PathNode>>,
    /// Number of successful operations on the on-chain tree.
    pub seq: u64,
    /// Changelog event index.
    pub index: u32,
}

use std::{alloc::Layout, mem::size_of, ptr::null_mut, usize};

#[cfg(target_os = "solana")]
use anchor_lang::{
    prelude::*,
    solana_program::entrypoint::{HEAP_LENGTH, HEAP_START_ADDRESS},
};

#[cfg(target_os = "solana")]
#[global_allocator]
pub static GLOBAL_ALLOCATOR: BumpAllocator = BumpAllocator {
    start: HEAP_START_ADDRESS as usize,
    len: HEAP_LENGTH,
};

pub struct BumpAllocator {
    pub start: usize,
    pub len: usize,
}

impl BumpAllocator {
    const RESERVED_MEM: usize = size_of::<*mut u8>();

    #[cfg(target_os = "solana")]
    pub fn new() -> Self {
        Self {
            start: HEAP_START_ADDRESS as usize,
            len: HEAP_LENGTH,
        }
    }

    /// Returns the current position of the heap.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it returns a raw pointer.
    pub unsafe fn pos(&self) -> usize {
        let pos_ptr = self.start as *mut usize;
        *pos_ptr
    }

    /// Reset heap start cursor to position.
    ///
    /// # Safety
    ///
    /// Do not use this function if you initialized heap memory after pos which you still need.
    pub unsafe fn move_cursor(&self, pos: usize) {
        let pos_ptr = self.start as *mut usize;
        *pos_ptr = pos;
    }

    #[cfg(target_os = "solana")]
    pub fn log_total_heap(&self, msg: &str) -> u64 {
        const HEAP_END_ADDRESS: u64 = HEAP_START_ADDRESS as u64 + HEAP_LENGTH as u64;

        let heap_start = unsafe { self.pos() } as u64;
        let heap_used = HEAP_END_ADDRESS - heap_start;
        msg!("{}: total heap used: {}", msg, heap_used);
        heap_used
    }

    #[cfg(target_os = "solana")]
    pub fn get_heap_pos(&self) -> usize {
        let heap_start = unsafe { self.pos() } as usize;
        heap_start
    }

    #[cfg(target_os = "solana")]
    pub fn free_heap(&self, pos: usize) {
        unsafe { self.move_cursor(pos) };
    }
}

unsafe impl std::alloc::GlobalAlloc for BumpAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pos_ptr = self.start as *mut usize;

        let mut pos = *pos_ptr;
        if pos == 0 {
            // First time, set starting position
            pos = self.start + self.len;
        }
        pos = pos.saturating_sub(layout.size());
        pos &= !(layout.align().wrapping_sub(1));
        // if pos < self.start + GLOBAL_ALLOCATOR.RESERVED_MEM {
        //     return null_mut();
        // }
        *pos_ptr = pos;
        pos as *mut u8
    }
    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // no dellaoc in Solana runtime :*(
    }
}
