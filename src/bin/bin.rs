use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::{info, warn, LevelFilter};
use rscp::tags;
use rscp;

fn setup_logging() {
    Builder::new()
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] {} - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)
        .init();
}

fn main() {

    setup_logging();

    info!("Hello, world!");
    info!("{:08x?}", tags::EMS::POWER_PV as u32);

    let g = tags::TagGroup::from(0);    
    info!("{}", g.tags(1));

    let mut c = rscp::Client::new("efwef", "eefwef".to_string(), "wgwegf".to_string());
    match c.connect("127.0.0.1", None) {
        Ok(_) => (),
        Err(err) => {
            warn!("Unable to connect: {:?}", err);
        }
    }
    
}
