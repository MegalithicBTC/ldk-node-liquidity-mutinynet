use std::io;
use std::io::prelude::*;
use std::sync::Arc;

use ldk_node::config::Config;
use ldk_node::Builder;

use ldk_node::bitcoin::Network;

fn main() {
	let mut config = Config::default();
	config.network = Network::Signet;

	let mut builder = Builder::from_config(config);
	builder.set_storage_dir_path("/tmp/ldk_node_liquidity_poc/".to_string());
	builder.set_log_level(ldk_node::LogLevel::Trace);

	builder.set_chain_source_esplora("https://mutinynet.com/api/".to_string(), None);

	let lsp_node_id = "...".parse().unwrap();
	let lsp_address = "...".parse().unwrap();
	let lsp_token = Some("...".to_string());

	builder.set_liquidity_source_lsps2(lsp_node_id, lsp_address, lsp_token);

	let node = Arc::new(builder.build().unwrap());
	node.start().unwrap();

	let event_node = Arc::clone(&node);
	std::thread::spawn(move || loop {
		let event = event_node.wait_next_event();
		println!("GOT NEW EVENT: {:?}", event);
		let _ = event_node.event_handled();
	});

	let invoice =
		node.bolt11_payment().receive_via_jit_channel(989322000, &"asdf", 3600, None).unwrap();

	println!("GOT JIT INVOICE: {}", invoice);
	pause();

	node.stop().unwrap();
}

fn pause() {
	let mut stdin = io::stdin();
	let mut stdout = io::stdout();

	// We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
	write!(stdout, "Press any key to continue...").unwrap();
	stdout.flush().unwrap();

	// Read a single byte and discard
	let _ = stdin.read(&mut [0u8]).unwrap();
}
