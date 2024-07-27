use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    program::invoke,
    program_pack::Pack,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    state::Mint,
};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct BorshPubkey([u8; 32]);

impl From<Pubkey> for BorshPubkey {
    fn from(pubkey: Pubkey) -> Self {
        BorshPubkey(pubkey.to_bytes())
    }
}

impl From<BorshPubkey> for Pubkey {
    fn from(borsh_pubkey: BorshPubkey) -> Self {
        Pubkey::new_from_array(borsh_pubkey.0)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum MarketplaceInstruction {
    InitAccount,
    List { price: u64 },
    Buy,
    Withdraw,
    MintNft,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MarketplaceAccount {
    pub is_initialized: bool,
    pub owner: BorshPubkey,
    pub price: u64,
}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MarketplaceInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        MarketplaceInstruction::InitAccount => init_account(program_id, accounts),
        MarketplaceInstruction::List { price } => list_item(program_id, accounts, price),
        MarketplaceInstruction::Buy => buy_item(program_id, accounts),
        MarketplaceInstruction::Withdraw => withdraw_item(program_id, accounts),
        MarketplaceInstruction::MintNft => mint_nft(program_id, accounts),
    }
}

fn init_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let account = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;

    let mut marketplace_account = MarketplaceAccount::try_from_slice(&account.data.borrow())?;
    marketplace_account.is_initialized = true;
    marketplace_account.owner = (*owner.key).into();

    marketplace_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    Ok(())
}

fn list_item(program_id: &Pubkey, accounts: &[AccountInfo], price: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let account = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut marketplace_account = MarketplaceAccount::try_from_slice(&account.data.borrow())?;

    if Pubkey::from(marketplace_account.owner.clone()) != *owner.key {
        return Err(ProgramError::IllegalOwner);
    }

    marketplace_account.price = price;
    marketplace_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    Ok(())
}

fn buy_item(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let buyer = next_account_info(account_info_iter)?;
    let account = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;

    let mut marketplace_account = MarketplaceAccount::try_from_slice(&account.data.borrow())?;

    if Pubkey::from(marketplace_account.owner.clone()) != *owner.key {
        return Err(ProgramError::IllegalOwner);
    }

    let price = marketplace_account.price;

    invoke(
        &system_instruction::transfer(buyer.key, owner.key, price),
        &[buyer.clone(), owner.clone()],
    )?;

    marketplace_account.owner = (*buyer.key).into();
    marketplace_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    Ok(())
}

fn withdraw_item(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let account = next_account_info(account_info_iter)?;
    let owner = next_account_info(account_info_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let marketplace_account = MarketplaceAccount::try_from_slice(&account.data.borrow())?;

    if Pubkey::from(marketplace_account.owner.clone()) != *owner.key {
        return Err(ProgramError::IllegalOwner);
    }

    marketplace_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    Ok(())
}

fn mint_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let mint_account = next_account_info(account_info_iter)?;
    let mint_authority = next_account_info(account_info_iter)?;
    let payer = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let rent_sysvar_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    let rent = &Rent::from_account_info(rent_sysvar_account)?;

    invoke(
        &system_instruction::create_account(
            payer.key,
            mint_account.key,
            rent.minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            token_program.key,
        ),
        &[payer.clone(), mint_account.clone()],
    )?;

    invoke(
        &initialize_mint(
            token_program.key,
            mint_account.key,
            mint_authority.key,
            None,
            0,
        )?,
        &[mint_account.clone(), rent_sysvar_account.clone(), token_program.clone()],
    )?;

    invoke(
        &mint_to(
            token_program.key,
            mint_account.key,
            token_account.key,
            mint_authority.key,
            &[],
            1,
        )?,
        &[
            mint_account.clone(),
            token_account.clone(),
            mint_authority.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}
