# rscp

[![Crate](https://img.shields.io/crates/v/rscp.svg)](https://crates.io/crates/rscp)
[![Docs](https://docs.rs/rscp/badge.svg)](https://docs.rs/rscp)
[![License](https://img.shields.io/crates/l/rscp.svg?maxAge=2592000)](https://github.com/mr-sven/rscp/blob/main/LICENSE)
[![Coverage](https://img.shields.io/badge/coverage-89.48%25-green)](https://github.com/mr-sven/rscp)

This lib is a Rust based E3/DC RSCP connector.

 - omit the use of double namings and wrappings
 - implemented native type parsing without wrapping types

The response bit in the Tag field is ignored due the fact there is no known need to take care on this.

Ths lib is in early stage.

# Usage

```rust
use rscp::GetItem;

let mut c = rscp::Client::new("RSCP_KEY", "RSCP_USER".to_string(), "RSCP_PASSWORD".to_string());
match c.connect("energy.storage.local", None) {
    Ok(_) => (),
    Err(err) => {
        panic!("Unable to connect: {:?}", err);
    }
}

let mut info_frame = rscp::Frame::new();
info_frame.push_item(rscp::Item { tag: rscp::tags::INFO::SERIAL_NUMBER.into(), data: None } );
info_frame.push_item(rscp::Item { tag: rscp::tags::INFO::MAC_ADDRESS.into(), data: None } );
info_frame.push_item(rscp::Item { tag: rscp::tags::INFO::SW_RELEASE.into(), data: None } );

match c.send_receive_frame(&info_frame) {
    Ok(result_frame) => {
        println!("{}", result_frame.get_item_data::<String>(rscp::tags::INFO::SERIAL_NUMBER.into()).unwrap());
    },
    Err(err) => {
        warn!("Unable send: {:?}", err);
    }
}

c.disconnect().unwrap();
```
