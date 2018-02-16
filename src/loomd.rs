use net;
use state;
use gossip;
use data;
use serde_json;
use reader;

use std::io::Read;
use result::Result;
use std::fs::File;
use std::mem::transmute;
use getopts::Options;
use std::env;
use std::string::String;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn loomd(s: &mut state::State, port: u16) {
    loop {
        let mut g = gossip::Gossip::new(1024);
        let mut m = vec![data::Message::default(); 1024];
        let srv = net::bindall(port).expect("bind server port");
        let mut num = 0;
        let start = num;
        net::read(&srv, &mut m[start..], &mut num).expect("read");
        let end = num;
        s.execute(&mut m[start..end]).expect("state");
        g.execute(&mut m[start..end]).expect("gossip");
        for s in &g.subs {
            net::sendtov4(&srv, &m[start..end], &mut num, s.addr, s.port).expect("send");
        }
    }
}

fn spoold(s: &mut state::State, loom: &str) {
    loop {
        let mut g = gossip::Gossip::new(1024);
        let mut m = vec![data::Message::default(); 1024];
        let mut num = 0;
        let start = num;
        let srv = net::socket().expect("connect to loom server");
        srv.connect(loom).expect("socket connect");
        net::read(&srv, &mut m[start..], &mut num).expect("read");
        let end = num;
        s.execute(&mut m[start..end]).expect("state");
        g.execute(&mut m[start..end]).expect("gossip");
    }
}

#[derive(Deserialize, Debug)]
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

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt(
        "s",
        "",
        "Run as a Spool node with the Loom address",
        "ADDRESS",
    );
    opts.optopt("l", "", "Run as a Loom with a listen port", "PORT");
    opts.optopt("t", "", "testnet accounts", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print_usage(&program, opts);
            panic!(f.to_string());
        }
    };
    let mut s = match matches.opt_str("t") {
        Some(f) => state_from_file(&f).expect("load test accounts"),
        None => state::State::new(1024),
    };
    if matches.opt_str("s").is_some() {
        let loom: String = matches.opt_str("s").expect("missing loom address");
        spoold(&mut s, &loom);
    } else if matches.opt_str("l").is_some() {
        let ports = matches.opt_str("l").expect("missing loom port");
        let port = ports.parse().expect("expecting u16 number for port");
        loomd(&mut s, port);
    } else {
        print_usage(&program, opts);
        return;
    }
}

#[cfg(test)]
mod tests {
    use loomd;
    use std::thread::spawn;

    fn check_balance(s: &UdpSocket, w: &Wallet, to: &[u8;32]) -> Result<u64> {
        num = 0;
        while num < 1 {
            let msg = w.check_balance(0, to).expect("new tx");
            net::write(&s, &[msg], &mut num)?;
        }
        num = 0;
        let mut msgs = [data::Message::default()];
        while num < 1 {
            net::read(s, &msgs, &num)?;
        }
        Ok(msgs[0].pld.balance.amount);
    }
    #[test]
    fn transaction_test() {
        let accounts = &"testdata/test_accounts.json";
        let mut s = state_from_file(accounts).expect("load test accounts"),
        let port = 24567;
        let t = spawn(move || loomd(&mut s, port));
        let ew = wallet::EncryptedWallet::from_file("testdata/loom.wallet");
        let w = ew.decrypt(&"foobar")?;
        let from = w.pubkeys[0];
        let kp = wallet::Wallet::new_keypair();
        let to = kp.1;
        let s = net::socket()?;
        s.connect("127.0.0.1:24567");
        let mut num = 0;
        while num < 1 {
            let msg = w.tx(0, to, 1000, 1).expect("new tx");
            net::write(&s, &[msg], &mut num)?;
        }
        let bto = check_balance(&s, &w, to);
        assert_eq!(bto, 1000);
        let bfrom = check_balance(&s, &w, from);
        assert_eq!(bfrom, 1000000000 - 1001);
    }
}


