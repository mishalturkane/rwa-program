use anchor_lang::prelude::*;

declare_id!("RWAaPWC9witz2n2vSA24KetECWPyCkBhSaBmr8MpXu9");

#[program]
pub mod rwa_program {
    use super::*;

    pub fn create_mint(ctx: Context<CreateMint>, params: CreateMintParams) -> Result<()> {
        let mint_key  = ctx.accounts.mint.key();
        let payer_key = ctx.accounts.payer.key();

        let mint  = &mut ctx.accounts.mint;
        let clock = Clock::get()?;

        mint.authority      = payer_key;
        mint.decimals       = params.decimals;
        mint.supply         = 0;
        mint.is_initialized = true;

        mint.mint_close_authority = Some(payer_key);
        mint.permanent_delegate   = Some(payer_key);
        mint.non_transferable     = params.non_transferable;

        let fee = TransferFee {
            epoch                    : clock.epoch,
            maximum_fee              : params.max_fee,
            transfer_fee_basis_points: params.fee_basis_points,
        };
        mint.transfer_fee = TransferFeeConfig {
            transfer_fee_config_authority: Some(payer_key),
            withdraw_withheld_authority  : Some(payer_key),
            withheld_amount              : 0,
            older_transfer_fee           : fee.clone(),
            newer_transfer_fee           : fee,
        };

        mint.interest_bearing = InterestBearingConfig {
            rate_authority           : Some(payer_key),
            initialization_timestamp : clock.unix_timestamp,
            pre_update_average_rate  : params.interest_rate,
            last_update_timestamp    : clock.unix_timestamp,
            current_rate             : params.interest_rate,
        };

        mint.default_account_state = DefaultAccountState {
            state: AccountState::Frozen,
        };

        mint.transfer_hook = TransferHookConfig {
            authority : Some(payer_key),
            program_id: Some(params.transfer_hook_program_id),
        };

        mint.confidential_transfer = ConfidentialTransferConfig {
            authority                : Some(payer_key),
            auto_approve_new_accounts: false,
            auditor_elgamal_pubkey   : None,
        };

        mint.metadata_pointer = MetadataPointerConfig {
            authority       : Some(payer_key),
            metadata_address: Some(mint_key),
        };

        mint.metadata = TokenMetadata {
            update_authority    : Some(payer_key),
            mint                : mint_key,
            name                : params.name,
            symbol              : params.symbol,
            uri                 : params.uri,
            additional_metadata : vec![],
        };

        Ok(())
    }

    pub fn update_transfer_fee(
        ctx                 : Context<UpdateAuthority>,
        new_fee_basis_points: u16,
        new_max_fee         : u64,
    ) -> Result<()> {
        let mint = &mut ctx.accounts.mint;

        require!(
            mint.transfer_fee
                .transfer_fee_config_authority
                .map(|k| k == ctx.accounts.authority.key())
                .unwrap_or(false),
            RwaError::Unauthorized
        );

        let clock = Clock::get()?;

        mint.transfer_fee.older_transfer_fee = mint.transfer_fee.newer_transfer_fee.clone();
        mint.transfer_fee.newer_transfer_fee = TransferFee {
            epoch                    : clock.epoch + 1,
            maximum_fee              : new_max_fee,
            transfer_fee_basis_points: new_fee_basis_points,
        };

        Ok(())
    }

    pub fn update_interest_rate(
        ctx     : Context<UpdateAuthority>,
        new_rate: i16,
    ) -> Result<()> {
        let mint = &mut ctx.accounts.mint;

        require!(
            mint.interest_bearing
                .rate_authority
                .map(|k| k == ctx.accounts.authority.key())
                .unwrap_or(false),
            RwaError::Unauthorized
        );

        let clock = Clock::get()?;
        mint.interest_bearing.pre_update_average_rate = mint.interest_bearing.current_rate;
        mint.interest_bearing.last_update_timestamp   = clock.unix_timestamp;
        mint.interest_bearing.current_rate            = new_rate;

        Ok(())
    }

    pub fn close_mint(ctx: Context<CloseMint>) -> Result<()> {
        let mint = &ctx.accounts.mint;

        require!(mint.supply == 0, RwaError::SupplyNotZero);

        require!(
            mint.mint_close_authority
                .map(|k| k == ctx.accounts.authority.key())
                .unwrap_or(false),
            RwaError::Unauthorized
        );

        Ok(())
    }

    pub fn update_metadata(
        ctx       : Context<UpdateAuthority>,
        new_name  : Option<String>,
        new_symbol: Option<String>,
        new_uri   : Option<String>,
    ) -> Result<()> {
        let mint = &mut ctx.accounts.mint;

        require!(
            mint.metadata
                .update_authority
                .map(|k| k == ctx.accounts.authority.key())
                .unwrap_or(false),
            RwaError::Unauthorized
        );

        if let Some(n) = new_name   { mint.metadata.name   = n; }
        if let Some(s) = new_symbol { mint.metadata.symbol = s; }
        if let Some(u) = new_uri    { mint.metadata.uri    = u; }

        Ok(())
    }

    pub fn add_metadata_field(
        ctx  : Context<UpdateAuthority>,
        key  : String,
        value: String,
    ) -> Result<()> {
        let mint = &mut ctx.accounts.mint;

        require!(
            mint.metadata
                .update_authority
                .map(|k| k == ctx.accounts.authority.key())
                .unwrap_or(false),
            RwaError::Unauthorized
        );

        if let Some(field) = mint.metadata.additional_metadata
            .iter_mut()
            .find(|f| f.key == key)
        {
            field.value = value;
        } else {
            mint.metadata.additional_metadata.push(MetadataField { key, value });
        }

        Ok(())
    }

    pub fn set_default_account_state(
        ctx      : Context<UpdateAuthority>,
        new_state: AccountState,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.mint.authority,
            RwaError::Unauthorized
        );
        ctx.accounts.mint.default_account_state.state = new_state;
        Ok(())
    }

    pub fn update_transfer_hook(
        ctx           : Context<UpdateAuthority>,
        new_program_id: Option<Pubkey>,
    ) -> Result<()> {
        let mint = &mut ctx.accounts.mint;

        require!(
            mint.transfer_hook
                .authority
                .map(|k| k == ctx.accounts.authority.key())
                .unwrap_or(false),
            RwaError::Unauthorized
        );

        mint.transfer_hook.program_id = new_program_id;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  PARAMS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateMintParams {
    pub decimals                : u8,
    pub fee_basis_points        : u16,
    pub max_fee                 : u64,
    pub interest_rate           : i16,
    pub non_transferable        : bool,
    pub transfer_hook_program_id: Pubkey,
    pub name                    : String,
    pub symbol                  : String,
    pub uri                     : String,
}

// ═══════════════════════════════════════════════════════════════════════════
//  ACCOUNT CONTEXTS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + RwaMint::SIZE
    )]
    pub mint          : Account<'info, RwaMint>,
    #[account(mut)]
    pub payer         : Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    #[account(mut)]
    pub mint     : Account<'info, RwaMint>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseMint<'info> {
    #[account(
        mut,
        close = authority
    )]
    pub mint     : Account<'info, RwaMint>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  MAIN STATE
// ═══════════════════════════════════════════════════════════════════════════

#[account]
pub struct RwaMint {
    pub authority      : Pubkey,
    pub decimals       : u8,
    pub supply         : u64,
    pub is_initialized : bool,

    pub mint_close_authority : Option<Pubkey>,
    pub permanent_delegate   : Option<Pubkey>,
    pub non_transferable     : bool,
    pub transfer_fee         : TransferFeeConfig,
    pub interest_bearing     : InterestBearingConfig,
    pub default_account_state: DefaultAccountState,
    pub transfer_hook        : TransferHookConfig,
    pub confidential_transfer: ConfidentialTransferConfig,
    pub metadata_pointer     : MetadataPointerConfig,
    pub metadata             : TokenMetadata,
}

impl RwaMint {
    pub const SIZE: usize =
        32 + 1 + 8 + 1
        + (1 + 32)
        + (1 + 32)
        + 1
        + TransferFeeConfig::SIZE
        + InterestBearingConfig::SIZE
        + DefaultAccountState::SIZE
        + TransferHookConfig::SIZE
        + ConfidentialTransferConfig::SIZE
        + MetadataPointerConfig::SIZE
        + TokenMetadata::SIZE;
}

// ═══════════════════════════════════════════════════════════════════════════
//  EXTENSION STRUCTS
// ═══════════════════════════════════════════════════════════════════════════

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TransferFee {
    pub epoch                    : u64,
    pub maximum_fee              : u64,
    pub transfer_fee_basis_points: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TransferFeeConfig {
    pub transfer_fee_config_authority: Option<Pubkey>,
    pub withdraw_withheld_authority  : Option<Pubkey>,
    pub withheld_amount              : u64,
    pub older_transfer_fee           : TransferFee,
    pub newer_transfer_fee           : TransferFee,
}
impl TransferFeeConfig {
    pub const SIZE: usize = 33 + 33 + 8 + 18 + 18; // 110
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct InterestBearingConfig {
    pub rate_authority           : Option<Pubkey>,
    pub initialization_timestamp : i64,
    pub pre_update_average_rate  : i16,
    pub last_update_timestamp    : i64,
    pub current_rate             : i16,
}
impl InterestBearingConfig {
    pub const SIZE: usize = 33 + 8 + 2 + 8 + 2; // 53
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq)]
pub enum AccountState {
    #[default]
    Initialized,
    Frozen,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct DefaultAccountState {
    pub state: AccountState,
}
impl DefaultAccountState {
    pub const SIZE: usize = 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TransferHookConfig {
    pub authority : Option<Pubkey>,
    pub program_id: Option<Pubkey>,
}
impl TransferHookConfig {
    pub const SIZE: usize = 33 + 33; // 66
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ConfidentialTransferConfig {
    pub authority                : Option<Pubkey>,
    pub auto_approve_new_accounts: bool,
    pub auditor_elgamal_pubkey   : Option<[u8; 32]>,
}
impl ConfidentialTransferConfig {
    pub const SIZE: usize = 33 + 1 + 33; // 67
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MetadataPointerConfig {
    pub authority       : Option<Pubkey>,
    pub metadata_address: Option<Pubkey>,
}
impl MetadataPointerConfig {
    pub const SIZE: usize = 33 + 33; // 66
}

// ── KEY FIX: named struct instead of tuple ───────────────────────────────────
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MetadataField {
    pub key  : String,   // 4 + up to 32 bytes
    pub value: String,   // 4 + up to 64 bytes
}
impl MetadataField {
    pub const SIZE: usize = (4 + 32) + (4 + 64); // 104
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct TokenMetadata {
    pub update_authority    : Option<Pubkey>,       // 33
    pub mint                : Pubkey,               // 32
    pub name                : String,               // 4+32  = 36
    pub symbol              : String,               // 4+10  = 14
    pub uri                 : String,               // 4+200 = 204
    pub additional_metadata : Vec<MetadataField>,   // 4 (empty vec prefix)
}
impl TokenMetadata {
    pub const SIZE: usize = 33 + 32 + 36 + 14 + 204 + 4; // 323
}

// ═══════════════════════════════════════════════════════════════════════════
//  ERRORS
// ═══════════════════════════════════════════════════════════════════════════

#[error_code]
pub enum RwaError {
    #[msg("Unauthorized: signer is not the correct authority")]
    Unauthorized,
    #[msg("No authority configured for this extension")]
    NoAuthority,
    #[msg("Token supply must be zero before closing the mint")]
    SupplyNotZero,
}