# rscp

[![License:MIT](https://badgen.net/github/license/mr-sven/rscp)](https://opensource.org/licenses/MIT)
[![Coverage](https://badgen.net/badge/coverage/58.49%/yellow)](https://github.com/mr-sven/rscp)

This lib is a Rust based E3/DC RSCP connetor.

 - omit the use of double namings and wrappings
 - implemented native type parsing without wrapping types

The response bit in the Tag field is ignored due the fact there is no known need to take care on this.

Ths lib is in early stage.

# Usage

```rust
use rscp::tags;
use rscp;

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
        println!("{}", result_frame.get_item_data::<String>(tags::INFO::SERIAL_NUMBER.into()).unwrap());            
    },
    Err(err) => {
        warn!("Unable send: {:?}", err);
    }
}

c.disconnect().unwrap();
```
