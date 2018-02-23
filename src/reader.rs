use std::sync::{Arc, Mutex};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use result::Result;
use result::Error::IO;
use std::time::Duration;
use data;
use net;
use otp::{OTP, Port, Data, Ports};

pub struct Reader {
    lock: Mutex<Vec<data::SharedMessages>>,
    sock: UdpSocket,
}
impl Reader {
    pub fn new(port: u16) -> Result<Reader> {
        let ipv4 = Ipv4Addr::new(0, 0, 0, 0);
        let addr = SocketAddr::new(IpAddr::V4(ipv4), port);
        let srv = UdpSocket::bind(&addr)?;
        let timer = Duration::new(1, 0);
        srv.set_read_timeout(Some(timer))?;
        let rv = Reader {
            lock: Mutex::new(Vec::new()),
            sock: srv,
        };
        return Ok(rv);
    }
    pub fn recycle(&self, _ports: &Vec<Port>, d: Data) {
        match d {
            Data::SharedMessages(m) => {
                let mut gc = self.lock.lock().expect("lock");
                gc.push(m);
            }
            _ => (),
        }
    }
    pub fn run(&self, ports: &Ports) -> Result<()> {
        let mut m = self.allocate();
        let mut total = 0usize;
        {
            let v = Arc::get_mut(&mut m).expect("only ref");
            v.msgs.resize(1024, data::Message::default());
            v.data.resize(1024, data::Messages::def_data());
            trace!("reading");
            let r = net::read_from(&self.sock, &mut v.msgs, &mut v.data);
            trace!("reading done");
            match r {
                Err(IO(e)) => {
                    debug!("failed with IO error {:?}", e);
                }
                Err(e) => {
                    debug!("read failed error {:?}", e);
                }
                Ok(0) => {
                    trace!("read returned 0");
                }
                Ok(num) => {
                    let s: usize = v.data.iter_mut().map(|v| v.0).sum();
                    total += s;
                    v.msgs.resize(s, data::Message::default());
                    v.data.resize(num, data::Messages::def_data());
                }
            }
        }
        if total > 0 {
            OTP::send(ports, Port::State, Data::SharedMessages(m))?;
            return Ok(());
        } else {
            let mut gc = self.lock.lock().expect("lock");
            gc.push(m);
            return Ok(());
        }
    }
    fn allocate(&self) -> data::SharedMessages {
        let mut gc = self.lock.lock().expect("lock");
        gc.pop().unwrap_or_else(|| Arc::new(data::Messages::new()))
    }
}

#[cfg(test)]
use std::thread::spawn;
#[cfg(test)]
use std::thread::sleep;

#[test]
fn reader_test() {
    let reader = Arc::new(Reader::new(12001).expect("reader"));
    let c_reader = reader.clone();
    let exit = Arc::new(Mutex::new(false));
    let c_exit = exit.clone();
    let t = spawn(move || c_reader.run(c_exit));
    let cli: UdpSocket = net::socket().expect("socket");
    cli.connect("127.0.0.1:12001").expect("client");
    let timer = Duration::new(1, 0);
    cli.set_write_timeout(Some(timer)).expect("write timer");
    let m = [data::Message::default(); 64];
    let mut num = 0;
    let mut tries = 0;
    while num < 64 && tries < 100 {
        match net::write(&cli, &m[0..num + 1], &mut num) {
            Err(_) => sleep(Duration::new(0, 500000000)),
            _ => (),
        }
        tries += 1;
        trace!("write {:?}", num);
    }
    let mut rvs = 0usize;
    tries = 0;
    while rvs < 64 && tries < 100 {
        match reader.next() {
            Err(_) => {
                sleep(Duration::new(0, 500000000));
            }
            Ok(msgs) => {
                rvs += msgs.data.len();
            }
        }
        tries += 1;
        trace!("read {:?} {:?}", rvs, tries);
    }
    *exit.lock().expect("lock") = true;
    let o = t.join().expect("thread join");
    o.expect("thread output");
    assert_eq!(rvs, 64);
}
