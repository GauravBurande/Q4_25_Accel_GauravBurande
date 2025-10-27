#[cfg(test)]

mod tests {
    use std::{path::PathBuf, vec};

    use litesvm::LiteSVM;
    use litesvm_token::{
        spl_token::{
            self,
            solana_program::{msg, rent::Rent, sysvar::SysvarId},
        },
        CreateAssociatedTokenAccount, CreateMint,
    };
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

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

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
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
        msg!("Contributor ATA A: {}\n", contributor_ata);

        // Derive fundraiser PDA
        let (fundraiser, _) = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Fundraiser PDA: {}\n", fundraiser);

        // Derive vault PDA (ATA owned by escrow PDA)
        let vault = spl_associated_token_account::get_associated_token_address(&fundraiser, &mint);
        msg!("Vault PDA: {}\n", vault);

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

        let (fundraiser, bump) = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        let amount_to_raise: u64 = 600u64.pow(6);
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
        let transaction = Transaction::new(&[payer], message, recent_blockhash);

        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send init tx");

        msg!("\n\n Init transaction sucessfull");
        msg!("Logs: {}", tx.pretty_logs());
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }
}
