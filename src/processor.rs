use std::mem::take;

use borsh::BorshSerialize;
use solana_program::borsh0_10::try_from_slice_unchecked;
use solana_program::clock::Clock;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use solana_program::{msg, system_program};

use crate::state::{GlobalState, VoteState};
use crate::{
    instruction::{CreatePollPayload, VoteOnPollPayload},
    state::PollState,
};

pub fn create_poll(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    payload: &mut CreatePollPayload,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initiator = next_account_info(accounts_iter)?;
    let global_pda_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    assert!(initiator.is_signer);
    assert!(initiator.is_writable);
    assert!(global_pda_account.is_writable);
    assert!(pda_account.is_writable);
    assert!(system_program::check_id(system_program.key));

    let (global_pda, global_bump_seed) =
        Pubkey::find_program_address(&[GlobalState::DISCRIMINATOR.as_bytes()], program_id);

    assert_eq!(global_pda_account.key, &global_pda);

    let rent = Rent::get()?;

    let is_global_state_initialized = **global_pda_account.try_borrow_lamports()? > 0;

    if !is_global_state_initialized {
        let global_account_len = GlobalState::SIZE;

        let global_rent_lamports = rent.minimum_balance(global_account_len);

        invoke_signed(
            &system_instruction::create_account(
                initiator.key,
                global_pda_account.key,
                global_rent_lamports,
                global_account_len.try_into().unwrap(),
                program_id,
            ),
            &[
                initiator.clone(),
                global_pda_account.clone(),
                system_program.clone(),
            ],
            &[&[GlobalState::DISCRIMINATOR.as_bytes(), &[global_bump_seed]]],
        )?;
    }

    let mut global_state =
        try_from_slice_unchecked::<GlobalState>(&global_pda_account.data.borrow()).unwrap();

    if !is_global_state_initialized {
        let initial_global_state = GlobalState::new();

        global_state.discriminator = initial_global_state.discriminator;
        global_state.id_counter = initial_global_state.id_counter;

        msg!("Serializing initial global state");
        global_state.serialize(&mut &mut global_pda_account.data.borrow_mut()[..])?;
    }

    let id_as_string = global_state.id_counter.to_string();
    let id_as_bytes = id_as_string.as_bytes();

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[PollState::DISCRIMINATOR.as_bytes(), &id_as_bytes],
        program_id,
    );

    assert_eq!(pda_account.key, &pda);

    let payload_as_state = PollState {
        discriminator: PollState::DISCRIMINATOR.to_string(),
        is_initialized: true,
        id: global_state.id_counter,
        creator_address: initiator.key.to_string(),
        title: take(&mut payload.title),
        description: take(&mut payload.description),
        ends_at: payload.ends_at,
        options: take(&mut payload.options),
        authorized_addresses: take(&mut payload.authorized_addresses),
    };

    let account_len = payload_as_state.get_account_size();

    assert!(account_len < 100000);

    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            initiator.key,
            pda_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initiator.clone(),
            pda_account.clone(),
            system_program.clone(),
        ],
        &[&[
            PollState::DISCRIMINATOR.as_bytes(),
            &id_as_bytes,
            &[bump_seed],
        ]],
    )?;

    let mut account_data =
        try_from_slice_unchecked::<PollState>(&pda_account.data.borrow()).unwrap();

    assert!(!account_data.is_initialized());

    account_data.discriminator = payload_as_state.discriminator;
    account_data.is_initialized = payload_as_state.is_initialized;
    account_data.id = payload_as_state.id;
    account_data.creator_address = payload_as_state.creator_address;
    account_data.title = payload_as_state.title;
    account_data.description = payload_as_state.description;
    account_data.ends_at = payload_as_state.ends_at;
    account_data.options = payload_as_state.options;
    account_data.authorized_addresses = payload_as_state.authorized_addresses;

    msg!("Serializing initial poll state");
    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    global_state.id_counter += 1;

    msg!("Serializing global state");
    global_state.serialize(&mut &mut global_pda_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn vote_on_poll(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    payload: &mut VoteOnPollPayload,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initiator = next_account_info(accounts_iter)?;
    let poll_pda_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    assert!(initiator.is_signer);
    assert!(initiator.is_writable);
    assert!(pda_account.is_writable);
    assert_eq!(poll_pda_account.owner, program_id);
    assert!(system_program::check_id(system_program.key));

    let payload_as_state = VoteState {
        discriminator: VoteState::DISCRIMINATOR.to_string(),
        is_initialized: true,
        poll_id: payload.poll_id,
        voter_address: initiator.key.to_string(),
        option: take(&mut payload.option),
    };

    let poll_id_as_string = payload_as_state.poll_id.to_string();
    let poll_id_as_bytes = poll_id_as_string.as_bytes();

    let (poll_pda, _) = Pubkey::find_program_address(
        &[PollState::DISCRIMINATOR.as_bytes(), &poll_id_as_bytes],
        program_id,
    );

    assert_eq!(poll_pda_account.key, &poll_pda);

    let poll_state =
        try_from_slice_unchecked::<PollState>(&poll_pda_account.data.borrow()).unwrap();

    assert!(poll_state.is_initialized);

    let clock = Clock::get()?;

    assert!((clock.unix_timestamp as u64).lt(&poll_state.ends_at));
    assert!(poll_state
        .authorized_addresses
        .contains(&initiator.key.to_string()));
    assert!(poll_state.options.contains(&payload_as_state.option));

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[
            VoteState::DISCRIMINATOR.as_bytes(),
            initiator.key.as_ref(),
            &poll_id_as_bytes,
        ],
        program_id,
    );

    assert_eq!(pda_account.key, &pda);

    let account_size = payload_as_state.get_account_size();

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_size);

    invoke_signed(
        &system_instruction::create_account(
            initiator.key,
            pda_account.key,
            rent_lamports,
            account_size.try_into().unwrap(),
            program_id,
        ),
        &[
            initiator.clone(),
            pda_account.clone(),
            system_program.clone(),
        ],
        &[&[
            VoteState::DISCRIMINATOR.as_bytes(),
            initiator.key.as_ref(),
            &poll_id_as_bytes,
            &[bump_seed],
        ]],
    )?;

    let mut account_state =
        try_from_slice_unchecked::<VoteState>(&pda_account.data.borrow()).unwrap();

    assert!(!account_state.is_initialized());

    account_state.discriminator = payload_as_state.discriminator;
    account_state.is_initialized = payload_as_state.is_initialized;
    account_state.poll_id = payload_as_state.poll_id;
    account_state.voter_address = payload_as_state.voter_address;
    account_state.option = payload_as_state.option;

    msg!("Serializing initial vote state");

    account_state.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    Ok(())
}
