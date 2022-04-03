### Task Stake Pool

Solana program to create a Task registry pool where users can pool funds to vote on tasks.

#### Expected Control Flow

1. User creates a task account that accepts Task submissions
2. A user can submit tasks to the account, will require to stake a deposit to the task pool
3. Once the submission period has completed, the voting period starts
4. In the voting period, all users who have staked previously can vote for a task
5. At the end of the voting period, the manager can payout the stake amount to the winning task (TBD)

#### States of a Task Proposal Account
AcceptingSubmissions(expiry_time) : This is the first state of the Task, in this state new accounts can submit task programs
until the expiry_time has passed.

Voting(expiry_time) : This state is set by the task_manager, in this state no new submissions are accepted, instead votes
can be cast by the submitted accounts until the expiry_time has passed.

Completed : This state is set after the voting has completed and payout has completed. Alternatively the task account can be deleted.

Cancelled : The task status has been set to cancelled and the submitters can withdraw the bounty and submitted tasks.