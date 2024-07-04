use anchor_lang::prelude::*;

#[error_code]
pub enum LaunchpadErrorCode {
    #[msg("token amount zero")]
    TokenAmtErr,
    #[msg("game grid error")]
    GameGridErr,
    #[msg("pool create error")]
    PoolCreateErr,
    #[msg("pool owner error")]
    PoolOwnerErr,
    #[msg("game pre step error")]
    GamePreStepErr,
    #[msg("game user error")]
    GameUserErr,
    #[msg("game first error")]
    GameFirstStepErr,
    #[msg("game status error")]
    GameStatusErr,
    #[msg("game step pos error")]
    GameStepPosErr,
    #[msg("user cool down error")]
    UserCoolDownPosErr,
    #[msg("pool ended error")]
    PoolEndedErr,
    #[msg("vrf force error")]
    VrfForceErr,
    #[msg("vrf status error")]
    VrfStatusErr,
    #[msg("vrf result error")]
    VrfResultErr,
    #[msg("game id error")]
    GameIdErr,
    #[msg("param error")]
    ParamErr,
    #[msg("token exist")]
    TokenExistErr,
    #[msg("admin error")]
    AdminErr,
}
