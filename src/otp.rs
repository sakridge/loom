use std::sync::{RwLock, Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread::{JoinHandle, spawn};
use std::time::Duration;
use data;
use result::Result;

enum Port {
    Reader,
    State,
    Max,
}

impl Port {
    fn to_usize(self) -> usize {
        match self {
            Reader => 0,
            State => 1,
            Max => 2,
        }
    }
}

#[derive(Clone)]
enum Data {
    Signal,
    SharedMessages(data::SharedMessages),
}

#[derive(Clone)]
struct Locked {
    ports: Vec<Sender<Data>>,
    readers: Vec<Arc<Mutex<Receiver<Data>>>>,
    threads: Vec<Arc<Option<JoinHandle<Result<()>>>>>,
}

#[derive(Clone)]
struct OTP {
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
            match func(c_ports) {
                Ok(()) => (),
                e => return e
            }
            if *c_exit.lock().unwrap() == true {
                return Ok(());
            }
        });
        w.threads[port.to_usize()] = Arc::new(Some(j));
    }
    pub fn listener<F>(&mut self, port: Port, func: F)
        where F: Send + 'static + Fn(Vec<Sender<Data>>, Data) -> Result<()>
    {
        let mut w = self.lock.write().unwrap();
        let recv_lock = w.readers[port.to_usize()].clone();
        let c_ports = w.ports.clone();
        let c_exit = self.exit.clone();
        let j: JoinHandle<Result<()>> = spawn(move|| loop {
            let recv = recv_lock.lock().unwrap();
            let timer = Duration::new(0, 500000);
            match recv.recv_timeout(timer) {
                Ok(val) => func(c_ports, val)?,
                _ => (),
            }
            if *c_exit.lock().unwrap() == true {
                return Ok(());
            }
        });
        w.threads[port.to_usize()] = Arc::new(Some(j));
    }
    pub fn send(ports: Vec<Sender<Data>>, to: Port, m: Data) {
        ports[to.into()].unwrap().send(m);
    }
    pub fn shutdown(&mut self) {
        {
            *self.exit.lock().unwrap() = true;
        }
        {
            let r = self.lock.read().unwrap();
            for t in r.threads {
                t.join();
            }
        }
    }
}
