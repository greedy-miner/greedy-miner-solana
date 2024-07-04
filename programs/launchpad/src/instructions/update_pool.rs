use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::seeds::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdatePoolArgs {
    pub gogr_ext: u64,
    pub token_amount: u64,
}

#[derive(Accounts)]
#[instruction(args: UpdatePoolArgs)]
pub struct UpdatePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut,seeds = [LAUNCHPAD_CONFIG_SEED],bump=launchpad.bump)]
    pub launchpad: Box<Account<'info, Launchpad>>,

    #[account(
        mut,
        seeds = [
            POOL_INFO_SEED,
            launchpad.last_pool_id.to_be_bytes().as_ref()
            ],
        bump)
    ]
    pub pool: Box<Account<'info, Pool>>,

    #[account(address=launchpad.gogr_mint)]
    pub gogr_mint: Box<Account<'info, Mint>>,

    /// CHECK:
    #[account(address=launchpad.gogr_receiver)]
    pub gogr_receiver: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = gogr_mint,
        associated_token::authority = signer
    )]
    pub gogr_from_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = gogr_mint,
        associated_token::authority = gogr_receiver
    )]
    pub gogr_receiver_account: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub pool_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = pool_token_mint,
        associated_token::authority = signer
    )]
    pub token_owner_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = pool_token_mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
impl<'info> UpdatePool<'info> {
    fn transfer_gogr(&self, amount: u64) -> Result<()> {
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    to: self.gogr_receiver_account.to_account_info(),
                    from: self.gogr_from_account.to_account_info(),
                    authority: self.signer.to_account_info(),
                },
            ),
            amount,
        )
    }

    fn transfer_pool_token(&self, amount: u64) -> Result<()> {
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    to: self.pool_token_account.to_account_info(),
                    from: self.token_owner_account.to_account_info(),
                    authority: self.signer.to_account_info(),
                },
            ),
            amount,
        )
    }
}

pub fn update_pool_handler(ctx: Context<UpdatePool>, args: UpdatePoolArgs) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    if args.gogr_ext > 0 {
        pool.gogr_ext += args.gogr_ext;
        ctx.accounts.transfer_gogr(args.gogr_ext)?;
    }
    let pool = &mut ctx.accounts.pool;

    // transfer gogr
    if args.token_amount > 0 {
        pool.token_amount += args.token_amount;
        pool.available_token_amount += args.token_amount;
        ctx.accounts.transfer_pool_token(args.token_amount)?;
    }
    Ok(())
}
