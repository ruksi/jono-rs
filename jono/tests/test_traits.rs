use jono_core::Context;
use jono_core::JonoError;
use jono_core::Result;

fn is_send_sync<T: Send + Sync>() {}

#[tokio::test]
async fn context_is_send_sync() -> Result<()> {
    is_send_sync::<Context>();
    Ok(())
}

#[cfg(all(feature = "consume"))]
#[tokio::test]
async fn workload_is_send_sync() -> Result<()> {
    use jono_consume::Workload;
    is_send_sync::<Workload>();
    Ok(())
}

fn is_error<T: std::error::Error>() {}

#[tokio::test]
async fn jono_error_is_error() -> Result<()> {
    is_error::<JonoError>();
    Ok(())
}
