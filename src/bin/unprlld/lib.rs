pub fn run() {
    info!("{}", format_args!("Unprll {} - {}", cryptonote_config::VERSION, cryptonote_config::RELEASE_NAME));

    cryptonote_core::init();
}
