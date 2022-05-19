use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::{info, warn, LevelFilter};
use rscp::{tags, GetItem};
use rscp::{Frame, Item};

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

    let mut c = rscp::Client::new("RSCP_KEY", "RSCP_USER".to_string(), "RSCP_PASSWORD".to_string());
    match c.connect("energy.storage.local", None) {
        Ok(_) => (),
        Err(err) => {
            panic!("Unable to connect: {:?}", err);
        }
    }

    let mut info_frame = Frame::new();
    info_frame.push_item(Item { tag: tags::INFO::SERIAL_NUMBER.into(), data: None } );
    info_frame.push_item(Item { tag: tags::INFO::MAC_ADDRESS.into(), data: None } );
    info_frame.push_item(Item { tag: tags::INFO::SW_RELEASE.into(), data: None } );

    match c.send_receive_frame(&info_frame) {
        Ok(result_frame) => {
            info!("{}", result_frame.get_item_data::<String>(tags::INFO::SERIAL_NUMBER.into()).unwrap());            
        },
        Err(err) => {
            warn!("Unable send: {:?}", err);
        }
    }

    c.disconnect().unwrap();
    
}
