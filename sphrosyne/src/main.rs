use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::spawn,
};

use eyre::Result;
use slab::Slab;
use slog::{info, trace, Logger};
use vigem_client_c::client::{Client, Target};

use crate::request::PadRequest;

fn setup_logging() -> Logger {
    use slog::Drain;
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    Logger::root(drain, slog::o!())
}

mod request;

mod server;

fn handle_pads(logger: Logger, req_rx: Receiver<PadRequest>, id_tx: Sender<usize>) -> Result<()> {
    let client = Client::new()?;

    let mut pads = Slab::<Target<_>>::new();

    loop {
        match req_rx.recv()? {
            PadRequest::NewID => {
                let id = pads.insert(client.connect_x360_pad()?);
                info!(logger, "pad.id.request"; "id" => id);
                id_tx.send(id)?;
            }

            PadRequest::Discard(id) => {
                info!(logger, "pad.id.discard"; "id" => id);
                pads.remove(id);
            }

            PadRequest::Update(id, state) => {
                trace!(logger, "pad.update"; "id" => id, "state" => ?state);
                pads[id].update(state)?;
            }
        }
    }
}

fn main() -> Result<()> {
    let logger = setup_logging();
    let (msg_tx, msg_rx) = channel();
    let (id_tx, id_rx) = channel();
    {
        let logger = logger.clone();
        spawn(move || server::mainloop(logger, msg_tx, id_rx));
    }
    handle_pads(logger, msg_rx, id_tx)
}
