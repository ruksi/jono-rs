use jono_core::{Context, Error, Forum, Inspector, generate_job_id, get_redis_url};

#[test]
fn test_job_not_found_for_metadata() {
    let context = create_test_context("test_not_found");
    let inspector = Inspector::with_context(context);
    let unknown_job_id = generate_job_id();
    assert!(matches!(
        inspector.get_job_metadata(&unknown_job_id).err().unwrap(),
        Error::NotFound(_)
    ));
}

fn create_test_context(topic: &str) -> Context {
    let redis_url = get_redis_url();
    let forum = Forum::new(&redis_url).expect("Failed to connect to Redis");
    forum.topic(topic)
}
