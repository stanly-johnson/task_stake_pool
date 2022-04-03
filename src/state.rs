use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::clock::UnixTimestamp,
    solana_program::pubkey::Pubkey,
    std::collections::BTreeMap,
};

/**
Contents of a Task Proposal Account
AcceptingSubmissions(expiry_time) : This is the first state of the Task, in this state new accounts can submit task programs
until the expiry_time has passed.

Voting(expiry_time) : This state is set by the task_manager, in this state no new submissions are accepted, instead votes
can be cast by the submitted accounts until the expiry_time has passed.

Completed : This state is set after the voting has completed and payout has completed. Alternatively the task account can be deleted.

Cancelled : The task status has been set to cancelled and the submitters can withdraw the bounty and submitted tasks.
*/
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum TaskStatus {
    /// Task proposal is now ready to accept submissions
    AcceptingSubmissions(UnixTimestamp),
    /// Task is ready to be voted on
    Voting(UnixTimestamp),
    /// The task is completed and bounty paid out
    Completed,
    /// The task is cancelled and stake returned
    Cancelled,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct TaskState {
    /// The account that created the task
    pub task_manager: Pubkey,
    /// the task_audit_program reference stored offchain
    pub task_audit_program: String,
    /// The stake pot account
    pub stake_pot_account: Pubkey,
    /// Map of all submissions {key -> task_program}
    pub submissions: BTreeMap<Pubkey, String>,
    /// The map of all votes by the submitters {key -> key}
    pub votes: BTreeMap<Pubkey, Pubkey>,
    /// The amount deducted from the submission user
    pub stake_amount: u64,
    /// The total stake amount in the stake_pot_account
    pub total_stake_amount: u64,
    // The current status of Task [Accepting, Voting, Completed, Cancelled]
    pub status: TaskStatus,
}
