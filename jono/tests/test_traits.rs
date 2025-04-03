use jono::prelude::*;
use jono_core::Result;

fn is_send_sync<T: Send + Sync>() {}

#[test]
fn these_are_send_sync() -> Result<()> {
    is_send_sync::<Context>();
    is_send_sync::<Workload>();
    Ok(())
}
