use ws::{self, Handler, Result, Message, Handshake, CloseCode};
use url::Url;

use std::sync::mpsc;

pub type GuiCallbackChannel = mpsc::Sender<String>;
pub type GuiRequestChannel = mpsc::Sender<Option<(Url, GuiCallbackChannel)>>;

pub struct WSHandler {
    out: ws::Sender,
    url_request_sender: GuiRequestChannel,
    origin: Option<Url>,
    trust_origin_in_data: bool
}

impl WSHandler {
    pub fn new(out: ws::Sender, url_request_sender: GuiRequestChannel) -> WSHandler {
        WSHandler{out: out, url_request_sender: url_request_sender, origin: None, trust_origin_in_data: false}
    }
}

impl Handler for WSHandler {
    fn on_open(&mut self, handshake: Handshake) -> Result<()> {
        let origin = handshake.request.origin().unwrap_or(None);
        let valid_connection = origin.is_some() &&
            handshake.peer_addr.map(|addr| addr.ip().is_loopback()).unwrap_or(false);
        if !valid_connection {
            self.out.close_with_reason(CloseCode::Error, "You're bad, invalid connection!")

        } else {
            self.trust_origin_in_data = origin==Some("http://127.0.0.1:3202");
            self.origin = origin.and_then(|origin| Url::parse(origin).ok());
            Ok(())
        }
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("{:?}", msg);

        let (sender, receiver) = mpsc::channel::<String>();
        let url = if self.trust_origin_in_data {
            unimplemented!()
        } else {
            self.origin.clone().expect("Hey, this Origin header has just been there recently!")
        };
        self.url_request_sender.send(Some((url, sender))).expect("Cannot communicate with GUI thread");
        let payload = receiver.recv().unwrap();
        self.url_request_sender.send(None).expect("Cannot communicate with GUI thread"); // Unsetting the callback
        self.out.send(payload)
    }
}