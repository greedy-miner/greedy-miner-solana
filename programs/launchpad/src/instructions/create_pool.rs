use crate::errors::LaunchpadErrorCode;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer},
};

use crate::constants::ALLOCATION_PERCENTAGE;
use crate::seeds::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreatePoolArgs {
    pub icon_url: String,
    pub web_site: String,
    pub gold_grid: u8,
    pub bomb_grid: u8,
    pub cooldown_duration: u16,
    pub allocation_value: u64,
    pub allocation_percentage: u64,
    pub gogr_ext: u64,
    pub token_amount: u64,
}

#[derive(Accounts)]
#[instruction(args: CreatePoolArgs)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut,seeds = [LAUNCHPAD_CONFIG_SEED],bump=launchpad.bump)]
    pub launchpad: Box<Account<'info, Launchpad>>,

    #[account(
        init,
        payer = signer,
        seeds = [
            POOL_INFO_SEED,
            launchpad.last_pool_id.to_be_bytes().as_ref()
            ],
        bump,
        space = 8 + Pool::INIT_SPACE)
    ]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut,address=launchpad.gogr_mint)]
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

    #[account(mut)]
    pub pool_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        seeds = [
            TOKEN_EXIST_SEED,
            pool_token_mint.key().as_ref()
        ],
        bump,
        space = 8 + TokenExist::INIT_SPACE)
    ]
    pub token_exist: Box<Account<'info, TokenExist>>,

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
impl<'info> CreatePool<'info> {
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

    fn burn_gogr(&self, amount: u64) -> Result<()> {
        burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.gogr_mint.to_account_info(),
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

pub fn create_pool_handler(ctx: Context<CreatePool>, args: CreatePoolArgs) -> Result<()> {
    let launchpad = &mut ctx.accounts.launchpad;
    let pool = &mut ctx.accounts.pool;

    pool.bump = ctx.bumps.pool;
    pool.pool_id = launchpad.last_pool_id;
    pool.token_mint = ctx.accounts.pool_token_mint.key();
    pool.owner = ctx.accounts.signer.key();
    pool.icon_url = args.icon_url;
    pool.web_site = args.web_site;
    pool.gold_grid = args.gold_grid;
    pool.bomb_grid = args.bomb_grid;
    pool.cooldown_duration = args.cooldown_duration;
    pool.allocation_value = args.allocation_value;
    require!(
        args.allocation_percentage < ALLOCATION_PERCENTAGE,
        LaunchpadErrorCode::ParamErr
    );
    pool.allocation_percentage = args.allocation_percentage;

    let total_grid = args.gold_grid.checked_add(args.bomb_grid).unwrap();
    require!(
        total_grid < pool.total_grid() && total_grid > 0,
        LaunchpadErrorCode::GameGridErr
    );

    launchpad.last_pool_id = launchpad.last_pool_id.checked_add(1).unwrap();

    require!(args.token_amount > 0, LaunchpadErrorCode::TokenAmtErr);

    if args.token_amount > 0 {
        pool.token_amount += args.token_amount;
        pool.available_token_amount += args.token_amount;
    }
    // transfer gogr
    if launchpad.gogr_mint != pool.token_mint {
        let gogr_amt = args.gogr_ext.checked_add(launchpad.gogr_fee).unwrap();
        if gogr_amt > 0 {
            pool.gogr_ext += gogr_amt;
            let burn_amt = gogr_amt * launchpad.burn_rate as u64 / 100 as u64;
            let left_amt = gogr_amt - burn_amt;
            ctx.accounts.transfer_gogr(left_amt)?;
            ctx.accounts.burn_gogr(burn_amt)?;
        }
    }

    // transfer pool token
    if args.token_amount > 0 {
        ctx.accounts.transfer_pool_token(args.token_amount)?;
    }
    Ok(())
}
