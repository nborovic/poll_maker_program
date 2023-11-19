use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum PollMakerInstruction {
    CreatePoll(CreatePollPayload),
    VoteOnPoll(VoteOnPollPayload),
}

#[derive(BorshDeserialize)]
pub struct CreatePollPayload {
    pub title: String,
    pub description: String,
    pub ends_at: u64,
    pub options: Vec<String>,
    pub authorized_addresses: Vec<String>,
}

#[derive(BorshDeserialize)]
pub struct VoteOnPollPayload {
    pub poll_id: u64,
    pub option: String,
}

impl PollMakerInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        match variant {
            0 => {
                let payload: CreatePollPayload = CreatePollPayload::try_from_slice(&rest).unwrap();
                Ok(PollMakerInstruction::CreatePoll(payload))
            }
            1 => {
                let payload = VoteOnPollPayload::try_from_slice(&rest).unwrap();
                Ok(PollMakerInstruction::VoteOnPoll(payload))
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
