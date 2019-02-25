mod roblox_install;
mod message_receiver;
mod old_stuff;

use clap::App;

fn main() {
    {
        let log_env = env_logger::Env::default()
            .default_filter_or("warn");

        env_logger::Builder::from_env(log_env)
            .default_format_timestamp(false)
            .init();
    }

    let app = App::new("run-in-roblox")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"));

    let matches = app.get_matches();
}
