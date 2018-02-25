use std::sync::{Arc, Mutex, RwLock};
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
    pub fn recycle(&self, d: Data) {
        match d {
            Data::SharedMessages(m) => {
                let mut gc = self.lock.lock().expect("lock");
                gc.push(m);
            }
            _ => (),
        }
    }

    fn read(&self, m: data::SharedMessages) -> Result<usize> {
            let mut v = m.write().unwrap();
            v.msgs.resize(1024, data::Message::default());
            v.data.resize(1024, data::Messages::def_data());
            unsafe {
                let msgs = &v.msgs as *mut data::Message;
                let data = &v.data as *mut (usize, SocketAddr);
                net::read_from(&self.sock, &mut msgs, &mut data)
            }
    }

    pub fn run(&self, ports: &Ports) -> Result<()> {
        let m = self.allocate();
        let mut total = 0usize;
        {
            trace!("reading");
            let r = self.read(m);
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
                    let mut v = m.write().unwrap();
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
        gc.pop().unwrap_or_else(|| Arc::new(RwLock::new(data::Messages::new())))
    }
}

#[cfg(test)]
mod test {
    use std::thread::sleep;
    use otp::{OTP, Port, Data};
    use std::sync::{Arc, Mutex};
    use std::net::UdpSocket;
    use reader::Reader;
    use std::time::Duration;
    use net;
    use data;
    
    #[test]
    fn reader_test() {
        let reader = Arc::new(Reader::new(12001).expect("reader"));
        let mut o = OTP::new();
        let a_reader = reader.clone();
        assert_eq!(Ok(()),
            o.source(Port::Reader, move |ports| {
                a_reader.run(ports)
            }));
        let b_reader = reader.clone();
        assert_eq!(Ok(()),
            o.listen(Port::Recycle, move |_ports, data| {
                b_reader.recycle(data);
                Ok(())
            }));
    
        let rvs = Arc::new(Mutex::new(0usize));
        let a_rvs = rvs.clone();
        assert_eq!(Ok(()), o.listen(Port::State, move |ports, data|
            match data {
                Data::SharedMessages(msgs) => {
                    let mut v = a_rvs.lock().unwrap();
                    *v += msgs.read().unwrap().data.len();
                    OTP::send(ports, Port::Recycle, Data::SharedMessages(msgs))?;
                    Ok(())
                }
                _ => Ok(()),
            }));
     
        let cli: UdpSocket = net::socket().expect("socket");
        cli.connect("127.0.0.1:12001").expect("client");
        let timer = Duration::new(1, 0);
        cli.set_write_timeout(Some(timer)).expect("write timer");
        let m = [data::Message::default(); 64];
        let mut num = 0;
        let mut tries = 0;
        while num < 64 && tries < 100 {
            match net::write(&cli, &m[0..num + 1], &mut num) {
                Err(_) => sleep(Duration::new(0, 50000000)),
                _ => (),
            }
            tries += 1;
            trace!("write {:?}", num);
        }
        sleep(Duration::new(1, 0));
        assert_eq!(Ok(()), o.shutdown());
        assert_eq!(*rvs.lock().unwrap(), 64);
    }
}
