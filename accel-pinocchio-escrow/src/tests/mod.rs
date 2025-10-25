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

    const PROGRAM_ID: &str = "4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT";
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
        Pubkey,
    ) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file
        let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/deploy/escrow.so");
        msg!("The path is!! {:?}", so_path);

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        svm.add_program(program_id(), &program_data);

        // Create mints
        let mint_a = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint A: {}", mint_a);

        let mint_b = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint B: {}", mint_b);

        // Create maker ATA for Mint A
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
            .owner(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        // Derive escrow PDA
        let (escrow, _) = Pubkey::find_program_address(
            &[b"escrow".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Escrow PDA: {}\n", escrow);

        // Derive vault PDA (ATA owned by escrow PDA)
        let vault = spl_associated_token_account::get_associated_token_address(&escrow, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        // Return all important addresses
        (
            svm,
            payer,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program,
            token_program,
            system_program,
        )
    }

    fn build_make_instruction(
        svm: &LiteSVM, // pass by ref — no need to move svm
        payer: &Keypair,
        bump: u8,
        mint_a: Pubkey,
        mint_b: Pubkey,
        escrow: Pubkey,
        maker_ata_a: Pubkey,
        vault: Pubkey,
        system_program: Pubkey,
        token_program: Pubkey,
        associated_token_program: Pubkey,
    ) -> Transaction {
        let program_id = program_id(); // assuming you're in an Anchor context

        let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
        let amount_to_give: u64 = 500000000; // 500 tokens with 6 decimal places

        // Instruction data layout:
        // [ discriminator (u8) | bump (u8) | amount_to_receive (u64) | amount_to_give (u64) ]
        let make_data = [
            vec![0u8], // discriminator for "Make"
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();

        let make_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(mint_a, false),
                AccountMeta::new_readonly(mint_b, false),
                AccountMeta::new(escrow, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(associated_token_program, false),
                AccountMeta::new_readonly(Rent::id(), false),
            ],
            data: make_data,
        };

        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        Transaction::new(&[payer], message, recent_blockhash)
    }

    #[test]
    pub fn test_make_instruction() {
        let (
            mut svm,
            payer,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program,
            token_program,
            system_program,
        ) = setup();

        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let (escrow_pda, bump) = Pubkey::find_program_address(
            &[b"escrow".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Escrow PDA: {}\n", escrow);

        msg!("Bump: {}", bump);

        let transaction = build_make_instruction(
            &svm,
            &payer,
            bump,
            mint_a,
            mint_b,
            escrow_pda,
            maker_ata_a,
            vault,
            system_program,
            token_program,
            associated_token_program,
        );

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }

    #[test]
    pub fn test_cancel_instruction() {
        let (
            mut svm,
            payer,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program,
            token_program,
            system_program,
        ) = setup();

        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let (escrow_pda, bump) = Pubkey::find_program_address(
            &[b"escrow".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Escrow PDA: {}\n", escrow);

        msg!("Bump: {}", bump);

        let transaction1 = build_make_instruction(
            &svm,
            &payer,
            bump,
            mint_a,
            mint_b,
            escrow_pda,
            maker_ata_a,
            vault,
            system_program,
            token_program,
            associated_token_program,
        );

        // Send the transaction and capture the result
        let _tx1 = svm.send_transaction(transaction1).unwrap();

        let cancel_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(mint_a, false),
                AccountMeta::new(escrow, false),
                AccountMeta::new(maker_ata_a, false),
                AccountMeta::new(vault, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(associated_token_program, false),
            ],
            data: vec![2u8],
        };

        let message = Message::new(&[cancel_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send cancel txn");

        // Log transaction details
        msg!("\n\n Cancel transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
    }
}
