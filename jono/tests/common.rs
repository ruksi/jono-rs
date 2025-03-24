use jono_core::{Context, Forum, get_redis_url};

pub fn create_test_context(topic: &str) -> Context {
    let redis_url = get_redis_url("redis://localhost:6380");
    let forum = Forum::new(&redis_url).expect("Failed to connect to Redis");
    forum.topic(topic)
}
