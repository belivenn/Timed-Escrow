use anchor_lang::prelude::*;

declare_id!("4PK12mvcUeybBJ1rBYocsKRALj8KdCoGH5Kn2tCzn1Mw");

mod errors;
mod state;
mod contexts;
mod constants;
use contexts::*;


#[program]
pub mod timed_escrow_program {
    use super::*;

    pub fn make(
        ctx: Context<Make>,
        seed: u64,
        deposit_amount: u64,
        expiry: u64,
        locking_period: u64
    ) -> Result<()> {
        ctx.accounts.init_escrow(seed, expiry, locking_period, &ctx.bumps)?;
        ctx.accounts.issue_tokens()?;
        ctx.accounts.transfer_to_vault(deposit_amount)
    }

    // Cancel and refund escrow to the maker
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.empty_vault()?;
        ctx.accounts.close_vault()
    }

    // Allow taker to accept the escrow
    pub fn claim(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.check_expiry()?;
        ctx.accounts.deposit_to_vault()
    }

    // Allow taker to receive the Vault Funds
    pub fn take(ctx: Context<Take>) -> Result<()> {        
        ctx.accounts.check_locked()?;
        ctx.accounts.empty_vault_to_taker()?;
        ctx.accounts.close_vault()
    }
}


