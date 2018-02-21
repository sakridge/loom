use std::thread::spawn;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
std::sync::RwLock;
enum Port {
    Reader,
    State,
    Max,
}

impl Into<usize> for Port {
    fn into(p: Port) -> usize {
        match sz {
            Reader => 0,
            State => 1,
            Max => 2,
        }
    }
}

enum Data {
    Signal,
    SharedMessages(data::SharedMessages),
}

enum Reply {
    Noop,
    Send(Port, Data),
}

struct Locked {
    ports: Vec<Option<Sender<Data>>>,
    threads: Vec<Option<JoinHandle<Result<()>>>>,
}
struct OTP {
    lock: RwLock<Locked>,
    exit: exit: Arc<Mutex<bool>>,
};

impl OTP {
    pub fn new() -> OTP {
        let locked = Locked {
            ports : vec![Port::Max.into(); None],
            threads : vec![Port::Max.into(); None],
        };
        let exit = Arc::new(Mutex::new(false));
        OTP {lock: RwLock::new(locked), exit: exit}
    }
    pub fn source<F>(&mut self, port: Port, func: F)
        where F: () -> Reply {
        let mut w = self.lock.write().unwrap();
        let j = spawn(move|| loop {
            func(&self, val)?;
            let e = self.exit.lock().expect("lock");
            if *e == true {
                trace!("exiting");
                return Ok(());
            }
        });
        w.threads[port.into()] = Some(j);
    }
    pub fn listener<F>(&mut self, port: Port, func: F)
        where F: (Data) -> Reply {
        let mut w = self.lock.write().unwrap();
        let recv = Self::register(w, port);
        let j = spawn(move|| loop {
            let timer = Duration::new(1, 0);
            match recv.recv_timeout(timer) {
                Ok(val) => {
                    match func(&self, val) {
                        Send(p,m) => self.send(p, m);
                    }
                }
                _ => (),
            }
            let e = self.exit.lock().expect("lock");
            if *e == true {
                trace!("exiting");
                return Ok(());
            }
        });
        w.threads[port.into()] = Some(j);
    }
    pub fn send(&self, to: Port, m: Data) {
        let r = self.lock.read().unwrap();
        r.ports[to.into()].unwrap().send(m);
    }
    pub fn shutdown(&mut self) {
        let mut w = self.lock.write().unwrap();
        *self.exit.lock().expect("lock") = true;
        for t in self.threads {
            t.join();
        }
        w.ports = vec![Port::Max.into(); None];
        w.threads = vec![Port::Max.into(); None];
    }

    fn register(w: &mut Locked, port: Port) -> Receiver<Data> {
        if w.ports[port.into()].is_none() {
            let s,r = channel()
            w.ports[port.into()] = Some(s);
            return r;
        } else {
            return w.ports[port.into()].unwrap();
        }
    }

}
