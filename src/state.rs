//! state machine for transactions

use data;
use result::Result;
use hasht::Key;
use otp::{OTP, Port, Data, Ports};

#[repr(C)]
pub struct State {
    accounts: Vec<data::Account>,
    used: usize,
}

impl State {
    pub fn new(size: usize) -> State {
        State {
            accounts: vec![data::Account::default(); size],
            used: 0,
        }
    }
    pub fn from_list(v: &[data::Account]) -> Result<State> {
        let mut s = Self::new(v.len() * 2);
        for a in v {
            let fp = data::AccountT::find(&s.accounts, &a.from)?;
            assert!(s.accounts[fp].from.unused());
            s.accounts[fp].balance = a.balance;
            s.accounts[fp].from = a.from;
        }
        s.used = v.len();
        return Ok(s);
    }
    fn double(&mut self) -> Result<()> {
        let size = self.accounts.len() * 2;
        let mut v = vec![data::Account::default(); size];
        data::AccountT::migrate(&self.accounts, &mut v)?;
        self.accounts = v;
        Ok(())
    }
    fn find_accounts(
        state: &[data::Account],
        fk: &[u8; 32],
        tk: &[u8; 32],
    ) -> Result<(usize, usize)> {
        let sf = data::AccountT::find(&state, fk)?;
        let st = data::AccountT::find(&state, tk)?;
        Ok((sf, st))
    }
    fn load_accounts<'a>(
        state: &'a mut [data::Account],
        (sf, st): (usize, usize),
    ) -> (&'a mut data::Account, &'a mut data::Account) {
        let ptr = state.as_mut_ptr();
        let from = unsafe { ptr.offset(sf as isize).as_mut().unwrap() };
        let to = unsafe { ptr.offset(st as isize).as_mut().unwrap() };
        (from, to)
    }
    pub fn run(&mut self, p: &Ports, d: Data) -> Result<()> {
        match d {
            Data::SharedMessages(m) => {
                self.execute(&mut m.write().unwrap().msgs)?;
                for v in m.read().unwrap().msgs.iter() {
                    assert_eq!(v.pld.kind, data::Kind::Transaction);
                    assert_eq!(v.pld.state, data::State::Deposited);
                }
                assert!(false);
                OTP::send(p, Port::Recycle, Data::SharedMessages(m))?;
            }
            _ => (),
        }
        return Ok(());
    }
    fn exec(state: &mut [data::Account], m: &mut data::Message, num_new: &mut usize) -> Result<()> {
        assert_eq!(m.pld.kind, data::Kind::Transaction, "{:?}", m.pld.from); 
        if m.pld.kind != data::Kind::Transaction {
            return Ok(());
        }
        let pos = Self::find_accounts(state, &m.pld.from, &m.pld.get_tx().to)?;
        let (mut from, mut to) = Self::load_accounts(state, pos);
        if from.from != m.pld.from {
            return Ok(());
        }
        if !to.from.unused() && to.from != m.pld.get_tx().to {
            return Ok(());
        }
        Self::charge(&mut from, m);
        if m.pld.state != data::State::Withdrawn {
            return Ok(());
        }
        Self::new_account(&to, num_new);
        Self::deposit(&mut to, m);
        assert_eq!(m.pld.state, data::State::Deposited, "{:?}", m.pld.from); 
        Ok(())
    }
    fn execute(&mut self, msgs: &mut [data::Message]) -> Result<()> {
        let mut num_new = 0;
        for mut m in msgs.iter_mut() {
            let len = self.accounts.len();
            if self.used * 4 > len * 3 {
                println!("{:?} {:?}", self.used, len);
                self.double()?;
            }
            Self::exec(&mut self.accounts, &mut m, &mut num_new)?;
            assert_eq!(m.pld.state, data::State::Deposited); 
            self.used += num_new;
        }
        assert!(false);
        Ok(())
    }
    fn charge(acc: &mut data::Account, m: &mut data::Message) -> () {
        let combined = m.pld.get_tx().amount + m.pld.fee;
        if acc.balance >= combined {
            m.pld.state = data::State::Withdrawn;
            acc.balance = acc.balance - combined;
        }
    }
    fn new_account(to: &data::Account, num: &mut usize) -> () {
        if to.from.unused() {
            *num = *num + 1;
        }
    }
    fn deposit(to: &mut data::Account, m: &mut data::Message) -> () {
        to.balance = to.balance + m.pld.get_tx().amount;
        if to.from.unused() {
            to.from = m.pld.get_tx().to;
            assert!(!to.from.unused());
        }
        m.pld.state = data::State::Deposited;
    }
}

#[cfg(test)]
mod tests {
    use state::State;
    use reader::Reader;
    use data;
    use std::sync::{Arc, Mutex};
    use net;
    use std::net::UdpSocket;
    use hasht::Key;
    use otp::OTP;
    use otp::Port;
    use otp::Data::{Signal, SharedMessages};

    #[test]
    fn state_test() {
        let mut s: State = State::new(64);
        let mut msgs = [];
        s.execute(&mut msgs).expect("e");
    }

    fn init_msgs(msgs: &mut [data::Message]) {
        for (i, m) in msgs.iter_mut().enumerate() {
            m.pld.kind = data::Kind::Transaction;
            m.pld.get_tx_mut().to = [255u8; 32];
            m.pld.get_tx_mut().to[0] = i as u8;
            m.pld.from = [255u8; 32];
            m.pld.fee = 1;
            m.pld.get_tx_mut().amount = 1;
            assert!(!m.pld.get_tx().to.unused());
        }
    }
    #[test]
    fn state_from_list_test() {
        let f = [255u8; 32];
        let list = [data::Account {from: f, balance: 2u64}];
        let s = State::from_list(&list).expect("from list");
        assert_eq!(s.used, list.len());
        let fp = data::AccountT::find(&s.accounts, &f).expect("f");
        assert_eq!(s.accounts[fp].from, f);
        assert_eq!(s.accounts[fp].balance, 2u64);
    }
    #[test]
    fn state_system_test() {
        const NUM: usize = 128usize;
        let f = [255u8; 32];
        let reader = Arc::new(Reader::new(12002).expect("reader"));
        let mut o = OTP::new();
        let a_reader = reader.clone();
        assert_eq!(Ok(()),
            o.source(Port::Reader, move |p| a_reader.run(p)));
        let b_reader = reader.clone();
        assert_eq!(Ok(()),
            o.listen(Port::Recycle, move |p, d| {
                let d_ = d.clone();
                match d {
                    SharedMessages(m) => {
                        for v in m.read().unwrap().msgs.iter() {
                            assert_eq!(v.pld.state, data::State::Deposited);
                        }
                        OTP::send(p, Port::Main, Signal)?;
                    }
                    _ => (),
                }
                b_reader.recycle(d_);
                Ok(())
            }));
        let list = [data::Account {from: f, balance: NUM as u64 * 2u64}];
        let state = Arc::new(Mutex::new(State::from_list(&list).expect("from list")));

        let a_state = state.clone();
        assert_eq!(Ok(()),
            o.listen(Port::State, move |p, d| a_state.lock().unwrap().run(p, d)));
        let cli: UdpSocket = net::socket().expect("socket");
        cli.connect("127.0.0.1:12002").expect("client");
        let mut msgs = [data::Message::default(); NUM];
        init_msgs(&mut msgs);
        let mut num = 0;
        while num < 64 {
            net::write(&cli, &msgs, &mut num).expect("send msgs");
        }
        assert_eq!(Ok(()), o.join());
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;
    use self::test::Bencher;
    use data;
    use state::State;
    use state::test::init_msgs;
    use hasht::Key;


    #[bench]
    fn state_bench(b: &mut Bencher) {
        const NUM: usize = 128usize;
        let mut s: State = State::new(NUM * 2);
        let mut msgs = [data::Message::default(); NUM];
        init_msgs(&mut msgs);
        let fp = data::AccountT::find(&s.accounts, &[255u8; 32]).expect("f");
        s.accounts[fp].from = [255u8; 32];
        b.iter(|| {
            s.accounts[fp].balance = NUM as u64 * 2u64;
            s.execute(&mut msgs).expect("execute");
            assert_eq!(s.accounts[fp].balance, 0);
        })
    }
}
