#[cfg(test)]
mod tests {
    use {
        anchor_lang::{
            prelude::{msg, Clock},
            solana_program::program_pack::Pack,
            AccountDeserialize, InstructionData, ToAccountMetas,
        },
        anchor_spl::{
            associated_token::{self, spl_associated_token_account},
            token::spl_token,
        },
        litesvm::LiteSVM,
        litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
        },
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::path::PathBuf,
    };
    pub struct EscrowTestEnvironment {
        pub program: LiteSVM,
        pub maker: Keypair,
        pub taker: Keypair,
        pub mint_b: Pubkey,
        pub mint_a: Pubkey,
        pub maker_ata_a: Pubkey,
        pub maker_ata_b: Pubkey,
        pub taker_ata_a: Pubkey,
        pub taker_ata_b: Pubkey,
        pub escrow: Pubkey,
        pub vault: Pubkey,
    }

    static PROGRAM_ID: Pubkey = crate::ID;

    pub fn build_make_instruction(
        program: &mut LiteSVM,
        maker: &Keypair,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
        maker_ata_a: &Pubkey,
        escrow: &Pubkey,
        vault: &Pubkey,
    ) -> Instruction {
        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(program, &maker, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program: Pubkey = spl_associated_token_account::ID;
        let token_program: Pubkey = TOKEN_PROGRAM_ID;
        let system_program: Pubkey = SYSTEM_PROGRAM_ID;

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker.pubkey(),
                mint_a: *mint_a,
                mint_b: *mint_b,
                maker_ata_a: *maker_ata_a,
                escrow: *escrow,
                vault: *vault,
                associated_token_program: associated_token_program,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                deposit: 10,
                seed: 123u64,
                receive: 10,
                freeze_period: 5,
            }
            .data(),
        };

        make_ix
    }

    fn setup() -> EscrowTestEnvironment {
        // Initialize LiteSVM and maker
        let mut program = LiteSVM::new();
        let maker = Keypair::new();
        let taker = Keypair::new();

        // Airdrop some SOL to the maker keypair
        program
            .airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to maker");

        // Airdrop some SOL to the taker keypair
        program
            .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to taker");

        // Load program SO file
        let so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/anchor_escrow.so");
        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
        program.add_program(PROGRAM_ID, &program_data);

        // Example: Load an account from devnet
        // let rpc_client = RpcClient::new("https://api.devnet.solana.com");
        // let account_address =
        //     Address::from_str("DRYvf71cbF2s5wgaJQvAGkghMkRcp5arvsK2w97vXhi2").unwrap();
        // let fetched_account = rpc_client
        //     .get_account(&account_address)
        //     .expect("Failed to fetch account from devnet");

        // this was fucking up the sol amount in maker, if this is not there, we don't have to airdrop sol to the escrow pda at line 144
        // program
        //     .set_account(
        //         maker.pubkey(),
        //         Account {
        //             lamports: fetched_account.lamports,
        //             data: fetched_account.data,
        //             owner: Pubkey::from(fetched_account.owner.to_bytes()),
        //             executable: fetched_account.executable,
        //             rent_epoch: fetched_account.rent_epoch,
        //         },
        //     )
        //     .unwrap();

        // msg!("Lamports of fetched account: {}", fetched_account.lamports);

        // Create two mints (Mint A and Mint B) with 6 decimals and maker as authority
        let mint_a = CreateMint::new(&mut program, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        msg!("Mint A: {}\n", mint_a);

        let mint_b = CreateMint::new(&mut program, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        msg!("Mint B: {}\n", mint_b);

        // Create maker’s ATAs
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &maker, &mint_a)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        msg!("Maker ATA A: {}\n", maker_ata_a);

        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &maker, &mint_b)
            .owner(&maker.pubkey())
            .send()
            .unwrap();
        msg!("Maker ATA B: {}\n", maker_ata_b);

        // Create taker’s ATAs
        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_a)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        msg!("Taker ATA A: {}\n", taker_ata_a);

        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_b)
            .owner(&taker.pubkey())
            .send()
            .unwrap();
        msg!("Taker ATA B: {}\n", taker_ata_b);

        // Derive PDA for escrow account using maker pubkey and seed
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker.pubkey().as_ref(), &123u64.to_le_bytes()],
            &PROGRAM_ID,
        )
        .0;
        msg!("Escrow PDA: {}\n", escrow);

        // todo: weird  behavior, escrow doesn't have enough sol to let the vault create, why does this happen?
        // program
        //     .airdrop(&escrow, 2 * LAMPORTS_PER_SOL)
        //     .expect("Failed to airdrop 2sol on escrow");

        // Derive PDA for the vault ATA using escrow PDA and Mint A
        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
        msg!("Vault PDA: {}\n", vault);

        // Return the test environment
        EscrowTestEnvironment {
            program,
            maker,
            taker,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            vault,
        }
    }

    #[test]
    fn test_make() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let EscrowTestEnvironment {
            mut program,
            maker,
            taker: _,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b: _,
            taker_ata_a: _,
            taker_ata_b: _,
            escrow,
            vault,
        } = setup();

        let make_ix = build_make_instruction(
            &mut program,
            &maker,
            &mint_a,
            &mint_b,
            &maker_ata_a,
            &escrow,
            &vault,
        );

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&maker], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify the vault account and escrow account data after the "Make" instruction
        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10);
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, mint_a);

        let escrow_account = program.get_account(&escrow).unwrap();
        let escrow_data =
            crate::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
        assert_eq!(escrow_data.seed, 123u64);
        assert_eq!(escrow_data.maker, maker.pubkey());
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, 10);
    }

    #[test]
    fn test_refund() {
        let EscrowTestEnvironment {
            mut program,
            maker,
            taker: _,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b: _,
            taker_ata_a: _,
            taker_ata_b: _,
            escrow,
            vault,
        } = setup();

        let make_ix = build_make_instruction(
            &mut program,
            &maker,
            &mint_a,
            &mint_b,
            &maker_ata_a,
            &escrow,
            &vault,
        );

        // Create the "Refund" instruction to deposit tokens into the escrow
        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Refund {
                maker: maker.pubkey(),
                mint_a: mint_a,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                token_program: TOKEN_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Refund {}.data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let make_message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction1 = Transaction::new(&[&maker], make_message, recent_blockhash);

        // Send the transaction and capture the result
        let _tx1 = program.send_transaction(transaction1).unwrap();

        // Create and send the transaction containing the "Refund" instruction
        let refund_message = Message::new(&[refund_ix], Some(&maker.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&maker], refund_message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nRefund transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify the vault account and escrow account are no more
        let vault_account = program.get_account(&vault).unwrap();
        assert!(
            vault_account.data.is_empty(),
            "Vault account should be closed after refund"
        );

        let escrow_account = program.get_account(&escrow).unwrap();
        assert!(
            escrow_account.data.is_empty(),
            "Escrow PDA should be closed after refund"
        );
    }

    #[test]
    fn test_take() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let EscrowTestEnvironment {
            mut program,
            maker,
            taker,
            mint_a,
            mint_b,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            escrow,
            vault,
        } = setup();

        let associated_token_program: Pubkey = spl_associated_token_account::ID;
        let token_program: Pubkey = TOKEN_PROGRAM_ID;
        let system_program: Pubkey = SYSTEM_PROGRAM_ID;

        let make_ix = build_make_instruction(
            &mut program,
            &maker,
            &mint_a,
            &mint_b,
            &maker_ata_a,
            &escrow,
            &vault,
        );

        let make_message = Message::new(&[make_ix], Some(&maker.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction1 = Transaction::new(&[&maker], make_message, recent_blockhash);
        let _tx1 = program.send_transaction(transaction1).unwrap();

        MintTo::new(&mut program, &maker, &mint_b, &taker_ata_b, 1000000000)
            .send()
            .unwrap();

        let mut current_slot = program.get_sysvar::<Clock>();
        msg!("current slot: {}", current_slot.slot);
        // we set the freeze_period 5 above but time_travel 10 slots, just to be sure
        current_slot.slot += 10;
        program.set_sysvar::<Clock>(&current_slot);

        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Take {
                taker: taker.pubkey(),
                maker: maker.pubkey(),
                mint_a,
                mint_b,
                taker_ata_a,
                taker_ata_b,
                maker_ata_b,
                escrow,
                vault,
                associated_token_program,
                token_program,
                system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Take {}.data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let take_message = Message::new(&[take_ix], Some(&taker.pubkey()));

        let transaction = Transaction::new(&[&taker], take_message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        let new_slot = program.get_sysvar::<Clock>();
        msg!("new slot: {}", new_slot.slot);
        msg!("\n\nTake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify the taker-maker account and vault account data after the "Take" instruction
        let vault_account = program.get_account(&vault).unwrap();
        assert!(
            vault_account.data.is_empty(),
            "Vault account should be closed after take"
        );

        let taker_ata_a_account = program.get_account(&taker_ata_a).unwrap();
        let taker_ata_data = spl_token::state::Account::unpack(&taker_ata_a_account.data).unwrap();
        msg!("taker ata a amount: {}", taker_ata_data.amount);
        assert_eq!(taker_ata_data.amount, 10);
        assert_eq!(taker_ata_data.owner, taker.pubkey());
        assert_eq!(taker_ata_data.mint, mint_a);

        let maker_ata_b_account = program.get_account(&maker_ata_b).unwrap();
        let maker_ata_data = spl_token::state::Account::unpack(&maker_ata_b_account.data).unwrap();
        msg!("maker ata b amount: {}", maker_ata_data.amount);
        assert_eq!(maker_ata_data.amount, 10);
        assert_eq!(maker_ata_data.owner, maker.pubkey());
        assert_eq!(maker_ata_data.mint, mint_b);
    }
}
