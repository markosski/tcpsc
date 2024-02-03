# servertest

Testing server - client communication over TCP.

* allow connection from multiple clients
* establish long lived TCP connection with timeout
* use simple communication protocol (command_name, cliend_id, data)

## Running locally

in terminal 1

`cd server`

`cargo run -- 7878`

in terminal 2

`cd client`

`cargo run -- 7878`
