use anchor_lang::prelude::*;
use switchboard_on_demand::on_demand::accounts::pull_feed::PullFeedAccountData;

declare_id!("3Co44vnKtvUd1RshCrEnFUferHi1x5sWwjYKuYco2yjU");

#[derive(Accounts)]
pub struct Test<'info> {
    /// CHECK: I'm sure I'm entering the right feed account, or do a check in the ixn maybe
    pub feed: AccountInfo<'info>,
}

#[program]
pub mod feed_onchain {
    use super::*;

    pub fn test<'a>(ctx: Context<Test>, slot: u64) -> Result<()> {
        let feed_account = ctx.accounts.feed.data.borrow();

        let feed = PullFeedAccountData::parse(feed_account).unwrap();

        // let clock_slot = Clock::get()?.slot;
        msg!(
            "Umbra Price: {:?}",
            feed.get_value(slot, 100, 1, true).unwrap()
        );
        Ok(())
    }
}

// TODO: solve NotEnoughSamples issue
