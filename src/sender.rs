use result::Result;
use std::net::UdpSocket;
use net;
use otp::{Data, Port, Ports, OTP};

struct Sender {
    s: UdpSocket,
}
impl Sender {
    pub fn new() -> Result<Sender> {
        let s = net::socket()?;
        Sender { socket: s }
    }

    pub fn run(&self, d: Data) -> Result<()> {
        match d {
            Data::SendMessage(m, a) => {
                let msgs = [m];
                let mut num = 0;
                while num < 1 {
                    net::send_to(&self.s, &msgs, &num, a)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}
