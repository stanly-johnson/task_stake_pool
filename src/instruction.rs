#![allow(dead_code)]
use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{clock::UnixTimestamp, pubkey::Pubkey},
};

/// The list of instructions to the program
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub enum TaskInstruction {
    /// Create a new task in the pool, the caller will be
    /// assigned as the manager of the task.
    CreateTask {
        task_audit_program: String,
        stake_amount: u64,
        deadline: UnixTimestamp,
        stake_pot_account: Pubkey,
    },
    /// A submitter can submit a new task,
    /// this will store the submission into the storage and deduct a
    /// stake_amount from the submitter.
    SubmitTask(String),
    /// Withdraw Submission, will remove the submission from the task state
    /// and return the stake amount deducted from the submitter
    WithdrawSubmission,
    /// Set the TaskStatus to Voting, can only be called by the manager
    /// of the task. Once the TaskStatus is set to Voting, no submissions can be
    /// added. The voting deadline should be set in this instruction.
    SetTaskToVoting(UnixTimestamp),
    /// Vote for a Submission, when voting a bounty amount is deducted from
    /// the voting account, which will be paid out to the succesful submission
    Vote,
    /// The manager can trigger a payout to the winning submission once the
    // voting deadline has passed.
    Payout,
}
