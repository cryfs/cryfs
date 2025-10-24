use std::sync::Once;

static INITED: Once = Once::new();

pub fn init() {
    INITED.call_once(|| {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Off)
            .parse_default_env()
            .init();
    });
}
