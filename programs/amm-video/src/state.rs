use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub seed: u64,                 // Seed to be able to create different pools / configs
    pub authority: Option<Pubkey>, // If we want an authority to lock the config account
    pub mint_x: Pubkey,            // Token X
    pub mint_y: Pubkey,            // Token Y
    pub fee: u16,                  // Swap fee in basis points
    pub protocol_fee: u16,         // bps routed to the treasury, appended after fee
    pub locked: bool,              // If the pool is locked
    pub config_bump: u8,           // Bump seed for the config account
    pub lp_bump: u8,               // Bump seed for the LP token
    pub treasury_bump: u8,        // Bump seed for the treasury accounts
}

// the config is the configuration for the AMM, 
// it holds the mints of the two tokens, the fee, and the bump seeds for the config and LP token accounts. 
// The config account is a PDA derived from the seed and the program id. 
// The config account is also used as the authority for the vaults and LP token accounts.