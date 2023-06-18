use anyhow::Result;

pub fn log_errors<R>(f: impl FnOnce() -> Result<R>) -> Result<R> {
    match f() {
        Ok(ok) => Ok(ok),
        Err(err) => {
            log::error!("Error: {:?}", err);
            Err(err)
        }
    }
}
