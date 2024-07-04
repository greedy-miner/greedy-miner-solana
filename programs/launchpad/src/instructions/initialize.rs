use anchor_lang::prelude::*;

use crate::seeds::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct InitLaunchpadArgs {
    pub gogr_mint: Pubkey,
    pub gogr_fee: u64,
    pub gogr_receiver: Pubkey,
    pub manager: Pubkey,
    pub burn_rate: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init, payer = signer, seeds = [LAUNCHPAD_CONFIG_SEED],bump, space = 8 + Launchpad::INIT_SPACE)]
    pub launchpad: Box<Account<'info, Launchpad>>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_handler(ctx: Context<Initialize>, args: InitLaunchpadArgs) -> Result<()> {
    let launchpad = &mut ctx.accounts.launchpad;
    launchpad.bump = ctx.bumps.launchpad;
    launchpad.gogr_mint = args.gogr_mint;
    launchpad.gogr_fee = args.gogr_fee;
    launchpad.gogr_receiver = args.gogr_receiver;
    launchpad.manager = args.manager;
    launchpad.burn_rate = args.burn_rate;
    Ok(())
}
