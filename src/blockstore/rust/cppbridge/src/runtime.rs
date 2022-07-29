pub struct LoggerInit {}
impl LoggerInit {
    pub fn new() -> Self {
        env_logger::init();
        Self {}
    }

    pub fn ensure_initialized(&self) {
        // noop. But calling this means the lazy static has to be created.
    }
}

lazy_static::lazy_static! {
    // TODO Will this runtime only execute things while an active call to rust is ongoing? Should we move it to a new thread
    //      and drive futures from there so that the runtime can execute even if we're currently mostly doing C++ stuff?
    pub static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    pub static ref LOGGER_INIT: LoggerInit = LoggerInit::new();
}
