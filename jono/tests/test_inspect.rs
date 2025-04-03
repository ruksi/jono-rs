mod common;

use common::create_test_context;
use jono::prelude::*;
use jono_core::generate_job_id;

#[test]
fn test_job_not_found_for_metadata() {
    let context = create_test_context("test_not_found");
    let inspector = Inspector::with_context(context);
    let unknown_job_id = generate_job_id();
    assert!(matches!(
        inspector.get_job_metadata(&unknown_job_id).err().unwrap(),
        JonoError::JobNotFound(_)
    ));
}
