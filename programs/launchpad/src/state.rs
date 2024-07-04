use crate::constants::*;
use crate::errors::LaunchpadErrorCode;
use anchor_lang::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[account]
#[derive(InitSpace)]
pub struct Launchpad {
    pub bump: u8,
    pub last_pool_id: u64,
    pub gogr_mint: Pubkey,
    pub gogr_fee: u64,
    pub gogr_receiver: Pubkey,
    pub manager: Pubkey,
    pub burn_rate: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub bump: u8,
    pub pool_id: u64,
    pub owner: Pubkey,
    // token info
    pub token_mint: Pubkey,
    pub token_amount: u64,
    pub available_token_amount: u64,
    #[max_len(100)]
    pub icon_url: String,
    #[max_len(100)]
    pub web_site: String,
    pub gogr_ext: u64,

    //game config
    pub gold_grid: u8,
    pub bomb_grid: u8,
    pub cooldown_duration: u16,
    pub allocation_value: u64,
    pub allocation_percentage: u64,
    pub vrf_count: u32,
    pub game_count: u64,
}

impl Pool {
    pub fn total_grid(&self) -> u8 {
        X_GRID * Y_GRID
    }
    pub fn get_reward_amt(&self) -> u64 {
        if self.allocation_percentage > 0 {
            let per = (self.available_token_amount as u128)
                .checked_mul(self.allocation_percentage as u128)
                .unwrap()
                .checked_div(ALLOCATION_PERCENTAGE as u128)
                .unwrap() as u64;
            if per > self.allocation_value {
                self.allocation_value
            } else {
                per
            }
        } else {
            self.allocation_value
        }
    }

    pub fn get_vrf(&mut self, cur: u64) -> u64 {
        self.vrf_count = (self.vrf_count + 1) % u32::MAX;
        let mut hasher = DefaultHasher::new();
        cur.hash(&mut hasher);
        self.vrf_count.hash(&mut hasher);
        self.available_token_amount.hash(&mut hasher);
        let hash_result = hasher.finish();
        hash_result as u64
    }
}

#[account]
#[derive(InitSpace)]
pub struct TokenExist {}

#[account]
#[derive(InitSpace)]
pub struct UserTokenInfo {
    pub pool_id: u64,
    pub pool: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct GameInfo {
    pub bump: u8,
    pub user: Pubkey,
    pub pool_id: u64,
    pub game_counter: u32,
    // last game info
    pub step: u8,
    pub token_amount: u64,
    pub game_status: GameStatus,
    pub mined_empty_grid: u8,
    pub mined_gold_grid: u8,
    pub grid_info: u128,
    pub grid_gold_info: u128,
    pub exploded_grid: u8,
    pub cooldown_timestamp: u32,
}

impl GameInfo {
    pub fn handle_vrf(&mut self, pool: &Pool, random: u64) -> Result<StepStatus> {
        let total_grid = pool.total_grid() as u16;
        let left_all_grid = total_grid - self.mined_empty_grid as u16 - self.mined_gold_grid as u16;
        let left_gold_grid = (pool.gold_grid - self.mined_gold_grid) as u16;
        let left_empty_grid = total_grid
            - pool.gold_grid as u16
            - pool.bomb_grid as u16
            - self.mined_empty_grid as u16;
        let remainder = (random % left_all_grid as u64) as u16;
        let step_statsu = if remainder <= left_empty_grid {
            self.game_status = GameStatus::Working;
            self.mined_empty_grid += 1;
            StepStatus::Empty
        } else if remainder <= left_empty_grid + left_gold_grid {
            let amt = pool.get_reward_amt();
            self.token_amount += amt;
            self.game_status = GameStatus::Working;
            self.mined_gold_grid += 1;
            StepStatus::Gold
        } else {
            self.game_status = GameStatus::Exploded;
            let clock = Clock::get()?;
            let cur = clock.unix_timestamp;
            self.cooldown_timestamp = cur as u32 + (pool.cooldown_duration as u32) * 60;
            StepStatus::Exploded
        };
        Ok(step_statsu)
    }

    pub fn clear_data(&mut self) {
        self.step = 0;
        self.token_amount = 0;
        self.game_status = GameStatus::None;
        self.mined_empty_grid = 0;
        self.mined_gold_grid = 0;
        self.grid_info = 0;
        self.cooldown_timestamp = 0;
        self.grid_gold_info = 0;
        self.exploded_grid = 0;
    }

    pub fn get_grid_mined(&self, x: u8, y: u8) -> bool {
        if x == 0 && y == 0 {
            return true;
        }
        let n = 10 * x + y;
        return (self.grid_info & (1 << n)) != 0;
    }

    pub fn set_grid_mined(&mut self, x: u8, y: u8, step_status: StepStatus) {
        let n = 10 * x + y;
        self.grid_info = self.grid_info | 1 << n;
        if step_status == StepStatus::Gold {
            self.grid_gold_info = self.grid_gold_info | 1 << n;
        } else if step_status == StepStatus::Exploded {
            self.exploded_grid = n
        }
    }

    pub fn check_grid(&self, x: u8, y: u8) -> Result<()> {
        require!(
            !self.get_grid_mined(x, y),
            LaunchpadErrorCode::GameStepPosErr
        );
        let left_mined = x > 0 && self.get_grid_mined(x - 1, y);
        let right_mined = x < X_GRID - 1 && self.get_grid_mined(x + 1, y);
        let up_mined = y > 0 && self.get_grid_mined(x, y - 1);
        let down_mined = y < Y_GRID - 1 && self.get_grid_mined(x, y + 1);

        require!(
            left_mined || right_mined || up_mined || down_mined,
            LaunchpadErrorCode::GameStepPosErr
        );
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Copy, Debug, InitSpace)]
pub enum GameStatus {
    None,
    Working,
    Collected,
    Exploded,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum StepStatus {
    Empty,
    Exploded,
    Gold,
}

#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Copy, Debug, InitSpace)]
pub enum AllocationType {
    Percentages,
    FixedAmount,
}

impl Default for AllocationType {
    fn default() -> Self {
        AllocationType::Percentages
    }
}
