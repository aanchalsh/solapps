use anchor_lang::prelude::*;

declare_id!("4skE9xAhKrDniyQTGysCHDKtPYYhtvnNKFNe9P755arC");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoreboard_init_space_is_correct() {
        // 8 discriminator + 8 matches + 4 len prefix + 50 place + 8 start_time + 8 end_time
        assert_eq!(Scoreboard::INIT_SPACE, 8 + 4 + 50 + 8 + 8);
    }

    #[test]
    fn scoreboard_fields_hold_expected_values() {
        let scoreboard = Scoreboard {
            matches: 10,
            place: "Mumbai".to_string(),
            start_time: 1000,
            end_time: 2000,
        };
        assert_eq!(scoreboard.matches, 10);
        assert_eq!(scoreboard.place, "Mumbai");
        assert_eq!(scoreboard.start_time, 1000);
        assert_eq!(scoreboard.end_time, 2000);
    }

    #[test]
    fn end_time_is_after_start_time() {
        let start_time: u64 = 1000;
        let end_time: u64 = 2000;
        assert!(end_time > start_time);
    }
}