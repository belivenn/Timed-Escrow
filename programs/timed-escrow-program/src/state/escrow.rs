use anchor_lang::prelude::*;

use crate::errors::EscrowError;
use crate::constants::*;

#[account]
pub struct Escrow {
    pub maker: Pubkey,
    pub maker_mint: Pubkey,
    pub taker_mint: Pubkey,
    pub taker: Pubkey,
    pub offer_amount: u8,
    pub seed: u64,
    pub expiry: u64,
    pub locking_period: u64,
    pub auth_bump: u8,
    pub vault_bump: u8,
    pub escrow_bump: u8,
    pub created_time: u64,
}

impl Escrow {
    pub const LEN: usize = 8 + 4 * PUBKEY_L + 4 * U64_L + 4 * U8_L;

    pub fn check_expiry(&self) -> Result<()> {
        require!(self.expiry > Clock::get()?.slot, EscrowError::Expired);
        Ok(())
    }

    pub fn check_locked(&self) -> Result<()> {
        require!(
            Clock::get()?.slot >= self.created_time + self.locking_period,
            EscrowError::InvalidRequiredTime
        );
        Ok(())
    }
}