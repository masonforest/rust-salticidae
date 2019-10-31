# The Salticidae protocol in idiomatic Rust

_This is just a proof of concept I wrote to learn how the protocol works. See: [Determinant/salticidae](https://github.com/Determinant/salticidae)_ for a fully functional implementation.


### Install

Add the following to your `Cargo.toml`


    [dependencies]
    salticidae = { git = 'https://github.com/masonforest/rust-salticidae' }

#### Test

    $ cargo test

### Demo

    $ cargo run --example test_msgnet
    [alice] connected, sending hello.
    [bob] connected, sending hello.
    [bob] accepted, waiting for greetings.
    [alice] accepted, waiting for greetings.
    [alice] bob says Hello there!
    [bob] alice says Hello there!
    [bob] the peer knows
    [alice] the peer knows

### Protocol documentation

See: [Protocol Spec](https://github.com/masonforest/go-salticidae-native/blob/master/PROTOCOL_SPEC.md)
