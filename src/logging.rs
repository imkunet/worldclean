pub(crate) fn setup_logging() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.init();

    log_panics::init();
}
