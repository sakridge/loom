use state;
use data;
use serde_json;

use std::sync::{Arc, Mutex};
use std::io::Read;
use result::Result;
use reader::Reader;
use std::fs::File;
use std::mem::transmute;
use getopts::Options;
use std::string::String;
use otp::{Port, OTP};
use sender::Sender;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn loomd(testnet: Option<String>, port: u16) -> Result<OTP> {
    let state = match testnet {
        Some(f) => state_from_file(&f).and_then(|x| Ok(Arc::new(Mutex::new(x))))?,
        None => Arc::new(Mutex::new(state::State::new(1024))),
    };
    let reader = Reader::new(port).and_then(|x| Ok(Arc::new(x)))?;
    let mut o = OTP::new();
    let a_reader = reader.clone();
    o.source(Port::Reader, move |p| a_reader.run(p))?;
    let b_reader = reader.clone();
    o.listen(Port::Recycle, move |_p, d| {
        b_reader.recycle(d);
        Ok(())
    })?;
    let sender = Sender::new().and_then(|x| Ok(Arc::new(x)))?;
    o.listen(Port::Sender, move |_p, d| sender.run(d))?;
    let a_state = state.clone();
    o.listen(Port::State, move |p, d| a_state.lock().unwrap().run(p, d))?;
    return Ok(o);
}

#[derive(Deserialize)]
struct TestAccount {
    pub pubkey: [u64; 4],
    pub balance: u32,
}

fn state_from_file(f: &str) -> Result<state::State> {
    let mut file = File::open(f)?;
    let mut e = Vec::new();
    let _sz = file.read_to_end(&mut e)?;
    let v: Vec<TestAccount> = serde_json::from_slice(&e)?;
    let acc: Vec<data::Account> = v.iter()
        .map(|a| {
            let pk = unsafe { transmute::<[u64; 4], [u8; 32]>(a.pubkey) };
            data::Account {
                from: pk,
                balance: a.balance as u64,
            }
        })
        .collect();
    state::State::from_list(&acc)
}

pub fn run(args: Vec<String>) -> Option<OTP> {
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("l", "", "Run as a Loom with a listen port", "PORT");
    opts.optopt("t", "", "testnet accounts", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print_usage(&program, opts);
            panic!(f.to_string());
        }
    };
    if matches.opt_str("l").is_some() {
        let ports = matches.opt_str("l").expect("missing loom port");
        let port = ports.parse().expect("expecting u16 number for port");
        let daemon = loomd(matches.opt_str("t"), port).expect("loomd");
        return Some(daemon);
    } else {
        print_usage(&program, opts);
    }
    return None;
}

#[cfg(test)]
mod tests {
    use daemon;
    use net;
    use data;
    use wallet;
    use result::Result;
    use std::net::UdpSocket;
    use std::mem::transmute;

    fn check_balance(s: &UdpSocket, w: &wallet::Wallet, to: [u8; 32]) -> Result<u64> {
        let mut num = 0;
        let addr = "127.0.0.1:24567".parse().expect("parse");
        while num < 1 {
            let msg = w.check_balance(0, to, 1);
            net::send_to(&s, &[msg], &mut num, addr)?;
        }
        assert_eq!(num, 1);
        let mut rmsgs = data::Messages::new();
        rmsgs
            .with_mut(|m, d| net::read_from(&s, m, d))
            .expect("read rmsgs");
        assert_eq!(rmsgs.data[0].0, 1);
        Ok(rmsgs.msgs[0].pld.get_bal().amount)
    }
    fn from_pk(d: [u64; 4]) -> [u8; 32] {
        unsafe { transmute::<[u64; 4], [u8; 32]>(d) }
    }
    #[test]
    fn help_test() {
        assert!(daemon::run(vec!["loomd".into(), "-h".into()]).is_none());
        assert!(daemon::run(vec!["loomd".into()]).is_none());
    }
    #[test]
    fn transaction_test() {
        let args = vec![
            "loomd".into(),
            "-l".into(),
            "24567".into(),
            "-t".into(),
            "testdata/test_accounts.json".into(),
        ];
        let mut t = daemon::run(args).expect("daemon load");
        let ew = wallet::EncryptedWallet::from_file("testdata/loom.wallet").expect("test wallet");
        let w = ew.decrypt("foobar".as_bytes()).expect("decrypt");
        let from = from_pk(w.pubkeys[0]);
        let kp = wallet::Wallet::new_keypair();
        let to = from_pk(kp.1);
        let s = net::socket().expect("socket");
        let addr = "127.0.0.1:24567".parse().expect("parse");
        let mut num = 0;
        while num < 1 {
            let msg = w.tx(0, to, 1000, 1);
            net::send_to(&s, &[msg], &mut num, addr).expect("write message");
        }
        let bto = check_balance(&s, &w, to).expect("check bal to");
        assert_eq!(bto, 1000);
        let bfrom = check_balance(&s, &w, from).expect("check bal from");
        assert_eq!(bfrom, 1000000000 - 1003);
        t.shutdown().expect("success");
    }
    #[test]
    fn realnet_test() {
        let args = vec!["loomd".into(), "-l".into(), "24568".into()];
        let mut t = daemon::run(args).expect("daemon load");
        t.shutdown().expect("success");
    }
}
