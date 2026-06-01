use clap::{Parser, Subcommand};
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::Instruction,
};
use solana_client::rpc_client::RpcClient;
use anchor_lang::{InstructionData, ToAccountMetas};

#[derive(Parser)]
#[command(name = "spl-rwa")]
#[command(about = "RWA Token CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    CreateToken {
        #[arg(short, long, default_value = "9")]
        decimals: u8,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::CreateToken { decimals } => {
            create_token(decimals);
        }
    }
}

fn create_token(decimals: u8) {
    // RPC Client
    let client = RpcClient::new("https://api.devnet.solana.com");

    // Payer — teri local wallet
    let payer = solana_sdk::signature::read_keypair_file(
        &*shellexpand::tilde("~/.config/solana/id.json")
    ).expect("Wallet keypair nahi mila");

    // Naya mint keypair
    let mint_keypair = Keypair::new();

    println!(
        "Creating token {} under program {}",
        mint_keypair.pubkey(),
        rwa_program::ID
    );

    // Accounts
    let accounts = rwa_program::accounts::CreateMint {
        mint: mint_keypair.pubkey(),
        payer: payer.pubkey(),
        system_program: anchor_lang::system_program::ID,
    }
    .to_account_metas(None);

    // Instruction
    let ix = Instruction {
        program_id: rwa_program::ID,
        accounts,
        data: rwa_program::instruction::CreateMint { decimals }.data(),
    };

    // Transaction
    let blockhash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &mint_keypair],
        blockhash,
    );

    // Send
    let sig = client.send_and_confirm_transaction(&tx).unwrap();

    println!("\nAddress:   {}", mint_keypair.pubkey());
    println!("Decimals:  {}", decimals);
    println!("\nSignature: {}", sig);
}