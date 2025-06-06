use log::LevelFilter;
use simplelog::{ConfigBuilder, SimpleLogger};

pub fn setup_simple_logger() -> anyhow::Result<()> {
    let logger_config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .add_filter_allow_str("pet_info")
        .build();

    Ok(SimpleLogger::init(LevelFilter::Info, logger_config)?)
}
