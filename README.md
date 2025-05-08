Basic usage of the Megalith LSPS2 service, currently running on MutinyNet, can be demonstrated in the following way:

1. Clone this repository

2. `cargo run --bin megalith_lsps2` .. this generates a JIT invoice

3. pay the invoice normally, or with MPP. LSP opens a channel.

4. `cargo run --bin make_invoice` This generates a private invoice from the Client LDK node. 

You can pay the invoice from any node (or from the MutinyNet Faucet) and the balance will reflect in the LDK client.
