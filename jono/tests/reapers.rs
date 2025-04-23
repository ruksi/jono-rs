#![cfg(feature = "harvest")]

use jono::prelude::*;
use jono_core::Result;

pub struct NoopReaper;

impl Reaper for NoopReaper {
    async fn reap(&self, _: &Reapload) -> Result<ReapSummary> {
        Ok(ReapSummary::Success(None))
    }
}
