use crate::errors::LaunchpadErrorCode;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::seeds::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CollectArgs {
    pub pool_id: u64,
}

#[derive(Accounts)]
#[instruction(args: CollectArgs)]
pub struct Collect<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut,seeds = [POOL_INFO_SEED,args.pool_id.to_be_bytes().as_ref()],bump=pool.bump)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        seeds = [
            GAME_INFO_SEED,
            pool.pool_id.to_be_bytes().as_ref(),
            signer.key().as_ref()
        ],
        bump=game_info.bump)]
    pub game_info: Box<Account<'info, GameInfo>>,

    #[account(address=pool.token_mint)]
    pub pool_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        associated_token::mint = pool_token_mint,
        associated_token::authority = signer,
        payer=signer
    )]
    pub token_owner_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = pool_token_mint,
        associated_token::authority = pool
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
impl<'info> Collect<'info> {
    fn transfer_pool_token(&self, amount: u64) -> Result<()> {
        let pool_id_bytes = self.pool.pool_id.to_be_bytes();
        let signer_seeds = [POOL_INFO_SEED, pool_id_bytes.as_ref(), &[self.pool.bump]];
        transfer(
            CpiContext::new(
                self.token_program.to_account_info(),
                Transfer {
                    to: self.token_owner_account.to_account_info(),
                    from: self.pool_token_account.to_account_info(),
                    authority: self.pool.to_account_info(),
                },
            )
            .with_signer(&[&signer_seeds]),
            amount,
        )
    }
}

pub fn collect_handler(ctx: Context<Collect>, _args: CollectArgs) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.game_count += 1;

    let game_info = &mut ctx.accounts.game_info;
    require!(
        game_info.game_status == GameStatus::Working,
        LaunchpadErrorCode::GameStatusErr
    );

    let token_amount = game_info.token_amount;

    game_info.game_status = GameStatus::Collected;
    require!(token_amount > 0, LaunchpadErrorCode::TokenAmtErr);
    game_info.game_counter += 1;

    // check cooldown
    let clock = Clock::get()?;
    let cur = clock.unix_timestamp;
    game_info.cooldown_timestamp = cur as u32 + (pool.cooldown_duration as u32) * 60;

    let mut amt = token_amount;
    if amt > pool.available_token_amount {
        amt = pool.available_token_amount
    }

    if amt > 0 {
        // transfer pool token
        pool.available_token_amount -= amt;
        ctx.accounts.transfer_pool_token(amt)?;
    }

    Ok(())
}
