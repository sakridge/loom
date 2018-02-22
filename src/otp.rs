//! see test for documentation

use std::sync::{RwLock, Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread::{JoinHandle, spawn};
use std::time::Duration;
use data;
use result::Result;
use result::Error;

pub enum Port {
    Reader,
    State,
    Max,
}

impl Port {
    fn to_usize(self) -> usize {
        match self {
            Port::Reader => 0,
            Port::State => 1,
            Port::Max => 2,
        }
    }
}

pub enum Data {
    Signal,
    SharedMessages(data::SharedMessages),
}

struct Locked {
    ports: Vec<Sender<Data>>,
    readers: Vec<Arc<Mutex<Receiver<Data>>>>,
    threads: Vec<Arc<Option<JoinHandle<Result<()>>>>>,
}

pub struct OTP {
    lock: Arc<RwLock<Locked>>,
    exit: Arc<Mutex<bool>>,
}

impl OTP {
    pub fn new() -> OTP {
        let (s1,r1) = channel();
        let (s2,r2) = channel();
        let locked = Locked {
            ports : [s1, s2].to_vec(),
            readers : [Arc::new(Mutex::new(r1)),
                       Arc::new(Mutex::new(r2))].to_vec(),
            threads : [Arc::new(None), Arc::new(None)].to_vec(),
        };
        let exit = Arc::new(Mutex::new(false));
        OTP {lock: Arc::new(RwLock::new(locked)), exit: exit}
    }
    pub fn source<F>(&self, port: Port, func: F) 
        where F: Send + 'static + Fn(Vec<Sender<Data>>) -> Result<()>
    {
        let mut w = self.lock.write().unwrap();
        let c_ports = w.ports.clone();
        let c_exit = self.exit.clone();
        let j = spawn(move|| loop {
            match func(c_ports.clone()) {
                Ok(()) => (),
                e => return e
            }
            if *c_exit.lock().unwrap() == true {
                return Ok(());
            }
        });
        w.threads[port.to_usize()] = Arc::new(Some(j));
    }
    pub fn listen<F>(&mut self, port: Port, func: F)
        where F: Send + 'static + Fn(Vec<Sender<Data>>, Data) -> Result<()>
    {
        let mut w = self.lock.write().unwrap();
        let pz = port.to_usize();
        let recv_lock = w.readers[pz].clone();
        let c_ports = w.ports.clone();
        let c_exit = self.exit.clone();
        let j: JoinHandle<Result<()>> = spawn(move|| loop {
            let recv = recv_lock.lock().unwrap();
            let timer = Duration::new(0, 500000);
            match recv.recv_timeout(timer) {
                Ok(val) => func(c_ports.clone(), val)?,
                _ => (),
            }
            if *c_exit.lock().unwrap() == true {
                return Ok(());
            }
        });
        w.threads[pz] = Arc::new(Some(j));
    }
    pub fn send(ports: Vec<Sender<Data>>, to: Port, m: Data) -> Result<()> {
        ports[to.to_usize()].send(m).or_else(|_| Err(Error::SendError))
    }
    pub fn shutdown(&mut self) -> Result<()> {
        {
            *self.exit.lock().unwrap() = true;
        }
        {
            let r = self.lock.read().unwrap();
            for t in r.threads.iter() {
                match Arc::try_unwrap((*t).clone()) {
                    Ok(Some(j)) => 
                        match j.join() {
                            Ok(Ok(())) => (),
                            Err(_) => return Err(Error::JoinError),
                            Ok(e) => return e,
                        },
                    _ => (),
                }
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use otp;
    use std::sync::{Arc, Mutex};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_init() {
        let mut o = otp::OTP::new();
        assert_eq!(Ok(()), o.shutdown());
    }
    
    #[test]
    fn test_source() {
        let mut o = otp::OTP::new();
        let val = Arc::new(Mutex::new(false));
        let c_val = val.clone();
        o.source(otp::Port::Reader, move |_ports| {
            *c_val.lock().unwrap() = true;
            Ok(())
        });
        sleep(Duration::new(1,0));
        assert_eq!(*val.lock().unwrap(), true);
        assert_eq!(Ok(()), o.shutdown());
    }
    #[test]
    fn test_listen() {
        let mut o = otp::OTP::new();
        let val = Arc::new(Mutex::new(false));
        o.source(otp::Port::Reader, move |ports| {
            otp::OTP::send(ports, otp::Port::State, otp::Data::Signal)
        });
        let c_val = val.clone();
        o.listen(otp::Port::State, move |ports, data| {
            match data {
                otp::Data::Signal => *c_val.lock().unwrap() = true,
                _ => (),
            }
            Ok(())
        });

        sleep(Duration::new(1,0));
        assert_eq!(*val.lock().unwrap(), true);
        assert_eq!(Ok(()), o.shutdown());
    }

}
