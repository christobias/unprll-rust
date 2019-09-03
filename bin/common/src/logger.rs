use fern::colors::Color;
use log::info;

use crate::Config;

pub fn init(config: &Config, binary_name: &str) -> Result<(), fern::InitError> {
    let colors = fern::colors::ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::Green)
        .trace(Color::Magenta);

    let log_level = match config.log_level {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => panic!("Invalid log level")
    };

    let mut log_file_path = if let Some(custom_data_directory) = &config.data_directory {
        custom_data_directory.to_path_buf()
    } else {
        directories::ProjectDirs::from("cash", "Unprll Project", "Unprll").expect("Failed to get project user directory").data_dir().to_path_buf()
    };

    std::fs::create_dir_all(&log_file_path).unwrap_or_else(|err| {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Unexpected error when creating log directory {}", err);
        }
    });

    log_file_path.push(binary_name);
    log_file_path.set_extension("log");

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date}][{target}][{level}{color_line}]\t{message}\x1B[0m",
                color_line = format_args!("\x1B[{}m", colors.get_color(&record.level()).to_fg_str()),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors.color(record.level()),
                message = message,
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .chain(fern::log_file(&log_file_path)?)
        .apply()?;

    info!("Logging events to {}", log_file_path.to_str().unwrap());
    Ok(())
}
