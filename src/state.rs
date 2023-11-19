use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::{IsInitialized, Sealed};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct GlobalState {
    pub discriminator: String,
    pub is_initialized: bool,
    pub id_counter: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct PollState {
    pub discriminator: String,
    pub is_initialized: bool,
    pub id: u64,
    pub creator_address: String,
    pub title: String,
    pub description: String,
    pub ends_at: u64,
    pub options: Vec<String>,
    pub authorized_addresses: Vec<String>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct VoteState {
    pub discriminator: String,
    pub is_initialized: bool,
    pub poll_id: u64,
    pub voter_address: String,
    pub option: String,
}

impl Sealed for GlobalState {}

impl Sealed for PollState {}

impl Sealed for VoteState {}

impl IsInitialized for GlobalState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for PollState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for VoteState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl GlobalState {
    pub const DISCRIMINATOR: &'static str = "global";
    pub const SIZE: usize = (4 + Self::DISCRIMINATOR.len()) + 1 + 8;

    pub fn new() -> Self {
        Self {
            discriminator: Self::DISCRIMINATOR.to_string(),
            is_initialized: true,
            id_counter: 1,
        }
    }
}

impl PollState {
    pub const DISCRIMINATOR: &'static str = "poll";

    pub fn get_account_size(&self) -> usize {
        let mut len = (4 + Self::DISCRIMINATOR.len())
            + 1
            + 8
            + (4 + self.creator_address.len())
            + (4 + self.title.len())
            + (4 + self.description.len())
            + 8
            + 4
            + 4;

        for option in self.options.iter() {
            len += 4 + option.len();
        }

        for authorized_address in self.authorized_addresses.iter() {
            len += 4 + authorized_address.len();
        }

        len
    }
}

impl VoteState {
    pub const DISCRIMINATOR: &'static str = "vote";

    pub fn get_account_size(&self) -> usize {
        (4 + Self::DISCRIMINATOR.len())
            + 1
            + 8
            + (4 + self.voter_address.len())
            + (4 + self.option.len())
    }
}
