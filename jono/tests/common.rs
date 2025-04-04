use jono_core::{Context, Forum};

pub fn create_test_context(topic: impl ToString) -> Context {
    Forum::new("redis://localhost:6380")
        .expect("Failed to connect to Redis")
        .topic(topic)
}
