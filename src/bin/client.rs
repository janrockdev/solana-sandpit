use solana::solana_service_client::SolanaServiceClient;
use solana::{AirdropRequest, BalanceRequest, CreateWalletRequest, SendSolRequest, GreetRequest
};
use std::env;
use std::fs::File;
use std::io::Write;

pub mod solana {
    tonic::include_proto!("solana");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <command> [<args>]", args[0]);
        std::process::exit(1);
    }
    let command = &args[1];
    let mut client = SolanaServiceClient::connect("http://[::1]:50051").await?;
    match command.as_str() {
        "get-balance" => {
            if args.len() != 4 {
                eprintln!("Usage: {} get_balance <network> <wallet-address>", args[0]);
                std::process::exit(1);
            }
            let network = args[2].as_str();
            let wallet_address = args[3].clone();
            let network = match network {
                "devnet" => "devnet",
                "testnet" => "testnet",
                "mainnet" => "mainnet",
                _ => {
                    eprintln!("Invalid network. Use 'devnet', 'testnet', or 'mainnet'.");
                    std::process::exit(1);
                }
            };
            let request = tonic::Request::new(BalanceRequest {  network: network.to_string(), wallet_address });
            let response = client.get_balance(request).await?;
            println!("Wallet balance: {} lamports", response.into_inner().balance);
        },
        "create-wallet" => {
            let request = tonic::Request::new(CreateWalletRequest {});
            let response = client.create_wallet(request).await?;
            let response = response.into_inner();
            println!("New wallet created:");
            println!("Public Key: {}", response.public_key);
            println!("Secret Key: {}", response.secret_key);

            // Save the credentials to a local file named after the public key
            let filename = format!("{}.txt", response.public_key);
            let mut file = File::create(&filename)?;
            writeln!(file, "Public Key: {}", response.public_key)?;
            writeln!(file, "Secret Key: {}", response.secret_key)?;
            println!("Credentials saved to {}", filename);
        },
        "request-airdrop" => {
            if args.len() != 5 {
                eprintln!("Usage: {} request-airdrop <network> <wallet-address> <amount>", args[0]);
                std::process::exit(1);
            }
            let network = args[2].as_str();
            let wallet_address = args[3].clone();
            let amount: u64 = args[4].parse().expect("Invalid amount");
            let network = match network {
                "devnet" => "devnet",
                "testnet" => "testnet",
                "mainnet" => {
                    eprintln!("Faucet is not available on mainnet.");
                    std::process::exit(1);
                }
                _ => {
                    eprintln!("Invalid network. Use 'devnet', 'testnet', or 'mainnet'.");
                    std::process::exit(1);
                }
            };

            let request = tonic::Request::new(AirdropRequest { network: network.to_string(), wallet_address, amount });
            let response = client.request_airdrop(request).await?;
            println!("Airdrop requested. Transaction signature: {}", response.into_inner().signature);
        },
        "send-sol" => {
            if args.len() != 7 {
                eprintln!("Usage: {} send-sol <network> <from-address> <to-address> <amount> <from-secret-key>", args[0]);
                std::process::exit(1);
            }
            let network = args[2].as_str();
            let from_address = args[3].clone();
            let to_address = args[4].clone();
            let amount: u64 = args[5].parse().expect("Invalid amount");
            let from_secret_key = args[6].clone();
            let network = match network {
                "devnet" => "devnet",
                "testnet" => "testnet",
                "mainnet" => "mainnet",
                _ => {
                    eprintln!("Invalid network. Use 'devnet', 'testnet', or 'mainnet'.");
                    std::process::exit(1);
                }
            };

            let request = tonic::Request::new(SendSolRequest {
                from_address,
                to_address,
                amount,
                rpc_url: network.to_string(),
                from_secret_key,
            });
            let response = client.send_sol(request).await?;
            println!("SOL sent. Transaction signature: {}", response.into_inner().signature);
        },
        "greet" => {
            if args.len() != 5 {
                eprintln!("Usage: {} greet <network> <payer-secret-key> <seed>", args[0]);
                std::process::exit(1);
            }
            let network: &str = args[2].as_str();
            let payer_secret_key = args[3].clone();
            let seed: String = args[4].clone();
            let network = match network {
                "devnet" => "devnet",
                "testnet" => "testnet",
                "mainnet" => "mainnet",
                _ => {
                    eprintln!("Invalid network. Use 'devnet', 'testnet', or 'mainnet'.");
                    std::process::exit(1);
                }
            };

            let request = tonic::Request::new(GreetRequest {
                network: network.to_string(),
                payer_secret_key,
                seed,
            });
            let response = client.greet(request).await?;
            println!("Greet transaction signature: {}", response.into_inner().signature);
        },
        _ => {
            eprintln!("Invalid command. Use 'get-balance', 'create-wallet', 'request-airdrop', 'send-sol' or greet.");
            std::process::exit(1);
        },
    }

    Ok(())
}