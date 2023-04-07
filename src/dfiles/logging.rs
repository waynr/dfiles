use crate::error::Result;
use log::LevelFilter;

pub fn setup(level: u64) -> Result<()> {
    let level = match level {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let mut logger_builder = &mut pretty_env_logger::formatted_builder();

    logger_builder = logger_builder.filter_level(level);
    if level == LevelFilter::Info {
        logger_builder = logger_builder.default_format();
        logger_builder = logger_builder.format_module_path(false);
        logger_builder = logger_builder.format_level(false);
        logger_builder = logger_builder.format_timestamp(None);
    }

    logger_builder.try_init()?;
    Ok(())
}
