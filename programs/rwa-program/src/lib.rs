use anchor_lang::prelude::*;

declare_id!("RWAaPWC9witz2n2vSA24KetECWPyCkBhSaBmr8MpXu9");

#[program]
pub mod rwa_program {
    use super::*;

    pub fn create_mint(ctx: Context<CreateMint>, decimals: u8) -> Result<()> {
        let mint = &mut ctx.accounts.mint;
        mint.authority = ctx.accounts.payer.key();
        mint.decimals = decimals;
        mint.supply = 0;
        mint.is_initialized = true;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + RwaMint::SIZE
    )]
    pub mint: Account<'info, RwaMint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct RwaMint {
    pub authority: Pubkey,    // 32
    pub decimals: u8,         // 1
    pub supply: u64,          // 8
    pub is_initialized: bool, // 1
}

impl RwaMint {
    pub const SIZE: usize = 32 + 1 + 8 + 1;
}