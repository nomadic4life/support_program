use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{clock::Clock, program_error::ProgramError, sysvar::Sysvar};

type Result = std::result::Result<(u64, u64, u64), ProgramError>;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StateAccount {
    pub discriminator: u8,
    pub bump: u8,

    pub prev_height: u8,
    pub last_height: u8,
    pub next_height: u8,
    pub accummulated_depth: u16,

    pub last_value: u64,
    pub next_value: u64,
    pub last_slot: u64,

    pub total_contributions: u64,
    pub total_claimed: u64,
}

impl StateAccount {
    pub const LEN: usize = 1 * 5 + 2 * 1 + 8 * 5;
    pub const DISCRIMINATOR: usize = 0;
    pub const TARGET: usize = 60 * 1000 * 1 / 400;
    pub const RESET: u64 = 0;
    pub const CLAIM_MINT: u64 = 1_000_000_000;
    pub const INIT_VALUE: u64 = 1_000_000;

    pub fn update(&mut self) -> Result {
        let clock = Clock::get()?;
        let distance = clock.slot - self.last_slot;
        let count = distance / StateAccount::TARGET as u64;
        let depth = StateAccount::get_depth(count);

        let multiplier = if self.next_value <= 10 { 4 } else { 2 };
        let off_set = if self.next_value <= 100 { 1 } else { 0 };

        let (
            // EXTRACTED VALUES
            current_value,
            current_height,
            next_value,
            next_height,
            pool_mint,
        ) = if self.next_height == 0 {
            let current_value = StateAccount::RESET;
            let current_height = self.next_height;
            let next_value = StateAccount::INIT_VALUE;
            let next_height = self.next_height + 1;
            let pool_mint = count * StateAccount::CLAIM_MINT;

            (
                current_value,
                current_height,
                next_value,
                next_height,
                pool_mint,
            )
        } else if depth == 0 {
            let next_value = self.next_value * multiplier;
            let next_height = self.next_height + 1;
            let pool_mint = count * StateAccount::CLAIM_MINT;

            (
                self.next_value,
                self.next_height,
                next_value,
                next_height,
                pool_mint,
            )
        } else if self.next_height >= depth {
            let value = (self.next_value >> depth) - off_set;
            let current_value = value + value * depth as u64 / 100;
            let current_height = self.next_height - depth;
            let next_value = value * 2;
            let next_height = current_height + 1;
            let pool_mint = count * StateAccount::CLAIM_MINT;

            (
                current_value,
                current_height,
                next_value,
                next_height,
                pool_mint,
            )
        } else {
            let value = (self.next_value >> self.next_height) - off_set;
            let current_value = 0;
            let current_height = 0;
            let next_value = value + value * self.next_height as u64 / 100;
            let next_height = 1;
            let pool_mint = count * StateAccount::CLAIM_MINT;

            (
                current_value,
                current_height,
                next_value,
                next_height,
                pool_mint,
            )
        };

        self.last_slot = clock.slot;
        self.last_height = current_height;
        self.last_value = current_value;

        self.next_height = next_height;
        self.next_value = next_value;
        self.prev_height = self.next_height;

        // don't remember what the accummulate depth is for?
        self.accummulated_depth += depth as u16;
        self.total_claimed += StateAccount::CLAIM_MINT;
        self.total_contributions += current_value;

        Ok((current_value, StateAccount::CLAIM_MINT, pool_mint))
    }

    fn get_depth(count: u64) -> u8 {
        let depth = if count >= 144 {
            64
        } else if count >= 133 {
            (count - 133) + 53
        } else if count >= 69 {
            (count - 69) / 2 + 21
        } else if count >= 21 {
            (count - 21) / 3 + 5
        } else if count >= 9 {
            (count - 9) / 4 + 1
        } else if count > 5 {
            1
        } else {
            0
        } as u8;

        return depth;
    }
}

// Notes:
//  How Claim Works
//      -   One token is available to claim, the avaiable to claim increases by one for every
//          minute that is passed, or more accuretly for every 150 slots.
//      -   the height, starts at 0 and increases for every claim,
//          it also decreases when a claim is not executed with in a minute
//          until the height is 0,
//      -   when the height is 0, to claim a to the cost is 0
//      -   when the height is 1 or above, their is a cost to claim
//      -   the depth starts at 0 and increases when the height decreases,
//          for every increase in the depth, 1 token is added to the pool
//          the depth resets to 0 when a claim is executed
//      -   when the height is not 0, on claims the price doubles, and when the depth
//          the price decreases by half + 1% offset
