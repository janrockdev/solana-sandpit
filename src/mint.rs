use solana_program::{
    pubkey::Pubkey,
    system_instruction,
    system_program,
    program_pack::Pack,
    instruction::Instruction,
};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport::TransportError,
};
use solana_client::rpc_client::RpcClient;
use metaplex_token_metadata::{
    id as metaplex_program_id,
    instruction::{create_metadata_accounts, create_master_edition},
    state::{Creator, Data},
};
use std::fs::File;
use std::io::Read;

pub struct NFT {
    pub mint: Pubkey,
    pub token_account: Pubkey,
    pub metadata_account: Pubkey,
    pub edition_account: Pubkey,
}

pub fn mint_nft(
    client: &RpcClient,
    payer: &Keypair,
    mint: &Keypair,
    uri: String,
    name: String,
    symbol: String,
    creators: Option<Vec<Creator>>,
    seller_fee_basis_points: u16,
) -> Result<NFT, TransportError> {
    let mint_rent = client.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?;
    let token_rent = client.get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)?;
    let mint_pubkey = mint.pubkey();

    let token_account = Keypair::new();
    let metadata_account = Pubkey::find_program_address(
        &[
            b"metadata",
            &metaplex_program_id().to_bytes(),
            &mint_pubkey.to_bytes(),
        ],
        &metaplex_program_id(),
    ).0;

    let edition_account = Pubkey::find_program_address(
        &[
            b"metadata",
            &metaplex_program_id().to_bytes(),
            &mint_pubkey.to_bytes(),
            b"edition",
        ],
        &metaplex_program_id(),
    ).0;

    let create_mint_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint_pubkey,
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
    );

    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &payer.pubkey(),
        Some(&payer.pubkey()),
        0,
    )?;

    let create_token_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &token_account.pubkey(),
        token_rent,
        spl_token::state::Account::LEN as u64,
        &spl_token::id(),
    );

    let init_token_account_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        &token_account.pubkey(),
        &mint_pubkey,
        &payer.pubkey(),
    )?;

    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_pubkey,
        &token_account.pubkey(),
        &payer.pubkey(),
        &[],
        1,
    )?;

    let create_metadata_account_ix = create_metadata_accounts(
        metaplex_program_id(),
        metadata_account,
        mint_pubkey,
        payer.pubkey(),
        payer.pubkey(),
        payer.pubkey(),
        name,
        symbol,
        uri,
        creators,
        seller_fee_basis_points,
        true,
        false,
    );

    let create_master_edition_ix = create_master_edition(
        metaplex_program_id(),
        edition_account,
        mint_pubkey,
        payer.pubkey(),
        payer.pubkey(),
        metadata_account,
        payer.pubkey(),
        Some(0),
    );

    let instructions = vec![
        create_mint_account_ix,
        init_mint_ix,
        create_token_account_ix,
        init_token_account_ix,
        mint_to_ix,
        create_metadata_account_ix,
        create_master_edition_ix,
    ];

    let recent_blockhash = client.get_latest_blockhash()?;
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    transaction.sign(&[payer, mint, &token_account], recent_blockhash);

    client.send_and_confirm_transaction(&transaction)?;

    Ok(NFT {
        mint: mint_pubkey,
        token_account: token_account.pubkey(),
        metadata_account,
        edition_account,
    })
}

pub fn upload_file_to_arweave(filepath: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(filepath)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let url = arweave::upload(buffer)?;
    Ok(url)
}
