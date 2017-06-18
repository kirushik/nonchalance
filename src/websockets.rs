use ws::{Handler, Sender, Result, Message, Handshake, CloseCode};

pub struct WSHandler {
    out: Sender,
    trusted_origin: bool
}

impl WSHandler {
    pub fn new(out: Sender) -> WSHandler {
        WSHandler{out: out, trusted_origin: false}
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
            self.trusted_origin = origin==Some("http://127.0.0.1:3202");
            Ok(())
        }
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("{:?}", msg);
        self.out.send("Принято!")
    }
}