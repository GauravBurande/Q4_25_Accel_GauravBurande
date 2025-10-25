use anchor_lang::prelude::*;

use solana_gpt_oracle::cpi::{
    accounts::{CreateLlmContext, InteractWithLlm},
    create_llm_context, interact_with_llm,
};
use solana_gpt_oracle::{ContextAccount, Counter, Identity};

declare_id!("FMCBSyWV6apm9XjL5oc3rSnyQLtgb38TCVUuxpHdaxvQ");

#[program]
pub mod magicblock_solana_ai_oracle {

    use super::*;

    const AGENT_DESC: &str = "You are a DeFi Credit Agent. Analyze a user's Twitter profile and activity to infer their on-chain reputation, trustworthiness, and DeFi literacy. Output a single DeFi Credit Score (0–100) as an integer based on these signals. Only return the number — do not include explanations, text, or any extra information.";
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.agent.context = ctx.accounts.llm_context.key();
        ctx.accounts.agent.bump = ctx.bumps.agent;

        // Create the context for the AI agent
        let cpi_program = ctx.accounts.oracle_program.to_account_info();
        let cpi_accounts = CreateLlmContext {
            payer: ctx.accounts.signer.to_account_info(),
            counter: ctx.accounts.counter.to_account_info(),
            context_account: ctx.accounts.llm_context.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        create_llm_context(cpi_ctx, AGENT_DESC.to_string())?;
        Ok(())
    }

    pub fn interact_agent(ctx: Context<InteractAgent>, text: String) -> Result<()> {
        let cpi_program = ctx.accounts.oracle_program.to_account_info();

        let cpi_accounts = InteractWithLlm {
            payer: ctx.accounts.user.to_account_info(),
            interaction: ctx.accounts.interaction.to_account_info(),
            context_account: ctx.accounts.context_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        let disc: [u8; 8] = instruction::CallbackFromAgent::DISCRIMINATOR
            .try_into()
            .expect("Discriminator must be 8 bytes");

        interact_with_llm(
            cpi_ctx,
            text,
            ID,
            disc,
            Some(vec![
                solana_gpt_oracle::AccountMeta {
                    pubkey: ctx.accounts.user.to_account_info().key(),
                    is_signer: true,
                    is_writable: false,
                },
                solana_gpt_oracle::AccountMeta {
                    pubkey: ctx.accounts.cred_score.to_account_info().key(),
                    is_signer: false,
                    is_writable: true,
                },
            ]),
        )?;
        Ok(())
    }
    pub fn callback_from_agent(ctx: Context<CallbackFromAgent>, response: String) -> Result<()> {
        // Ensure the identity is a signer
        if !ctx.accounts.identity.to_account_info().is_signer {
            return Err(ProgramError::InvalidAccountData.into());
        }

        msg!("AI response received: {}", response);

        // Parse the response as u8
        let parsed_score: u8 = response.trim().parse().map_err(|_| {
            msg!("Failed to parse AI response as a number");
            ProgramError::InvalidInstructionData
        })?;

        if parsed_score > 100 {
            msg!("Score exceeds 100, clamping to 100");
        }

        // Update the cred_score account
        let cred_score_account = &mut ctx.accounts.cred_score;
        cred_score_account.score = parsed_score.min(100);

        msg!("Stored score: {}", cred_score_account.score);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 1,
        seeds = [b"agent"],
        bump
    )]
    pub agent: Account<'info, Agent>,

    /// CHECK: checked in the oracle program
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,
    #[account(mut)]
    pub counter: Account<'info, Counter>,

    /// CHECK: the oracle program id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
// #[instruction(text: String)] don't know why do we need this
pub struct InteractAgent<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + CredScore::INIT_SPACE,
        seeds=[b"cred", user.key().as_ref()],
        bump
    )]
    pub cred_score: Account<'info, CredScore>,

    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub interaction: AccountInfo<'info>,

    #[account(seeds = [b"agent"], bump = agent.bump)]
    pub agent: Account<'info, Agent>,

    #[account(address = agent.context)]
    pub context_account: Account<'info, ContextAccount>,

    /// CHECK: the oracle program id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CallbackFromAgent<'info> {
    /// CHECK: this is checked by oracle program
    pub identity: Account<'info, Identity>,

    /// CHECK: the user who's checking their cred score
    pub user: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds=[b"cred", user.key().as_ref()],
        bump = cred_score.bump
    )]
    pub cred_score: Account<'info, CredScore>,
}

#[account]
#[derive(InitSpace)]
pub struct CredScore {
    pub score: u8,
    pub bump: u8,
}

#[account]
pub struct Agent {
    pub context: Pubkey,
    pub bump: u8,
}
