use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("The freeze period is still not over, take it sometime again!")]
    FreezePeriodNotOver,
}
