# Bincode-Epee

[![Build Status](https://travis-ci.com/servo/bincode.svg)](https://travis-ci.com/servo/bincode)
[![](https://meritbadge.herokuapp.com/bincode)](https://crates.io/crates/bincode)
[![](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)

_About: This is bincode with certain changes to match C++ epee's serialization. It does not support deserialization_

A compact encoder that uses a binary zero-fluff encoding scheme.
The size of the encoded object will be the same or smaller than the size that
the object takes up in memory in a running Rust program.

## [API Documentation](https://docs.rs/bincode/)

## Bincode in the wild

* [google/tarpc](https://github.com/google/tarpc): Bincode is used to serialize and deserialize networked RPC messages.
* [servo/webrender](https://github.com/servo/webrender): Bincode records webrender API calls for record/replay-style graphics debugging.
* [servo/ipc-channel](https://github.com/servo/ipc-channel): IPC-Channel uses Bincode to send structs between processes using a channel-like API.

## Example

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World(Vec<Entity>);

fn main() {
    let world = World(vec![Entity { x: 0.0, y: 4.0 }, Entity { x: 10.0, y: 20.5 }]);

    let encoded: Vec<u8> = bincode::serialize(&world).unwrap();

    // 8 bytes for the length of the vector, 4 bytes per float.
    assert_eq!(encoded.len(), 8 + 4 * 4);

    let decoded: World = bincode::deserialize(&encoded[..]).unwrap();

    assert_eq!(world, decoded);
}
```

## Details

The encoding (and thus decoding) proceeds unsurprisingly -- primitive
types are encoded according to the underlying `Writer`, tuples and
structs are encoded by encoding their fields one-by-one, and enums are
encoded by first writing out the tag representing the variant and
then the contents.

However, there are some implementation details to be aware of:

* `isize`/`usize` are encoded as `i64`/`u64`, for portability.
* enums variants are encoded as a `u32` instead of a `usize`.
  `u32` is enough for all practical uses.
* `str` is encoded as `(u64, &[u8])`, where the `u64` is the number of
  bytes contained in the encoded string.
