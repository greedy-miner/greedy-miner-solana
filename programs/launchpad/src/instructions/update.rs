use anchor_lang::prelude::*;
use crate::errors::LaunchpadErrorCode;


use crate::seeds::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct UpdateArgs {
    pub gogr_fee: Option<u64>,
    pub gogr_receiver: Option<Pubkey>,
    pub manager: Option<Pubkey>,
    pub burn_rate: Option<u8>,

}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub manager: Signer<'info>,
    #[account(
        mut,
        seeds = [LAUNCHPAD_CONFIG_SEED],
        bump = launchpad.bump, 
        has_one = manager @ LaunchpadErrorCode::AdminErr
    )]
    pub launchpad: Box<Account<'info, Launchpad>>,
    pub system_program: Program<'info, System>,
}

pub fn update_handler(ctx: Context<Update>, args: UpdateArgs) -> Result<()> {
    let launchpad = &mut ctx.accounts.launchpad;
    
    if let Some(gogr_fee) = args.gogr_fee {
        launchpad.gogr_fee = gogr_fee;
    }
    if let Some(gogr_receiver) = args.gogr_receiver {
        launchpad.gogr_receiver = gogr_receiver;
    }
    if let Some(manager) = args.manager {
        launchpad.manager = manager;
    }
    if let Some(burn_rate) = args.burn_rate {
        launchpad.burn_rate = burn_rate;
    }
    Ok(())
}
