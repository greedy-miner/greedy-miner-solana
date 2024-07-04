use crate::errors::LaunchpadErrorCode;
use crate::seeds::*;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MineArgs {
    pub pool_id: u64,
    pub pos_x: u8,
    pub pos_y: u8,
    pub step: u8,
}

#[derive(Accounts)]
#[instruction(args: MineArgs)]
pub struct Mine<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            POOL_INFO_SEED,
            args.pool_id.to_be_bytes().as_ref()
            ],
        bump=pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [
            GAME_INFO_SEED,
            args.pool_id.to_be_bytes().as_ref(),
            signer.key().as_ref()
            ],
        bump ,
        space = 8 + GameInfo::INIT_SPACE)]
    pub game_info: Box<Account<'info, GameInfo>>,

    pub system_program: Program<'info, System>,
}

pub fn mine_handler(ctx: Context<Mine>, args: MineArgs) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let game_info = &mut ctx.accounts.game_info;

    require!(
        pool.available_token_amount > 0,
        LaunchpadErrorCode::PoolEndedErr
    );

    let clock = Clock::get()?;
    let cur = clock.unix_timestamp;

    // check status if a new round
    if args.step == 1 {
        // check cooldown_timestamp
        require!(
            game_info.cooldown_timestamp == 0 || game_info.cooldown_timestamp <= cur as u32,
            LaunchpadErrorCode::UserCoolDownPosErr
        );

        if game_info.game_counter > 0 {
            require!(
                game_info.game_status == GameStatus::Collected
                    || game_info.game_status == GameStatus::Exploded,
                LaunchpadErrorCode::GameStatusErr
            );
            game_info.clear_data();
        } else {
            game_info.bump = ctx.bumps.game_info;
        }
    } else {
        require!(
            game_info.step + 1 == args.step,
            LaunchpadErrorCode::GameStepPosErr
        );
        require!(
            game_info.game_status == GameStatus::Working,
            LaunchpadErrorCode::GameStatusErr
        );
    }

    //check pos
    game_info.check_grid(args.pos_x, args.pos_y)?;

    // update game_info
    game_info.step = args.step;

    let pool = &mut ctx.accounts.pool;

    let random = pool.get_vrf(cur as u64);
    let step_statsu = game_info.handle_vrf(&ctx.accounts.pool, random)?;
    game_info.set_grid_mined(args.pos_x, args.pos_y, step_statsu);
    if step_statsu == StepStatus::Exploded {
        let pool = &mut ctx.accounts.pool;
        pool.game_count += 1;
        game_info.game_counter += 1;
    }

    Ok(())
}
