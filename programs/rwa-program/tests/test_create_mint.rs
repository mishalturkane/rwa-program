#[cfg(test)]
mod tests {
    use litesvm::LiteSVM;
    use solana_sdk::{
        signature::{Keypair, Signer},
        transaction::Transaction,
        instruction::Instruction,
    };
    use anchor_lang::{InstructionData, ToAccountMetas};

    #[test]
    fn test_create_mint() {
        // LiteSVM setup
        let mut svm = LiteSVM::new();

        // Program load karo
        svm.add_program_from_file(
            rwa_program::ID,
            "../../target/deploy/rwa_program.so",
        ).unwrap();

        // Keypairs
        let payer = Keypair::new();
        let mint_keypair = Keypair::new();

        // Payer ko SOL do
        svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

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
            data: rwa_program::instruction::CreateMint { decimals: 9 }.data(),
        };

        // Transaction
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &mint_keypair],
            blockhash,
        );

        // Execute
        let result = svm.send_transaction(tx);
        assert!(result.is_ok(), "create_mint failed: {:?}", result.err());

        // Verify
        let mint_account = svm
            .get_account(&mint_keypair.pubkey())
            .expect("Mint account nahi mila");

        println!("✅ Mint created!");
        println!("   Address  : {}", mint_keypair.pubkey());
        println!("   Lamports : {}", mint_account.lamports);
        println!("   Owner    : {}", mint_account.owner);

        // Owner tera RWA program hona chahiye
        assert_eq!(mint_account.owner, rwa_program::ID);
    }
}