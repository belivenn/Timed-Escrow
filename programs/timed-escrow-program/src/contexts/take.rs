use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::state::Escrow;

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    pub maker_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = taker_mint,
        associated_token::authority = maker
    )]
    pub maker_receive_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = taker_mint,
        associated_token::authority = taker
    )]
    pub taker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = maker_mint,
        associated_token::authority = taker
    )]
    pub taker_receive_ata: InterfaceAccount<'info, TokenAccount>,
    pub taker_mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"auth", escrow.key().as_ref()],
        bump = escrow.auth_bump
    )]
    ///CHECK: This is safe. It's just used to sign things
    pub auth: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"vault", escrow.key().as_ref()],
        bump = escrow.vault_bump,
        token::mint = maker_mint,
        token::authority = auth
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        has_one = maker,
        has_one = taker_mint,
        has_one = maker_mint,
        seeds = [b"escrow", maker.key.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.escrow_bump,
        close = taker
    )]
    pub escrow: Account<'info, Escrow>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    pub fn check_expiry(&self) -> Result<()> {
        self.escrow.check_expiry()
    }
    pub fn deposit_to_vault(&self) -> Result<()> {
        let accounts = TransferChecked {
            from: self.taker_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.taker.to_account_info(),
            mint: self.taker_mint.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer_checked(cpi_ctx, 1, self.taker_mint.decimals)
    }


    pub fn check_locked(&self) -> Result<()> {
        self.escrow.check_locked()
    }

    pub fn empty_vault_to_taker(&self) -> Result<()> {
        
        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.taker_receive_ata.to_account_info(),
            authority: self.auth.to_account_info(),
            mint: self.maker_mint.to_account_info(),
        };
        let seeds = &[
            &b"auth"[..],
            &self.escrow.key().to_bytes()[..],
            &[self.escrow.auth_bump]];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        transfer_checked(cpi_ctx, self.vault.amount, self.maker_mint.decimals)
    }

        pub fn empty_vault_to_maker(&self) -> Result<()> {
       
        let accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.maker_receive_ata.to_account_info(),
            authority: self.auth.to_account_info(),
            mint: self.taker_mint.to_account_info(),
        };
        let seeds = &[
            &b"auth"[..],
            &self.escrow.key().to_bytes()[..],
            &[self.escrow.auth_bump]];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        transfer_checked(cpi_ctx, self.vault.amount, self.maker_mint.decimals)
    }
    pub fn close_vault(&self) -> Result<()> {
        let accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
            authority: self.auth.to_account_info(),
        };

        let seeds = &[
            &b"auth"[..],
            &self.escrow.key().to_bytes()[..],
            &[self.escrow.auth_bump]];

        let signer_seeds = &[&seeds[..]];
        
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds
        );
        close_account(cpi_ctx)
    }
}
