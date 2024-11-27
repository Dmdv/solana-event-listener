use log::info;
use solana_tools::solana_logs::solana_event_listener::SolanaEventListener;
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    config::ListenConfig, postgres_writer::PostgresWriter,
    solana_logs_processor::MintedEventProcessor,
};

pub(crate) struct ListenerApp {
    solana_listener: SolanaEventListener,
    solana_logs_proc: MintedEventProcessor,
    postgres_writer: PostgresWriter,
    listen_config: ListenConfig,
}

impl ListenerApp {
    pub(crate) async fn execute(config_path: &str) {
        info!("Application restarted {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        let Ok(config) = ListenConfig::try_from_path(config_path) else {
            return;
        };

        let mut app = ListenerApp::new(config);
        app.execute_impl().await;
    }

    fn new(listen_config: ListenConfig) -> ListenerApp {
        let (logs_sender, logs_receiver) = unbounded_channel();
        let (mint_data_sender, mint_data_receiver) = unbounded_channel();
        ListenerApp {
            solana_listener: SolanaEventListener::new(listen_config.solana.clone(), logs_sender),
            solana_logs_proc: MintedEventProcessor::new(
                listen_config.solana.clone(),
                logs_receiver,
                mint_data_sender,
            ),
            postgres_writer: PostgresWriter::new(
                listen_config.postgres.clone(),
                mint_data_receiver,
            ),
            listen_config,
        }
    }

    async fn execute_impl(&mut self) {
        let Ok(tx_read_from) = self.listen_config.get_tx_read_from().await else {
            return;
        };
        info!("Last processed tx: {}", tx_read_from);
        tokio::select! {
            _ = self.solana_listener.listen_to_solana(tx_read_from) => {}
            _ = self.solana_logs_proc.execute() => {},
            _ = self.postgres_writer.execute() => {}
        };
    }
}
