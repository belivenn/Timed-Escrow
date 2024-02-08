use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, MintTo, mint_to, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub maker_mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"mint", escrow.key().as_ref()],
        bump
    )]
    pub taker_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = maker,
        associated_token::mint = taker_mint,
        associated_token::authority = taker
    )]
    taker_ata: InterfaceAccount<'info, TokenAccount>,    
    /// CHECK
    taker: AccountInfo<'info>,
    #[account(
        mut,
        associated_token::mint = maker_mint,
        associated_token::authority = maker
    )]
    pub maker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"auth", escrow.key().as_ref()],
        bump
    )]
    ///CHECK: This is safe. It's just used to sign things
    pub auth: UncheckedAccount<'info>,
    #[account(
        init,
        payer = maker,
        seeds = [b"vault", escrow.key().as_ref()],
        bump,
        token::mint = maker_mint,
        token::authority = auth
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        seeds = [b"escrow", maker.key.as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = Escrow::LEN
    )]
    pub escrow: Account<'info, Escrow>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    pub fn init_escrow(
        &mut self,
        seed: u64,
        expiry: u64,
        locking_period: u64,
        bumps: &MakeBumps,
    ) -> Result<()> {
        self.escrow.set_inner(Escrow {
            maker: self.maker.key(),
            maker_mint: self.maker_mint.key(),
            taker_mint: self.taker_mint.key(),
            taker: self.taker.key(),
            offer_amount: 1,
            seed,
            expiry,
            locking_period,
            auth_bump: bumps.auth,
            vault_bump: bumps.vault,
            escrow_bump: bumps.escrow,
            created_time: Clock::get()?.slot
        });
        Ok(())
    }
    
    pub fn transfer_to_vault(&self, amount: u64) -> Result<()> {
        let accounts = TransferChecked  {
            from: self.maker_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info(),
            mint: self.maker_mint.to_account_info(),

        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer_checked(cpi_ctx, amount, self.maker_mint.decimals)
    }

    pub fn issue_tokens(
        &self
    ) -> Result<()> {
        let accounts = MintTo {
            mint: self.taker_mint.to_account_info(),
            to: self.taker_ata.to_account_info(),
            authority: self.auth.to_account_info()
        };

        let seeds = &[
            &b"auth"[..],
            &self.escrow.key().to_bytes()[..],
            &[self.escrow.auth_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds
        );

        mint_to(ctx, 1)
    }

}
