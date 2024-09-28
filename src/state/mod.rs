use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{clock::Clock, entrypoint::ProgramResult, sysvar::Sysvar};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StateAccount {
    pub discriminator: u8,
    pub bump: u8,

    pub prev_height: u8,
    pub last_height: u8,
    pub next_height: u8,

    pub last_value: u64,
    pub next_value: u64,
    pub last_slot: u64,
}

impl StateAccount {
    pub const LEN: usize = 1 * 5 + 8 * 3;
    pub const DISCRIMINATOR: usize = 0;
    pub const TARGET: usize = 60 * 1000 * 2 / 400;
    pub const RESET: u64 = 0;
    pub const DEFUALT_AMOUNT: u64 = 1;
    pub const INIT_VALUE: u64 = 1_000_000;

    // there is a potential bug when USDC value is 0.000_001
    pub fn claim(&mut self, amount: u64) -> ProgramResult {
        let clock = Clock::get()?;
        let distance = clock.slot - self.last_slot;
        let depth = distance / StateAccount::TARGET as u64;

        let multiplier = if self.next_value <= 10 { 4 } else { 2 };
        let scale: u8 = if self.next_value <= 10 { 2 } else { 1 };
        let off_set = if self.next_value <= 10 { 1 } else { 0 };
        // let divisor = if self.next_value < 100 { 1000 } else { 100 };

        let (
            // EXTRACTED VALUES
            current_value,
            current_height,
            next_value,
            next_height,
            _transfer_amount,
            _pool_amount,
        ) = if self.next_height == 0 {
            let next_height = self.next_height + 1;
            let next_value = StateAccount::INIT_VALUE;
            let pool_amount = 0;

            (
                StateAccount::RESET,
                self.next_height,
                next_value,
                next_height,
                StateAccount::DEFUALT_AMOUNT,
                pool_amount,
            )
        } else if depth == 0 {
            let next_value = self.next_value * multiplier;
            let next_height = self.next_height + 1;
            let pool_amount = 0;

            (
                self.next_value,
                self.next_height,
                next_value,
                next_height,
                StateAccount::DEFUALT_AMOUNT,
                pool_amount,
            )
        } else if self.next_height > depth as u8 {
            let value = (self.next_value >> (depth * scale as u64)) - off_set;
            let current_value = value + value * depth / 100;
            let current_height = self.next_height - depth as u8;
            let next_height = current_height + 1;
            let next_value = value * 2;
            let transfer_amount = if amount > depth { depth } else { amount };
            let pool_amount = depth - transfer_amount;

            (
                current_value,
                current_height,
                next_value,
                next_height,
                transfer_amount,
                pool_amount,
            )
        } else {
            let value = (self.next_value >> (self.next_height * scale)) - off_set;
            let current_value = 0;
            let next_value = value + value * depth / 100;
            let current_height = 0;
            let next_height = 1;
            let transfer_amount = if amount > depth { depth } else { amount };
            let pool_amount = depth - transfer_amount;

            (
                current_value,
                current_height,
                next_value,
                next_height,
                transfer_amount,
                pool_amount,
            )
        };

        self.last_slot = clock.slot;
        self.last_height = current_height;
        self.last_value = current_value;

        self.next_height = next_height;
        self.next_value = next_value;
        self.prev_height = self.next_height;

        // if pool_amount != 0
        // transfer / mint pool amount to pool

        // if last_value != 0
        // transfer current value

        // transfer claim token

        Ok(())
    }
}
