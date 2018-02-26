//! data structures for the protocol, data types must have little endian C99 layout, no gaps, and same layout on LP64 and LLP64 and other variants.
//!
//! TBD a lightweight serialization format.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, RwLock};
use hasht::{HashT, Key, Val};
use result::Result;
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Transaction {
    pub to: [u8; 32],
    pub amount: u64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct POH {
    pub hash: [u8; 32],
    pub counter: u64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Signature {
    pub data: [u8; 64],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Subscriber {
    pub key: [u8; 32],
    pub addr: [u8; 4],
    pub port: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct GetLedger {
    pub start: u64,
    pub num: u64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CheckBalance {
    pub key: [u8; 32],
    pub amount: u64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union MessageData {
    pub tx: Transaction,
    pub poh: POH,
    pub sub: Subscriber,
    pub get: GetLedger,
    pub bal: CheckBalance,
}

impl Default for MessageData {
    fn default() -> MessageData {
        MessageData {
            tx: Transaction::default(),
        }
    }
}

#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum Kind {
    Invalid,
    Transaction,
    Signature,
    Subscribe,
    GetLedger,
    CheckBalance,
}

impl Default for Kind {
    fn default() -> Kind {
        Kind::Invalid
    }
}
impl Copy for Kind {}

impl Clone for Kind {
    fn clone(&self) -> Kind {
        *self
    }
}

#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum State {
    Unknown,
    Withdrawn,
    Deposited,
}
impl Copy for State {}

impl Clone for State {
    fn clone(&self) -> State {
        *self
    }
}

impl Default for State {
    fn default() -> State {
        State::Unknown
    }
}
pub const MAX_PACKET: usize = 1024 * 4;

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Payload {
    pub from: [u8; 32],
    pub lvh: [u8; 32],
    pub lvh_count: u64,
    pub fee: u64,
    pub data: MessageData,
    pub version: u32,
    pub kind: Kind,
    pub state: State, //zero when signed
    pub unused: u16,  //zero when signed
}

impl Payload {
    pub fn get_tx(&self) -> &Transaction {
        assert_eq!(self.kind, Kind::Transaction);
        unsafe { &self.data.tx }
    }
    pub fn get_tx_mut(&mut self) -> &mut Transaction {
        assert_eq!(self.kind, Kind::Transaction);
        unsafe { &mut self.data.tx }
    }
    pub fn get_sub(&self) -> &Subscriber {
        assert_eq!(self.kind, Kind::Subscribe);
        unsafe { &self.data.sub }
    }
    pub fn get_poh(&self) -> &POH {
        assert_eq!(self.kind, Kind::Signature);
        unsafe { &self.data.poh }
    }
    pub fn get_get(&self) -> &GetLedger {
        assert_eq!(self.kind, Kind::GetLedger);
        unsafe { &self.data.get }
    }
    pub fn get_bal(&self) -> &CheckBalance {
        assert_eq!(self.kind, Kind::CheckBalance);
        unsafe { &self.data.bal }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Message {
    pub pld: Payload,
    pub sig: [u8; 64],
}

impl Default for Message {
    fn default() -> Message {
        Message {
            pld: Payload::default(),
            sig: [0u8; 64],
        }
    }
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Account {
    pub from: [u8; 32],
    pub balance: u64,
}

impl Key for [u8; 32] {
    fn start(&self) -> usize {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let st = ((self[0] as u64) << ((7 - 0) * 8)) |
                 ((self[1] as u64) << ((7 - 1) * 8)) |
                 ((self[2] as u64) << ((7 - 2) * 8)) |
                 ((self[3] as u64) << ((7 - 3) * 8)) |
                 ((self[4] as u64) << ((7 - 4) * 8)) |
                 ((self[5] as u64) << ((7 - 5) * 8)) |
                 ((self[6] as u64) << ((7 - 6) * 8)) |
                 ((self[7] as u64) << ((7 - 7) * 8)) ;
        st as usize
    }
    fn unused(&self) -> bool {
        *self == [0u8; 32]
    }
}

impl Val<[u8; 32]> for Account {
    fn key(&self) -> &[u8; 32] {
        &self.from
    }
}
pub type AccountT = HashT<[u8; 32], Account>;

#[derive(Clone)]
pub struct Messages {
    pub msgs: Vec<Message>,
    pub data: Vec<(usize, SocketAddr)>,
}

impl Messages {
    pub fn new() -> Messages {
        Messages {
            msgs: vec![Message::default(); 1024],
            data: vec![Self::def_data(); 1024],
        }
    }
    pub fn def_data() -> (usize, SocketAddr) {
        let ipv4 = Ipv4Addr::new(0, 0, 0, 0);
        let addr = SocketAddr::new(IpAddr::V4(ipv4), 0);
        (0, addr)
    }
    pub fn with<F, A>(&mut self, f: F) -> Result<A>
    where
        F: Fn(&mut Vec<Message>, &mut Vec<(usize, SocketAddr)>) -> Result<A>,
    {
        f(&mut self.msgs, &mut self.data)
    }
}

pub type SharedMessages = Arc<RwLock<Messages>>;
