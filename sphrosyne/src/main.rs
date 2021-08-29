use std::{
    sync::mpsc::{channel, Sender},
    thread::spawn,
};

use eyre::Result;

use slab::Slab;
use slog::{error, info, trace, Logger};
use tiny_http::{Header, Request, Response, StatusCode};
use tungstenite::{protocol::Role, Message, WebSocket};
use vigem_client_c::{client::Target, Client, X360State};

fn setup_logging() -> Logger {
    use slog::Drain;
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    Logger::root(drain, slog::o!())
}

mod server;

enum ChanMessage {
    NewID,
    Discard(usize),
    Update(usize, X360State),
}

fn convert_key(input: &str) -> String {
    let mut input = input.to_string().into_bytes();
    input.extend("258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes());
    base64::encode(sha1::Sha1::from(input).digest().bytes())
}

fn handle_websocket(logger: Logger, id: usize, tx: Sender<ChanMessage>, request: Request) {
    let result: Result<()> = (|| {
        let key = &request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Sec-WebSocket-Key"))
            .ok_or_else(|| eyre::format_err!("no websocket key"))?
            .value;

        let mut response = Response::new_empty(StatusCode(101));
        response.add_header(
            Header::from_bytes("Sec-WebSocket-Accept", convert_key(key.as_str())).unwrap(),
        );

        let stream = request.upgrade("websocket", response);
        let mut ws = WebSocket::from_raw_socket(stream, Role::Server, None);

        loop {
            let msg = ws.read_message()?;
            let data = match msg {
                Message::Text(data) => data.into_bytes(),
                Message::Binary(data) => data,
                Message::Ping(_) | Message::Pong(_) | Message::Close(_) => continue,
            };
            let state: X360State = serde_json::from_slice(&data)?;
            tx.send(ChanMessage::Update(id, state))?;
        }
    })();

    let _ = tx.send(ChanMessage::Discard(id));

    if let Err(error) = result {
        error!(logger, "ws.error"; "error" => #%error);
    }
}

fn handle_pads(logger: Logger) -> Result<()> {
    let client = Client::new()?;

    let (msg_tx, msg_rx) = channel();
    let (id_tx, id_rx) = channel();

    {
        let logger = logger.clone();
        spawn(move || server::mainloop(logger, msg_tx, id_rx));
    }

    let mut pads = Slab::<Target<_>>::new();

    loop {
        match msg_rx.recv()? {
            ChanMessage::NewID => {
                let id = pads.insert(client.connect_x360_pad()?);
                info!(logger, "pad.id.request"; "id" => id);
                id_tx.send(id)?;
            }

            ChanMessage::Discard(id) => {
                info!(logger, "pad.id.discard"; "id" => id);
                pads.remove(id);
            }

            ChanMessage::Update(id, state) => {
                trace!(logger, "pad.update"; "id" => id, "state" => ?state);
                pads[id].update(state)?;
            }
        }
    }
}

fn main() -> Result<()> {
    let logger = setup_logging();
    handle_pads(logger)
}
