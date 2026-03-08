use anchor_lang::prelude::*;

declare_id!("AH4kBFYyJiR1aFkCuJ6zC4PyqiUxsR2hJTFVdfqoPZYn");

#[program]
pub mod scoreboard{
    use super::*;

    pub fn initialize_scoreboard(ctx : Context<InitalizeScoreboard>, matches: u64, place: String, start_time : u64, end_time : u64) -> Result<()>{
        let scoreboard = &mut ctx.accounts.scoreboard;
        scoreboard.matches = matches;
        scoreboard.place = place;
        scoreboard.start_time = start_time;
        scoreboard.end_time = end_time;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(matches : u64)]
pub struct InitalizeScoreboard<'info>{
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(
        init,
        payer = user,
        space = 8 + Scoreboard::INIT_SPACE,
        seeds = [matches.to_le_bytes().as_ref()],
        bump
    )]
    pub scoreboard: Account<'info, Scoreboard>,
}


#[account]
#[derive(InitSpace)]
pub struct Scoreboard {
    pub matches : u64,
    #[max_len(50)]
    pub place: String,
    pub start_time: u64,
    pub end_time: u64,
}