use std::io::Write;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use bryte_descriptor_attribute::state_descriptor;
use bryte_descriptor_state::states::Descriptor;
use bryte_descriptor_state::states::SchemaEventAnchor;
use bryte_descriptor_state::states::{
    DescriptorDeserialize, DescriptorSerialize, Discriminator, SchemaEvent,
};
use capnp::message::ReaderOptions;
use capnp::serialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use bryte_descriptor_state::discriminators::custom_discriminator;

entrypoint!(instruction);

pub mod person_capnp {
    include!("../../capnp-models/src/person_capnp.rs");
}

const PERSON_SCHEMA: &[u8] = include_bytes!("../../capnp-models/src/person.capnp");

#[state_descriptor]
#[derive(Default, Debug)]
pub struct PersonState {
    is_initialized: bool,
    first_name: String,
    last_name: String,
}

impl PersonState {
    const SIZE: usize = 8 + 1 + 24 + 24;
}

#[derive(Debug)]
pub enum DemoInstruction {
    InitializeCapnp { bump: Vec<u8> },
    DeserializeCapnpDemo,
    InitializeBorsh { bump: Vec<u8> },
    DeserializeBorshDemo,
}

#[derive(BorshDeserialize)]
pub struct DemoPayload {
    bump: Vec<u8>,
}

impl DemoInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match variant {
            0 => DemoInstruction::InitializeCapnp {
                bump: rest.to_vec(),
            },
            1 => DemoInstruction::DeserializeCapnpDemo,
            2 => DemoInstruction::InitializeBorsh {
                bump: rest.to_vec(),
            },
            3 => DemoInstruction::DeserializeBorshDemo,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}

pub fn initialize_capnp(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    // Getting required accounts
    let funding_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let pda_account_descriptor = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Getting PDA Bump from instruction data
    let (pda_bump, _) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // Checking if passed PDA and expected PDA are equal
    let signers_seeds: &[&[u8]; 3] = &[
        b"customaddress",
        &funding_account.key.to_bytes(),
        &[*pda_bump],
    ];
    let pda = Pubkey::create_program_address(signers_seeds, program_id)?;
    msg!("pda {:?}", pda);

    if pda.ne(&pda_account.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut message = capnp::message::Builder::new_default();
    let mut person_state_builder = message.init_root::<person_capnp::person::Builder>();
    person_state_builder.set_firstname("Captain".into());
    person_state_builder.set_lastname("Proto".into());


    // Potentially use this size to do space allocation
    // let size = serialize::compute_serialized_size_in_words(&message) + 8;

    // Assessing required lamports and creating transaction instruction
    let lamports_required = Rent::get()?.minimum_balance(200);
    let create_pda_account_ix = system_instruction::create_account(
        &funding_account.key,
        &pda_account.key,
        lamports_required,
        200,
        &program_id,
    );
    // Invoking the instruction but with PDAs as additional signer
    invoke_signed(
        &create_pda_account_ix,
        &[
            funding_account.clone(),
            pda_account.clone(),
            system_program.clone(),
        ],
        &[signers_seeds],
    )?;

    let discriminator = custom_discriminator("account".to_string(), "person".to_string());
    &pda_account.data.borrow_mut()[0..8].copy_from_slice(&discriminator);
    serialize::write_message(&mut &mut pda_account.data.borrow_mut()[8..], &message)
        .expect("Error serializing data using capnp");

    let (pda_descriptor, pda_descriptor_bump) =
        Pubkey::find_program_address(&[&discriminator], &program_id);

    if pda_descriptor.ne(&pda_account_descriptor.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    // TODO get the required space dynamically...
    let lamports_required = Rent::get()?.minimum_balance(193);
    let create_pda_account_descriptor_ix = system_instruction::create_account(
        &funding_account.key,
        &pda_descriptor,
        lamports_required,
        193,
        &program_id,
    );

    // Invoking the instruction but with PDAs as additional signer
    invoke_signed(
        &create_pda_account_descriptor_ix,
        &[
            funding_account.clone(),
            pda_account_descriptor.clone(),
            system_program.clone(),
        ],
        &[&[&discriminator, &[pda_descriptor_bump]]],
    )?;

    // Store the person.capnp schema on-chain
    &pda_account_descriptor.data.borrow_mut()[..].copy_from_slice(&PERSON_SCHEMA);

    Ok(())
}

pub fn deserialize_capnpn_demo(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // TODO clean up unnecessary accounts.
    let accounts_iter = &mut accounts.iter();
    // Getting required accounts
    let funding_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let pda_account_descriptor = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Get the bytes after the discriminator.
    let mut data = &pda_account.data.borrow_mut()[8..];
    // Use capnp reader to deserialize the data
    let reader = serialize::read_message_from_flat_slice_no_alloc(&mut data, ReaderOptions::new()).unwrap();

    let person = reader.get_root::<person_capnp::person::Reader>().unwrap();
    msg!("{:?} {:?}",  person.get_firstname(),  person.get_lastname());

    Ok(())
}

pub fn initialize_borsh(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    // Getting required accounts
    let funding_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let pda_account_descriptor = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Getting PDA Bump from instruction data
    let (pda_bump, _) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // Checking if passed PDA and expected PDA are equal
    let signers_seeds: &[&[u8]; 3] = &[
        b"customaddress",
        &funding_account.key.to_bytes(),
        &[*pda_bump],
    ];
    let pda = Pubkey::create_program_address(signers_seeds, program_id)?;
    msg!("pda {:?}", pda);

    if pda.ne(&pda_account.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    // Assessing required lamports and creating transaction instruction
    let lamports_required = Rent::get()?.minimum_balance(500);
    let create_pda_account_ix = system_instruction::create_account(
        &funding_account.key,
        &pda_account.key,
        lamports_required,
        500,
        &program_id,
    );
    // Invoking the instruction but with PDAs as additional signer
    invoke_signed(
        &create_pda_account_ix,
        &[
            funding_account.clone(),
            pda_account.clone(),
            system_program.clone(),
        ],
        &[signers_seeds],
    )?;

    // Setting state for PDA
    let pda_account_state = PersonState {
        is_initialized: true,
        first_name: "John".to_string(),
        last_name: "Borsh".to_string(),
    };
    pda_account_state.try_serialize(&mut &mut pda_account.data.borrow_mut()[..]);

    let (pda_descriptor, pda_descriptor_bump) =
        Pubkey::find_program_address(&[&PersonState::DISCRIMINATOR], &program_id);

    if pda_descriptor.ne(&pda_account_descriptor.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    // Assessing required lamports and creating transaction instruction
    let lamports_required = Rent::get()?.minimum_balance(PersonStateDescriptor::size());
    let create_pda_account_descriptor_ix = system_instruction::create_account(
        &funding_account.key,
        &pda_descriptor,
        lamports_required,
        PersonStateDescriptor::size().try_into().unwrap(),
        &program_id,
    );

    // Invoking the instruction but with PDAs as additional signer
    invoke_signed(
        &create_pda_account_descriptor_ix,
        &[
            funding_account.clone(),
            pda_account_descriptor.clone(),
            system_program.clone(),
        ],
        &[&[&PersonState::DISCRIMINATOR, &[pda_descriptor_bump]]],
    )?;

    // Setting state for PDA
    let pda_account_state_descriptor = PersonStateDescriptor::default();
    pda_account_state_descriptor
        .try_serialize(&mut &mut pda_account_descriptor.data.borrow_mut()[..]);

    Ok(())
}

pub fn deserialize_borsh_demo(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    // Getting required accounts
    let funding_account = next_account_info(accounts_iter)?;
    let pda_account = next_account_info(accounts_iter)?;
    let pda_account_descriptor = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let mut data = &pda_account.data.borrow_mut()[..];
    let person_state = PersonState::try_deserialize_unchecked(&mut data).unwrap();

    msg!("{:?} {:?}", person_state.first_name, person_state.last_name);
    Ok(())
}

pub fn instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = DemoInstruction::unpack(instruction_data)?;
    msg!("Instruction: {:?}", instruction);

    match instruction {
        DemoInstruction::InitializeCapnp { bump } => {
            initialize_capnp(program_id, accounts, &bump);
        }
        DemoInstruction::DeserializeCapnpDemo => {
            deserialize_capnpn_demo(program_id, accounts, instruction_data);
        }
        DemoInstruction::InitializeBorsh { bump } => {
            initialize_borsh(program_id, accounts, &bump);
        }
        DemoInstruction::DeserializeBorshDemo => {
            deserialize_borsh_demo(program_id, accounts, instruction_data);
        }
        _ => {}
    }

    Ok(())
}

