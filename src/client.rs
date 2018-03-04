use rpassword;
use getopts::Options;
use std::string::String;
use data_encoding::BASE32HEX_NOPAD;
use wallet::{EncryptedWallet, Wallet, to32b};
use net;
use result::Result;

struct Cfg {
    host: String,
    wallet: String,
}

fn vec_to_array(v: Vec<u8>) -> [u8; 32] {
    let mut a = [0; 32];
    a.copy_from_slice(&v);
    return a;
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn load_wallet(cfg: &Cfg, pass: String) -> Wallet {
    println!("loading from {:?}", cfg.wallet);
    match EncryptedWallet::from_file(&cfg.wallet) {
        Ok(ew) => ew.decrypt(pass.as_bytes()).expect("decrypt wallet"),
        _ => Wallet::new(),
    }
}

fn new_key_pair(cfg: &Cfg) {
    let prompt = "loom wallet password: ";
    let pass = rpassword::prompt_password_stdout(prompt).expect("password");
    println!("pass is {:?} long", pass.len());
    let mut w = load_wallet(cfg, pass.clone());
    println!("wallet has {:?} keys", w.pubkeys.len());
    let kp = Wallet::new_keypair();
    w.add_keypair(kp);
    w.encrypt(pass.as_bytes())
        .expect("encrypt")
        .to_file(&cfg.wallet)
        .expect("write");
}

fn transfer(cfg: &Cfg, from: String, to: String, amnt: u64) -> Result<()> {
    let prompt = "loom wallet password: ";
    let pass = rpassword::prompt_password_stdout(prompt).expect("password");
    let w = load_wallet(cfg, pass);
    let fpk = BASE32HEX_NOPAD.decode(from.as_bytes()).expect("from key");
    let tpk = BASE32HEX_NOPAD.decode(to.as_bytes()).expect("to key");
    let kix = w.find(vec_to_array(fpk))?;
    let msg = w.tx(kix, vec_to_array(tpk), amnt, 1);
    let s = net::socket()?;
    s.connect(cfg.host.clone())?;
    let mut num = 0;
    while num < 1 {
        net::write(&s, &[msg], &mut num)?;
    }
    Ok(())
}

fn balance(_addr: String) {}

fn list(cfg: &Cfg) {
    let prompt = "loom wallet password: ";
    let pass = rpassword::prompt_password_stdout(prompt).expect("password");
    println!("pass is {:?} long", pass.len());
    let w = load_wallet(cfg, pass);
    println!("wallet has {:?} keys", w.pubkeys.len());
    for k in w.pubkeys {
        let pretty = BASE32HEX_NOPAD.encode(&to32b(k));
        println!("key {:?}", pretty);
    }
}

pub fn run(args: Vec<String>) {
    let program = args[0].clone();
    let mut cfg = Cfg {
        host: "loom.loomprotocol.com:12345".to_string(),
        wallet: "loom.wallet".to_string(),
    };
    let mut opts = Options::new();
    opts.optflag("c", "", "create a new address");
    opts.optflag("x", "", "transfer");
    opts.optflag("b", "", "check the balance of destination address");
    opts.optflag("l", "list", "list your addresses and balances");
    opts.optflag("h", "help", "print this help menu");
    opts.optopt(
        "H",
        "",
        "loom node address to use instead of loom.looprotocol.com:12345",
        "HOST:PORT",
    );
    opts.optopt("W", "", "loom wallet instead of loom.wallet", "PATH");
    opts.optopt("t", "", "destination address", "ADDRESS");
    opts.optopt("f", "", "source address", "ADDRESS");
    opts.optopt("a", "", "amount", "AMOUNT");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if matches.opt_present("H") {
        cfg.host = matches.opt_str("H").expect("loom host address");
    }
    if matches.opt_present("W") {
        cfg.wallet = matches.opt_str("W").expect("loom wallet path");
    }
    if matches.opt_present("c") {
        new_key_pair(&cfg);
        return;
    } else if matches.opt_present("x") {
        let to = matches.opt_str("t").expect("missing destination address");
        let from = matches.opt_str("f").expect("missing source address");
        let astr = matches.opt_str("a").expect("missing ammount");
        let a = astr.parse().expect("ammount is not a number");
        transfer(&cfg, to, from, a).expect("transfer");
        return;
    } else if matches.opt_present("b") {
        let to = matches.opt_str("t").expect("missing destination address");
        balance(to);
        return;
    } else if matches.opt_present("l") {
        list(&cfg);
    }
}

#[cfg(test)]
mod tests {
    use client;

    #[test]
    fn help_test() {
        client::run(vec!["loom".into(), "-h".into()]);
        client::run(vec!["loom".into()]);
    }
}
