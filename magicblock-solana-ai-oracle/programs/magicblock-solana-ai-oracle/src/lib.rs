use anchor_lang::prelude::*;
use solana_gpt_oracle::{ContextAccount, Counter, Identity};

declare_id!("2cUo1HiLi1Hg4wj8Q12QjsTqYKeJDKaFR2XHFFHu24Pp");

#[program]
pub mod magicblock_solana_ai_oracle {

    use super::*;

    const AGENT_DESC: &str = "You are a GM BOT!";
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.agent.context = ctx.accounts.llm_context.key();

        // Create the context for the AI agent
        let cpi_program = ctx.accounts.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::CreateLlmContext {
            payer: ctx.accounts.signer.to_account_info(),
            counter: ctx.accounts.counter.to_account_info(),
            context_account: ctx.accounts.llm_context.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        solana_gpt_oracle::solana_gpt_oracle::create_llm_context(cpi_ctx, AGENT_DESC.to_string())?;
        Ok(())
    }

    pub fn interact_agent(ctx: Context<InteractAgent>, text: String) -> Result<()> {
        let cpi_program = ctx.accounts.oracle_program.to_account_info();

        let cpi_accounts = solana_gpt_oracle::InteractWithLlm {
            payer: ctx.accounts.user.to_account_info(),
            interaction: ctx.accounts.interaction.to_account_info(),
            context_account: ctx.accounts.context_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        let DISCRIMINA: [u8; 8] = instruction::CallbackFromAgent::DISCRIMINATOR
            .try_into()
            .expect("Discriminator must be 8 bytes");

        solana_gpt_oracle::solana_gpt_oracle::interact_with_llm(
            cpi_ctx, text, ID, DISCRIMINA, None, // add the metas for user and the score_pda
        )?;
        Ok(())
    }
    pub fn callback_from_agent(ctx: Context<CallbackFromAgent>, response: string) -> Result<()> {
        msg!("The ai response: {}", response);
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
        space = 8 + 32,
        seeds = [b"agent"],
        bump
    )]
    pub agent: Account<'info, Agent>,

    /// CHECK: checked in the oracle program
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,
    #[account(mut)]
    pub counter: Account<'info, Counter>,

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
