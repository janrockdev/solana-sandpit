use borsh::{BorshDeserialize, BorshSerialize};
use crossbeam::channel;
use solana::solana_service_server::{SolanaService, SolanaServiceServer};
use solana::{
    AirdropRequest, AirdropResponse, BalanceRequest, BalanceResponse, CreateWalletRequest,
    CreateWalletResponse, SendSolRequest, SendSolResponse, GreetRequest, GreetResponse};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{bs58, system_instruction };
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::transfer,
    transaction::Transaction,
};
use solana_program::instruction::Instruction;
use std::str::FromStr;
use tokio::task;
use tonic::{transport::Server, Request, Response, Status};

pub mod solana {
    tonic::include_proto!("solana");
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct GreetingAccount {
    pub counter: u32,
}

impl Default for GreetingAccount {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

#[derive(Debug, Default)]
pub struct MySolanaService;

#[tonic::async_trait]
impl SolanaService for MySolanaService {
    async fn get_balance(
        &self,
        request: Request<BalanceRequest>,
    ) -> Result<Response<BalanceResponse>, Status> {
        let BalanceRequest {
            network,
            wallet_address,
        } = request.into_inner();
        let (sender, receiver) = channel::unbounded();

        // Spawn a new thread to handle the RPC call
        task::spawn_blocking(move || {
            let rpc_url = match network.to_string().as_str() {
                "devnet" => "https://api.devnet.solana.com",
                "testnet" => "https://api.testnet.solana.com",
                "mainnet" => "https://api.mainnet-beta.solana.com",
                _ => {
                    eprintln!("Invalid network. Use 'devnet', 'testnet', or 'mainnet'.");
                    std::process::exit(1);
                }
            };
            let client = RpcClient::new(rpc_url);
            let pubkey = Pubkey::from_str(&wallet_address).expect("Invalid public key");

            match client.get_balance(&pubkey) {
                Ok(balance) => {
                    sender.send(Ok(balance)).unwrap();
                }
                Err(err) => {
                    sender
                        .send(Err(Status::internal(format!(
                            "Failed to get balance: {}",
                            err
                        ))))
                        .unwrap();
                }
            }
        });

        let balance = receiver.recv().unwrap()?;
        let response = BalanceResponse { balance };

        Ok(Response::new(response))
    }

    async fn create_wallet(
        &self,
        _request: Request<CreateWalletRequest>,
    ) -> Result<Response<CreateWalletResponse>, Status> {
        // Generate a new keypair
        let keypair = Keypair::new();
        let public_key = keypair.pubkey().to_string();
        let secret_key = bs58::encode(keypair.to_bytes()).into_string();

        let response = CreateWalletResponse {
            public_key,
            secret_key,
        };
        Ok(Response::new(response))
    }

    async fn request_airdrop(
        &self,
        request: Request<AirdropRequest>,
    ) -> Result<Response<AirdropResponse>, Status> {
        let AirdropRequest {
            network,
            wallet_address,
            amount,
        } = request.into_inner();
        let (sender, receiver) = channel::unbounded();

        // Determine the actual RPC URL based on the network identifier
        let rpc_url = match network.as_str() {
            "devnet" => "https://api.devnet.solana.com",
            "testnet" => "https://api.testnet.solana.com",
            "mainnet" => {
                return Err(Status::invalid_argument(
                    "Faucet is not available on mainnet.",
                ));
            }
            _ => {
                return Err(Status::invalid_argument("Invalid network identifier."));
            }
        };

        // Spawn a new thread to handle the RPC call
        task::spawn_blocking(move || {
            let client = RpcClient::new(rpc_url.to_string());
            let pubkey = Pubkey::from_str(&wallet_address).expect("Invalid public key");

            match client.request_airdrop(&pubkey, amount) {
                Ok(signature) => {
                    sender.send(Ok(signature)).unwrap();
                }
                Err(err) => {
                    sender
                        .send(Err(Status::internal(format!(
                            "Failed to request airdrop: {}",
                            err
                        ))))
                        .unwrap();
                }
            }
        });

        let signature = receiver.recv().unwrap()?;
        let response = AirdropResponse {
            signature: signature.to_string(),
        };

        Ok(Response::new(response))
    }

    async fn send_sol(
        &self,
        request: Request<SendSolRequest>,
    ) -> Result<Response<SendSolResponse>, Status> {
        let SendSolRequest {
            from_address,
            to_address,
            amount,
            rpc_url,
            from_secret_key,
        } = request.into_inner();
        let (sender, receiver) = channel::unbounded();

        // Determine the actual RPC URL based on the network identifier
        let rpc_url = match rpc_url.as_str() {
            "devnet" => "https://api.devnet.solana.com",
            "testnet" => "https://api.testnet.solana.com",
            "mainnet" => "https://api.mainnet-beta.solana.com",
            _ => {
                return Err(Status::invalid_argument("Invalid network identifier."));
            }
        };

        // Spawn a new thread to handle the RPC call
        task::spawn_blocking(move || {
            let client = RpcClient::new(rpc_url.to_string());
            let from_pubkey = Pubkey::from_str(&from_address).expect("Invalid from address");
            let to_pubkey = Pubkey::from_str(&to_address).expect("Invalid to address");

            let from_keypair_bytes = bs58::decode(from_secret_key)
                .into_vec()
                .expect("Invalid secret key");
            let from_keypair = Keypair::from_bytes(&from_keypair_bytes).expect("Invalid keypair");

            let blockhash = client
                .get_latest_blockhash()
                .expect("Failed to get latest blockhash");
            let tx = Transaction::new_signed_with_payer(
                &[transfer(&from_pubkey, &to_pubkey, amount)],
                Some(&from_pubkey),
                &[&from_keypair],
                blockhash,
            );

            match client.send_and_confirm_transaction(&tx) {
                Ok(signature) => {
                    sender.send(Ok(signature)).unwrap();
                }
                Err(err) => {
                    sender
                        .send(Err(Status::internal(format!(
                            "Failed to send SOL: {}",
                            err
                        ))))
                        .unwrap();
                }
            }
        });

        let signature = receiver.recv().unwrap()?;
        let response = SendSolResponse {
            signature: signature.to_string(),
        };

        Ok(Response::new(response))
    }

    async fn greet(&self, request: Request<GreetRequest>) -> Result<Response<GreetResponse>, Status> {
        let GreetRequest {
            network,
            payer_secret_key,
            seed,
        } = request.into_inner();
        //let (sender, receiver) = channel::unbounded();

        //task::spawn_blocking(move || {
            let rpc_url = match network.as_str() {
                "devnet" => "https://api.devnet.solana.com",
                "testnet" => "https://api.testnet.solana.com",
                "mainnet" => "https://api.mainnet-beta.solana.com",
                _ => {
                    return Err(Status::invalid_argument("Invalid network identifier."));
                }
            };

            //establish connection to the network
            let client = RpcClient::new(rpc_url);
            let version = client.get_version().unwrap();
            println!("Connection to cluster established to: {}, version: {}", &rpc_url.to_string(), version);

            //converting the secret key to a keypair.pubkey and searching for the balance
            let payer = Keypair::from_bytes(&bs58::decode(payer_secret_key).into_vec().unwrap()).unwrap();
            let lamports = client.get_balance(&payer.pubkey()).unwrap();
            println!("Balance of payer({}): {}", payer.pubkey(), lamports);

            //searching program accounts connected with the program_pubkey
            let program_pubkey = Pubkey::from_str("D36yRZ6n8AwhhStGRJQvjZL78nx5DP2qR3CtqraQuLJF").unwrap();
            let greeted_account = client.get_program_accounts(&program_pubkey).unwrap();
            println!("Program accounts: {:?}", greeted_account);

            //creating a new account with the program_pubkey with seed "cauves!"
            let greeted_pubkey = Pubkey::create_with_seed(&payer.pubkey(), &seed, &program_pubkey).unwrap();
            println!("Greeted pubkey: {}", greeted_pubkey);

            let instruction = Instruction {
                program_id: program_pubkey,
                accounts: vec![
                    solana_program::instruction::AccountMeta::new(greeted_pubkey, false),
                    solana_program::instruction::AccountMeta::new_readonly(payer.pubkey(), true),
                ],
                data: vec![], // No additional data needed for this instruction
            };
            println!("Instruction: {:?}", instruction);

            // //if the account does not exist, create a new account
            if greeted_account.iter().find(|x: &&(Pubkey, solana_sdk::account::Account)| x.0 == greeted_pubkey).is_none() {

                let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
                    &[system_instruction::create_account_with_seed(
                        &payer.pubkey(),
                        &greeted_pubkey,
                        &payer.pubkey(),
                        &seed,
                        10_000_000,
                        std::mem::size_of::<GreetingAccount>() as u64,
                        &program_pubkey,
                    )],
                    Some(&payer.pubkey()),
                    &[&payer],
                    client.get_latest_blockhash().unwrap(),
                );

                let signature = client.send_and_confirm_transaction(&transaction).unwrap();

                println!("Signature: {}", signature);
                                
            let program_pubkey = Pubkey::from_str("D36yRZ6n8AwhhStGRJQvjZL78nx5DP2qR3CtqraQuLJF").unwrap();
            let greeted_account = client.get_program_accounts(&program_pubkey).unwrap();
                println!("Program accounts: {:?}", greeted_account);
            } else {
                println!("Account {} already exists. Try different seed.", greeted_pubkey);
            }

            let recent_blockhash = client.get_latest_blockhash().unwrap();
            let transaction = Transaction::new_signed_with_payer(
                &[instruction],
                Some(&payer.pubkey()),
                &[&payer],
                recent_blockhash,
            );

            let signature = client.send_and_confirm_transaction(&transaction).unwrap();
            println!("Signature: {}", signature);


            let account_info = client.get_account(&greeted_pubkey).unwrap();
            println!("Account Info: {:?}", account_info);
            
            let response = GreetResponse { signature: format!("{:?}", account_info) };

            println!("Report: {:?}", report_greetings(&client, &greeted_pubkey).await);
            
            Ok(Response::new(response))
    }
}

async fn report_greetings(client: &RpcClient, greeted_pubkey: &Pubkey) -> Result<(), Box<dyn std::error::Error>> {
    let account_info = client.get_account(greeted_pubkey)?;
    println!("Account Info Size: {:?}", account_info.data.len());

    if account_info.lamports == 0 {
        return Err("Error: cannot find the greeted account".into());
    }

    if account_info.data.is_empty() {
        return Err("Error: account data is empty".into());
    }

    // Ensure that the data length matches the expected length for GreetingAccount
    if account_info.data.len() != std::mem::size_of::<GreetingAccount>() {
        return Err("Error: account data length mismatch".into());
    }

    let greeting = GreetingAccount::try_from_slice(&account_info.data)?;
    println!(
        "{} has been greeted {} time(s)",
        greeted_pubkey.to_string(),
        greeting.counter
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let solana_service = MySolanaService::default();

    println!("SolanaServiceServer listening on {}", addr);

    Server::builder()
        .add_service(SolanaServiceServer::new(solana_service))
        .serve(addr)
        .await?;

    Ok(())
}