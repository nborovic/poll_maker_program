use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

use crate::{
    instruction::PollMakerInstruction,
    processor::{create_poll, vote_on_poll},
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!(
        "process_instruction: {}: {} accounts, data={:?}",
        program_id,
        accounts.len(),
        instruction_data
    );

    let instruction = PollMakerInstruction::unpack(instruction_data)?;

    match instruction {
        PollMakerInstruction::CreatePoll(mut payload) => {
            create_poll(&program_id, &accounts, &mut payload)
        }
        PollMakerInstruction::VoteOnPoll(mut payload) => {
            vote_on_poll(&program_id, &accounts, &mut payload)
        }
    }
}
