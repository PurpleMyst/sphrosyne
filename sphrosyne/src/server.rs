use std::{
    io::{self, Cursor},
    sync::mpsc::{Receiver, Sender},
    thread::spawn,
};

use build_html::{Html, HtmlContainer, HtmlPage};
use eyre::{format_err, Result};
use image::GenericImage;
use qrcodegen::{QrCode, QrCodeEcc};
use slog::{debug, error, info, o, Logger};
use tiny_http::{Header, Request, Response, Server, StatusCode};
use tungstenite::{protocol::Role, Message, WebSocket};
use vigem_client_c::X360State;

use crate::request::PadRequest;

const QR_SCALE: u32 = 16;

/// Convert a key into a Sec-Websocket-Accept header
fn convert_key(key: &str) -> String {
    let mut key = key.to_string().into_bytes();
    key.extend("258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes());
    base64::encode(sha1::Sha1::from(key).digest().bytes())
}

/// Given a request that wants to become a websocket, make it become one and handle pad updates coming from it.
fn handle_websocket(logger: Logger, id: usize, req_tx: Sender<PadRequest>, request: Request) {
    let result: Result<()> = (|| {
        let key = &request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Sec-WebSocket-Key"))
            .ok_or_else(|| format_err!("no websocket key"))?
            .value;

        let response = Response::new_empty(StatusCode(101)).with_header(
            Header::from_bytes("Sec-WebSocket-Accept", convert_key(key.as_str())).unwrap(),
        );

        let stream = request.upgrade("websocket", response);
        let mut ws = WebSocket::from_raw_socket(stream, Role::Server, None);

        loop {
            let msg = match ws.read_message() {
                Ok(msg) => msg,
                Err(tungstenite::Error::ConnectionClosed) => return Ok(()),
                Err(error) => return Err(error.into()),
            };
            let data = match msg {
                Message::Text(data) => data.into_bytes(),
                Message::Binary(data) => data,
                Message::Ping(_) | Message::Pong(_) | Message::Close(_) => continue,
            };
            match serde_json::from_slice(&data) {
                Ok(state) => req_tx.send(PadRequest::Update(id, state))?,
                Err(error) => error!(logger, "ws.msg_error"; "error" => #%error),
            }
        }
    })();

    let _ = req_tx.send(PadRequest::Discard(id));

    if let Err(error) = result {
        error!(logger, "ws.error"; "error" => #%error);
    }
}

/// Generate a QR code from a given text and return it as a PNG data url
fn qr_data_url(text: &str) -> Result<String> {
    let qr = QrCode::encode_text(text, QrCodeEcc::Low)?;

    let side = qr.size();
    debug_assert!(<u32 as std::convert::TryFrom<i32>>::try_from(side).is_ok());
    let mut img = image::DynamicImage::new_rgb8(QR_SCALE * side as u32, QR_SCALE * side as u32);

    for y in 0..side {
        for x in 0..side {
            if !qr.get_module(x, y) {
                for i in 0..QR_SCALE {
                    for j in 0..QR_SCALE {
                        img.put_pixel(
                            i + QR_SCALE * x as u32,
                            j + QR_SCALE * y as u32,
                            image::Rgba([0xFF, 0xFF, 0xFF, 0xFF]),
                        );
                    }
                }
            }
        }
    }

    let mut data = Vec::new();
    img.write_to(&mut data, image::ImageOutputFormat::Png)?;

    Ok(format!("data:image/png;base64,{}", base64::encode(data)))
}

/// Return the HTML of the index page
fn index_page(port: u16) -> Result<String> {
    let host = gethostname::gethostname();
    let host = host
        .to_str()
        .ok_or_else(|| format_err!("Invalid hostname {:?}", host))?;
    let url = format!("http://{}:{}/controller", host, port);

    Ok(HtmlPage::new()
        .add_title("Sphrosyne")
        .add_meta(vec![
            ("charset", "utf8"),
            ("viewport", "width=device-width, initial-scale=1.0"),
        ])
        .add_style(include_str!("style.css"))
        .add_paragraph("The server is running. Scan the following QR code to connect your device:")
        .add_image(qr_data_url(&url)?, url)
        .to_html_string())
}

// Return the HTML of the controller page
fn controller_page(port: u16) -> Result<String> {
    let host = gethostname::gethostname();
    let host = host
        .to_str()
        .ok_or_else(|| format_err!("Invalid hostname {:?}", host))?;
    let url = format!("ws://{}:{}/websocket", host, port);

    Ok(HtmlPage::new()
        .add_title("Sphrosyne Controller")
        .add_meta(vec![
            ("charset", "utf8"),
            ("viewport", "width=device-width, initial-scale=1.0"),
        ])
        .add_style(include_str!("style.css"))
        // We pass in our websocket URL as a hidden input on the page so our javascript can retrieve it
        .add_raw(format_args!(
            r#"<input type="hidden" id="url" value="{}">"#,
            url
        ))
        .add_script_literal(include_str!("controller.js"))
        .to_html_string())
}

fn html_response(data: impl Into<String>) -> Response<Cursor<Vec<u8>>> {
    Response::from_string(data)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap())
}

pub(crate) fn mainloop(logger: Logger, tx: Sender<PadRequest>, rx: Receiver<usize>) -> Result<()> {
    let server = Server::http("0.0.0.0:0").map_err(|err| format_err!("no server :< {}", err))?;

    let addr = server.server_addr();
    let port = addr.port();
    info!(logger, "server.bound"; "addr" => addr, "url" => format_args!("http://localhost:{}", port));

    loop {
        let req = server.recv()?;
        debug!(logger, "req"; "req" => ?req, "headers" => ?req.headers());

        match req.url() {
            "/" => req.respond(html_response(index_page(port)?))?,

            "/controller" => req.respond(html_response(controller_page(port)?))?,

            "/websocket" => {
                let logger = logger.clone();
                tx.send(crate::PadRequest::NewID)?;
                let req_tx = tx.clone();

                let id = rx.recv()?;
                let logger = logger.new(o!("id" => id));
                info!(logger, "ws.new");
                spawn(move || handle_websocket(logger, id, req_tx.clone(), req));
            }

            _ => {
                let status_code = StatusCode(404);
                req.respond(Response::new(
                    status_code,
                    vec![],
                    io::Cursor::new(status_code.default_reason_phrase()),
                    Some(status_code.default_reason_phrase().len()),
                    None,
                ))?;
            }
        }
    }
}
