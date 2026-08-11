pub mod crypto;
pub mod storage;
pub mod server;
mod gui;
use std::sync::{Arc, Mutex}; 
use server::{start_server, AppStateServeur, SafeState}; 

fn main() -> eframe::Result<()> {
    let server_state: SafeState = Arc::new(Mutex::new(AppStateServeur {
        coffre_ouvert: None,
    }));

    start_server(Arc::clone(&server_state));

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "SafeVault",
        options,
        Box::new(|_cc| Box::new(gui::SafeVaultApp::new(server_state))),
    )
}
