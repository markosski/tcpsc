# servertest

Testing server - client communication over TCP.

* allow connection from multiple clients
* establish long lived TCP connection with timeout
* use simple communication protocol (command_name, cliend_id, data)

## Running examples locally

in terminal 1

`cd examples/server`

`RUST_LOG=debug cargo run -- 7878`

in terminal 2

`cd examples/client`

`RUST_LOG=debug cargo run -- 7878`
