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
There are multiple user levels (NO_AUTH => USER ... => ADMIN => E3DC_ROOT). For most cases NO_AUTH and USER level access is enough. 
Therefore before sending a request towards you E3DC system we need to authenticate first. 

Every requests towards RSCP system consists of a rscp::Frame (container) with multiple rscp::Item with the later holding the tags and data.   

## Login and First Frame
`RSCP_KEY` is the password you define on your S10 screen in Settings => Personalize => User Profile +> local.user    
`RSCP_USER` is  is the username (i.e. email ) in your S10 portal.  
`RSCP_PASSWORD` is the password in your S10 portal 

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

## Compose Battery Information Request

Similarly to the official example of the rscp call to request battery information, we can also define a containered request. The `BAT::DATA` tag defines that the following tags 
should be evaluated to receive some data about the battery. 
```rust
let mut battery_info_frame = Frame::new();
battery_info_frame.push_item(rscp::Item::new(rscp::tags::BAT::DATA.into(),
    vec![
        rscp::Item { tag: rscp::tags::BAT::INDEX.into(), data: Some(Box::new(0)), },
        rscp::Item { tag: rscp::tags::BAT::RSOC.into(), data: None },
        rscp::Item { tag: rscp::tags::BAT::MODULE_VOLTAGE.into(), data: None },
        rscp::Item { tag: rscp::tags::BAT::CURRENT.into(), data: None },
        rscp::Item { tag: rscp::tags::BAT::STATUS_CODE.into(), data: None },
        rscp::Item { tag: rscp::tags::BAT::ERROR_CODE.into(), data: None },
    ]));
```

Expected output of `println!("Result: {:?}", result_frame.items.get_item(DATA.into()));` inside `match`

```rust
Result: Ok(Item 
{ tag: "BAT_DATA", data: [
    Item { tag: "BAT_INDEX", data: 0 }, 
    Item { tag: "BAT_RSOC", data: 38.218014 }, 
    Item { tag: "BAT_MODULE_VOLTAGE", data: 51.7 }, 
    Item { tag: "BAT_CURRENT", data: 108.9 }, 
    Item { tag: "BAT_STATUS_CODE", data: 0 }, 
    Item { tag: "BAT_ERROR_CODE", data: 0 }
] })
```

## Tags vs Tags
Be careful which tag you use in your requests because some tags set values instead of getting them. The official example app documentation and excel table provide necessary information
to distinguish between these tags. 

The structure of tags in this project is similar to the documentation of tags but not necessarily to the tags in the official example app. 
For Example this official tag: `#define TAG_BAT_INDEX 0x03040001` corresponds to project tag:

rscp::tags::BAT::INDEX: 
- BAT = 0x03,
- INDEX = 0x040001,

The advantage lies in the reuse of tag names and therefore in readability of code. 

| Project Tags           | Example App Tags               |
-------------------------|--------------------------------|
| PVI::DATA = 0x040000,  | TAG_PVI_DATA = 0x02840000      |
| BAT::DATA = 0x040000,  | TAG_BAT_INDEX = 0x03840000     |
| DCDC::DATA = 0x040000, | TAG_DCDC_REQ_DATA = 0x04040000 |
| ...                    | ...                            |