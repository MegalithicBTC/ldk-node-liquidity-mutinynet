use std::sync::Arc;
use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
use ldk_node_liquidity_mutinynet::setup_node;

fn main() {
    // ── paths ──────────────────────────────────────────────────────────────
    let storage_dir = "tmp".to_string();
    let log_path = format!("{}/make_invoice.log", storage_dir);

    // ── setup node ─────────────────────────────────────────────────────────
    let node = setup_node(storage_dir, log_path.clone());

    // ── create invoice for 10,000 satoshis ─────────────────────────────────
    let desc = Bolt11InvoiceDescription::Direct(
        Description::new("test-invoice-10000-sats".into()).unwrap(),
    );
    let invoice = node
        .bolt11_payment()
        .receive_via_jit_channel(10_000, &desc, 3_600, None)
        .unwrap();

    println!("INVOICE for 10,000 satoshis:\n{invoice}\n");
    println!("Logs ➜ {log_path}");

    // keep running so the LSP can actually open the channel
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}