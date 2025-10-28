#[cfg(test)]

mod tests {
    use std::{path::PathBuf, vec};

    use litesvm::LiteSVM;
    use litesvm_token::{
        spl_token::{
            self,
            solana_program::{msg, rent::Rent, sysvar::SysvarId},
        },
        CreateAssociatedTokenAccount, CreateMint, MintTo,
    };
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::{
        get_associated_token_address,
        solana_program::{clock::Clock, program_pack::Pack},
    };

    use crate::constant::SECONDS_TO_DAYS;

    const PROGRAM_ID: &str = "BbFoDc7zsPk4QJLQmL6boWhc4HoGWbW8w4PPXGbdNfKL";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn setup() -> (
        LiteSVM,
        Keypair,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
    ) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 20 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/deploy/fundraiser.so");
        msg!("The path is!! {:?}", so_path);

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        svm.add_program(program_id(), &program_data);

        let mint = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint: {}", mint);

        let contributor_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint)
            .owner(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Contributor ATA: {}\n", contributor_ata);

        // Derive fundraiser PDA
        let (fundraiser, _) = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Fundraiser PDA: {}\n", fundraiser);

        // Derive vault PDA (ATA owned by escrow PDA)
        // let vault = spl_associated_token_account::get_associated_token_address(&fundraiser, &mint);
        // Create it client side to save on CUs in the program
        let vault = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint)
            .owner(&fundraiser)
            .send()
            .unwrap();
        msg!("Vault ATA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        // Return all important addresses
        (
            svm,
            payer,
            mint,
            contributor_ata,
            fundraiser,
            vault,
            associated_token_program,
            token_program,
            system_program,
        )
    }

    pub fn build_init_transaction(
        svm: &LiteSVM,
        payer: &Keypair,
        mint: Pubkey,
        vault: Pubkey,
        program_id: Pubkey,
        token_program: Pubkey,
        system_program: Pubkey,
        associated_token_program: Pubkey,
    ) -> Transaction {
        let (fundraiser, bump) = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        let amount_to_raise: u64 = 600 * 10u64.pow(6);

        let duration: u8 = 1;
        let init_data = [
            vec![0u8], // discriminator
            bump.to_le_bytes().to_vec(),
            amount_to_raise.to_le_bytes().to_vec(),
            duration.to_le_bytes().to_vec(),
        ]
        .concat();

        let init_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new(fundraiser, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(associated_token_program, false),
                AccountMeta::new_readonly(Rent::id(), false),
            ],
            data: init_data,
        };

        let message = Message::new(&[init_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        Transaction::new(&[payer], message, recent_blockhash)
    }

    pub fn build_contribute_transaction(
        mut svm: &mut LiteSVM,
        contributor: &Keypair,
        amount: u64,
        mint: Pubkey,
        contributor_ata: Pubkey,
        fundraiser: Pubkey,
        vault: Pubkey,
        program_id: Pubkey,
        token_program: Pubkey,
        system_program: Pubkey,
        associated_token_program: Pubkey,
    ) -> Transaction {
        // Derive contributor PDA
        let (contributor_pda, bump) = Pubkey::find_program_address(
            &[b"contributor", contributor.pubkey().as_ref()],
            &program_id,
        );

        MintTo::new(
            &mut svm,
            contributor,
            &mint,
            &contributor_ata,
            100000_000_000,
        )
        .send()
        .unwrap();

        // Instruction data layout:
        // [0] = discriminator (1 for contribute)
        // [1] = bump (u8)
        // [2..10] = amount (u64, LE)
        let contribute_data = [
            vec![1u8],                     // discriminator for "Contribute"
            vec![bump],                    // bump byte
            amount.to_le_bytes().to_vec(), // contribution amount
        ]
        .concat();

        // Build the contribute instruction
        let contribute_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true), // contributor (signer)
                AccountMeta::new(contributor_ata, false),     // contributor's token account
                AccountMeta::new(contributor_pda, false),     // contributor PDA
                AccountMeta::new_readonly(mint, false),       // mint
                AccountMeta::new(fundraiser, false),          // fundraiser state
                AccountMeta::new(vault, false),               // vault token account
                AccountMeta::new_readonly(system_program, false), // system program
                AccountMeta::new_readonly(token_program, false), // token program
                AccountMeta::new_readonly(associated_token_program, false), // associated token program
            ],
            data: contribute_data,
        };

        let message = Message::new(&[contribute_ix], Some(&contributor.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        Transaction::new(&[contributor], message, recent_blockhash)
    }

    pub fn build_refund_transaction(
        svm: &LiteSVM,
        contributor: &Keypair,
        mint: Pubkey,
        contributor_ata: Pubkey,
        fundraiser: Pubkey,
        vault: Pubkey,
        program_id: Pubkey,
        token_program: Pubkey,
        system_program: Pubkey,
        associated_token_program: Pubkey,
    ) -> Transaction {
        // Derive contributor PDA (same as in process_refund)
        let (contributor_pda, bump) = Pubkey::find_program_address(
            &[b"contributor", contributor.pubkey().as_ref()],
            &program_id,
        );

        // Instruction data layout:
        // [0] = discriminator (2 for refund)
        // [1] = bump (u8)
        let refund_data = [vec![2u8], vec![bump]].concat();

        // Build the refund instruction
        let refund_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true), // contributor (signer)
                AccountMeta::new(contributor_ata, false),     // contributor's ATA
                AccountMeta::new(contributor_pda, false),     // contributor PDA
                AccountMeta::new_readonly(mint, false),       // mint
                AccountMeta::new(fundraiser, false),          // fundraiser state
                AccountMeta::new(vault, false),               // vault
                AccountMeta::new_readonly(system_program, false), // system program
                AccountMeta::new_readonly(token_program, false), // token program
                AccountMeta::new_readonly(associated_token_program, false), // associated token program
            ],
            data: refund_data,
        };

        let message = Message::new(&[refund_ix], Some(&contributor.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        Transaction::new(&[contributor], message, recent_blockhash)
    }

    pub fn build_collect_transaction(
        svm: &LiteSVM,
        maker: &Keypair,
        mint: Pubkey,
        maker_ata: Pubkey,
        vault: Pubkey,
        program_id: Pubkey,
        token_program: Pubkey,
        system_program: Pubkey,
        associated_token_program: Pubkey,
    ) -> Transaction {
        // Derive fundraiser PDA (same seed pattern as process_collect)
        let (fundraiser_pda, bump) =
            Pubkey::find_program_address(&[b"fundraiser", maker.pubkey().as_ref()], &program_id);

        // Instruction data layout:
        // [0] = discriminator (3 for Collect)
        // [1] = bump (u8)
        let collect_data = [vec![3u8], vec![bump]].concat();

        // Build the collect instruction
        let collect_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(maker.pubkey(), true),  // maker (signer)
                AccountMeta::new(maker_ata, false),      // maker's ATA
                AccountMeta::new_readonly(mint, false),  // mint
                AccountMeta::new(fundraiser_pda, false), // fundraiser PDA
                AccountMeta::new(vault, false),          // vault
                AccountMeta::new_readonly(system_program, false), // system program
                AccountMeta::new_readonly(token_program, false), // token program
                AccountMeta::new_readonly(associated_token_program, false), // associated token program
            ],
            data: collect_data,
        };

        let message = Message::new(&[collect_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        Transaction::new(&[maker], message, recent_blockhash)
    }

    #[test]
    pub fn test_init_instruction() {
        let (
            mut svm,
            payer,
            mint,
            _contributor_ata,
            _fundraiser,
            vault,
            associated_token_program,
            token_program,
            system_program,
        ) = setup();

        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let transaction = build_init_transaction(
            &svm,
            &payer,
            mint,
            vault,
            program_id,
            token_program,
            system_program,
            associated_token_program,
        );

        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send init tx");

        msg!("\n\n Init transaction sucessfull");
        msg!("Logs: {}", tx.pretty_logs());
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }

    #[test]
    pub fn test_contribute_instruction() {
        let (
            mut svm,
            payer,
            mint,
            contributor_ata,
            fundraiser,
            vault,
            associated_token_program,
            token_program,
            system_program,
        ) = setup();

        let program_id = program_id();
        let amount: u64 = 1_000_000; // just enough

        let transaction1 = build_init_transaction(
            &svm,
            &payer,
            mint,
            vault,
            program_id,
            token_program,
            system_program,
            associated_token_program,
        );

        let _tx = svm
            .send_transaction(transaction1)
            .expect("Failed to send init tx");

        let transaction = build_contribute_transaction(
            &mut svm,
            &payer,
            amount,
            mint,
            contributor_ata,
            fundraiser,
            vault,
            program_id,
            token_program,
            system_program,
            associated_token_program,
        );

        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send contribute tx");

        let vault_acc = svm.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_acc.data).unwrap();

        msg!("Amount in the vault: {}", vault_data.amount);
        msg!("Amount deposited by contributor: {}", amount);
        assert_eq!(vault_data.amount, amount);

        msg!("\n\n Contribute transaction sucessfull");
        msg!("Logs: {}", tx.pretty_logs());
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }

    #[test]
    pub fn test_refund_instruction() {
        let (
            mut svm,
            payer,
            mint,
            contributor_ata,
            fundraiser,
            vault,
            associated_token_program,
            token_program,
            system_program,
        ) = setup();

        let program_id = program_id();

        let transaction1 = build_init_transaction(
            &svm,
            &payer,
            mint,
            vault,
            program_id,
            token_program,
            system_program,
            associated_token_program,
        );

        let _tx1 = svm
            .send_transaction(transaction1)
            .expect("Failed to send init tx");

        let amount: u64 = 1_000_000; // just enough

        let transaction2 = build_contribute_transaction(
            &mut svm,
            &payer,
            amount,
            mint,
            contributor_ata,
            fundraiser,
            vault,
            program_id,
            token_program,
            system_program,
            associated_token_program,
        );

        let _tx2 = svm
            .send_transaction(transaction2)
            .expect("Failed to send contribute tx");

        let transaction = build_refund_transaction(
            &svm,
            &payer,
            mint,
            contributor_ata,
            fundraiser,
            vault,
            program_id,
            token_program,
            system_program,
            associated_token_program,
        );

        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send refund tx");

        msg!("\n\n Refund transaction sucessfull");
        msg!("Logs: {}", tx.pretty_logs());
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }

    #[test]
    pub fn test_collect_instruction() {
        let (
            mut svm,
            payer,
            mint,
            contributor_ata,
            fundraiser,
            vault,
            associated_token_program,
            token_program,
            system_program,
        ) = setup();

        let program_id = program_id();
        let maker_ata = get_associated_token_address(&payer.pubkey(), &mint);

        let transaction1 = build_init_transaction(
            &svm,
            &payer,
            mint,
            vault,
            program_id,
            token_program,
            system_program,
            associated_token_program,
        );

        let _tx1 = svm
            .send_transaction(transaction1)
            .expect("Failed to send init tx");

        let amount: u64 = 400_000_000; // just enough

        let transaction2 = build_contribute_transaction(
            &mut svm,
            &payer,
            amount,
            mint,
            contributor_ata,
            fundraiser,
            vault,
            program_id,
            token_program,
            system_program,
            associated_token_program,
        );
        let _tx2 = svm
            .send_transaction(transaction2)
            .expect("Failed to send contribute tx");

        let contributor2 = Keypair::new();
        let contributor2_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint)
            .owner(&contributor2.pubkey())
            .send()
            .expect("Failed to create ata for contributor 2!");

        svm.airdrop(&contributor2.pubkey(), 10 * LAMPORTS_PER_SOL)
            .unwrap();

        MintTo::new(&mut svm, &payer, &mint, &contributor2_ata, amount)
            .send()
            .expect("Failed to mint tokens to contributor 2!");

        let (contributor_pda, bump) = Pubkey::find_program_address(
            &[b"contributor", contributor2.pubkey().as_ref()],
            &program_id,
        );

        // Instruction data layout:
        // [0] = discriminator (1 for contribute)
        // [1] = bump (u8)
        // [2..10] = amount (u64, LE)
        let contribute_data = [
            vec![1u8],                     // discriminator for "Contribute"
            vec![bump],                    // bump byte
            amount.to_le_bytes().to_vec(), // contribution amount
        ]
        .concat();

        // Build the contribute instruction
        let contribute_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(contributor2.pubkey(), true), // contributor (signer)
                AccountMeta::new(contributor2_ata, false),     // contributor's token account
                AccountMeta::new(contributor_pda, false),      // contributor PDA
                AccountMeta::new_readonly(mint, false),        // mint
                AccountMeta::new(fundraiser, false),           // fundraiser state
                AccountMeta::new(vault, false),                // vault token account
                AccountMeta::new_readonly(system_program, false), // system program
                AccountMeta::new_readonly(token_program, false), // token program
                AccountMeta::new_readonly(associated_token_program, false), // associated token program
            ],
            data: contribute_data,
        };

        let message = Message::new(&[contribute_ix], Some(&contributor2.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let transaction3 = Transaction::new(&[contributor2], message, recent_blockhash);
        let _tx3 = svm
            .send_transaction(transaction3)
            .expect("Failed to send contribute 2 tx");

        let transaction = build_collect_transaction(
            &svm,
            &payer,
            mint,
            maker_ata,
            vault,
            program_id,
            token_program,
            system_program,
            associated_token_program,
        );

        // time trave 2 days into future, duration is 1 day

        let mut clock = svm.get_sysvar::<Clock>();
        clock.unix_timestamp += 2 * SECONDS_TO_DAYS as i64;
        svm.set_sysvar::<Clock>(&clock);

        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send collect tx");

        msg!("\n\n Collect transaction sucessfull");
        msg!("Logs: {}", tx.pretty_logs());
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }
}
