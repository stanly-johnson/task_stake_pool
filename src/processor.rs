use {
    crate::instruction::TaskInstruction,
    crate::state::{TaskState, TaskStatus},
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    solana_program::{
        account_info::next_account_info, borsh::try_from_slice_unchecked, clock::UnixTimestamp,
        entrypoint::ProgramResult, msg, program::invoke, program_error::ProgramError,
        pubkey::Pubkey, system_instruction, sysvar::clock::Clock,
        sysvar::slot_history::AccountInfo, sysvar::Sysvar,
    },
};

pub struct Processor {}

impl Processor {
    /// entry point for program
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        use TaskInstruction::*;
        let instruction = TaskInstruction::try_from_slice(input)?;
        match instruction {
            CreateTask {
                task_audit_program,
                stake_amount,
                deadline,
                stake_pot_account,
            } => {
                msg!("Instruction: Create Task");
                Self::process_create_task(
                    program_id,
                    accounts,
                    task_audit_program,
                    stake_amount,
                    deadline,
                    stake_pot_account,
                )
            }
            SubmitTask(submission) => {
                msg!("Instruction: Submit Task");
                Self::process_submit_task(program_id, accounts, submission)
            }
            SetTaskToVoting(deadline) => {
                msg!("Instruction: Set Task to Voting");
                Self::process_start_voting(program_id, accounts, deadline)
            }
            Vote => {
                msg!("Instruction: Vote for Submission");
                Self::process_vote(program_id, accounts)
            }
            Payout => {
                msg!("Instruction: Payout");
                Self::process_payout(program_id, accounts)
            }
            _ => return Err(ProgramError::InvalidArgument),
        }?;
        Ok(())
    }

    /**
    Create a new task and set state to AcceptingSubmissions, a new task account will be created to be
    controlled by the manager_account.
    */
    fn process_create_task(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        task_audit_program: String,
        stake_amount: u64,
        deadline: UnixTimestamp,
        stake_pot_account: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let manager_info = next_account_info(account_info_iter)?;
        let task_state_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;

        // TODO : check the task account is not initialized

        invoke(
            &system_instruction::create_account(
                manager_info.key,
                task_state_info.key,
                1,
                20,
                program_id,
            ),
            &[
                manager_info.clone(),
                task_state_info.clone(),
                system_program_info.clone(),
            ],
        )?;

        let mut task = try_from_slice_unchecked::<TaskState>(&task_state_info.data.borrow())?;
        task.task_manager = *manager_info.key;
        task.task_audit_program = task_audit_program;
        task.submissions = Default::default();
        task.votes = Default::default();
        task.stake_amount = stake_amount;
        task.stake_pot_account = stake_pot_account;
        task.status = TaskStatus::AcceptingSubmissions(deadline);

        task.serialize(&mut *task_state_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    /**
    Submit a task to open task account. The submitter will deposit the stake_amount to the
    stake_pot as part of the submission.
    */
    fn process_submit_task(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        submission: String,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let task_state_info = next_account_info(account_info_iter)?;
        let submitter_info = next_account_info(account_info_iter)?;
        let stake_pot_account = next_account_info(account_info_iter)?;
        let clock_sysvar_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_sysvar_info)?;

        Self::check_account_owner(task_state_info, program_id)?;
        let mut task = try_from_slice_unchecked::<TaskState>(&task_state_info.data.borrow())?;

        // ensure the task is still accepting submissions
        match task.status {
            TaskStatus::AcceptingSubmissions(deadline) => {
                // ensure the acceptance window has not expired
                if clock.unix_timestamp >= deadline {
                    return Err(ProgramError::InvalidArgument);
                }
            }
            _ => {
                msg!("Error: Task not in AcceptingSubmissions mode");
                return Err(ProgramError::InvalidArgument);
            }
        }

        // ensure stake pot account is correct
        if *stake_pot_account.key != task.stake_pot_account {
            return Err(ProgramError::InvalidArgument);
        }

        // transfer deposit to stake pot account
        invoke(
            &system_instruction::transfer(
                submitter_info.key,
                stake_pot_account.key,
                task.stake_amount,
            ),
            &[submitter_info.clone(), stake_pot_account.clone()],
        )?;

        // update the task stake amount
        task.total_stake_amount = task.total_stake_amount + task.stake_amount;

        // add submission to task state
        task.submissions.insert(*submitter_info.key, submission);

        task.serialize(&mut *task_state_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    /**
    Set the task account state to Voting. The manager_account can set the task account to
    Voting. This ensures no new submissions are accepted and accounts can only vote on existing
    submissions.
    */
    fn process_start_voting(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        deadline: UnixTimestamp,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let task_state_info = next_account_info(account_info_iter)?;
        let manager_signer = next_account_info(account_info_iter)?;
        let clock_sysvar_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_sysvar_info)?;

        // sanity check : ensure deadline is in the future
        if clock.unix_timestamp <= deadline {
            return Err(ProgramError::InvalidArgument);
        }

        Self::check_account_owner(task_state_info, program_id)?;
        let mut task = try_from_slice_unchecked::<TaskState>(&task_state_info.data.borrow())?;

        // ensure manager is signer
        if !manager_signer.is_signer {
            msg!("The task manager is not a signer");
            return Err(ProgramError::InvalidAccountData);
        }

        // set the task status to voting
        task.status = TaskStatus::Voting(deadline);

        task.serialize(&mut *task_state_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    /**
    Vote for a submission. Only a user with an existing stake/submission will be able to vote and
    a user will only have a single vote.
    */
    fn process_vote(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let task_state_info = next_account_info(account_info_iter)?;
        let voter_info = next_account_info(account_info_iter)?;
        let candidate_info = next_account_info(account_info_iter)?;
        let clock_sysvar_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_sysvar_info)?;

        Self::check_account_owner(task_state_info, program_id)?;
        let mut task = try_from_slice_unchecked::<TaskState>(&task_state_info.data.borrow())?;

        // ensure the voting period is active
        match task.status {
            TaskStatus::Voting(deadline) => {
                // ensure the acceptance window has not expired
                if clock.unix_timestamp >= deadline {
                    return Err(ProgramError::InvalidArgument);
                }
            }
            _ => {
                msg!("Error: Task not in Voting mode");
                return Err(ProgramError::InvalidArgument);
            }
        }

        // ensure voter is a submitter and has staked amount
        if !task.submissions.contains_key(voter_info.key) {
            msg!("The voter has not staked any amount");
            return Err(ProgramError::InvalidAccountData);
        }

        // ensure the candidate has submitted a task
        if !task.submissions.contains_key(candidate_info.key) {
            msg!("The candidate has not submitted a task!");
            return Err(ProgramError::InvalidAccountData);
        }

        // ensure the voter has not already voted
        if task.votes.contains_key(voter_info.key) {
            msg!("The voter has already voted!");
            return Err(ProgramError::InvalidAccountData);
        }

        // register the vote
        task.votes.insert(*voter_info.key, *candidate_info.key);

        task.serialize(&mut *task_state_info.data.borrow_mut())
            .map_err(|e| e.into())
    }

    // The manager account can payout the stake_pot amount to the user with winning votes
    fn process_payout(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
        todo!()
    }

    /// Check account owner is the given program
    fn check_account_owner(
        account_info: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<(), ProgramError> {
        if *program_id != *account_info.owner {
            msg!(
                "Expected account to be owned by program {}, received {}",
                program_id,
                account_info.owner
            );
            Err(ProgramError::IncorrectProgramId)
        } else {
            Ok(())
        }
    }
}
