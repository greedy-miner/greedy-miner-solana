use anchor_lang::prelude::*;

declare_id!("9L83XgZMdX1Sj52WPkLE5HoHd3Ygp8xRUTqWS2Vx9cF6");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod seeds;
pub mod state;
use instructions::*;

#[program]
pub mod launchpad {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitLaunchpadArgs) -> Result<()> {
        instructions::initialize_handler(ctx, args)
    }

    pub fn create_pool(ctx: Context<CreatePool>, args: CreatePoolArgs) -> Result<()> {
        instructions::create_pool_handler(ctx, args)
    }

    pub fn update_pool(ctx: Context<UpdatePool>, args: UpdatePoolArgs) -> Result<()> {
        instructions::update_pool_handler(ctx, args)
    }

    pub fn mine(ctx: Context<Mine>, args: MineArgs) -> Result<()> {
        instructions::mine_handler(ctx, args)
    }

    pub fn collect(ctx: Context<Collect>, args: CollectArgs) -> Result<()> {
        instructions::collect_handler(ctx, args)
    }

    pub fn update(ctx: Context<Update>, args: UpdateArgs) -> Result<()> {
        instructions::update_handler(ctx, args)
    }
}
