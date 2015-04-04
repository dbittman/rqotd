#![feature(convert)]
#![feature(collections)]
extern crate getopts;
extern crate toml;
use getopts::Options;
use std::env;
use std::fs::File;
use std::sync::Arc;
use std::path::Path;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::process::Command;
use std::convert::AsRef;
use toml::{Parser};

/*
 * Basic configuration stuff. Load a TOML file and parse it
 */

static DEFAULT_CONFIG: &'static str = "/etc/rqotd.conf";
static DEFAULT_PORT: i64 = 17;
struct Conf {
    execute: String,
    args: Vec<String>,
    message: String,
    port: i64
}

/* Read a string from the config file. Return an empty string
 * if it doesn't exist */
fn toml_read_string(toml: &toml::Table, key: &str) -> String {
    let val = match toml.get(key) {
        None => "",
        Some(s) => match s.as_str() {
            None => "",
            Some(s) => s
        }
    };
    val.to_string()
}

fn load_configuration(path: String) -> Conf {

    let mut contents: String = "".to_string();
    let file = File::open(&Path::new(path.as_str()));
    if let Err(ref e) = file {
        writeln!(&mut std::io::stderr(), "Failed to open configuration file: {}", e).unwrap();
        std::process::exit(1);
    }
    file.unwrap().read_to_string(&mut contents).unwrap();

    let mut p = Parser::new(contents.as_str());
    let table = p.parse();

    for e in p.errors.iter() {
        writeln!(&mut std::io::stderr(), "TOML file error: {}", e).unwrap();
    }
    if p.errors.len() != 0 {
        writeln!(&mut std::io::stderr(), "Failed to parse configuration file").unwrap();
        std::process::exit(1);
    }

    let toml = table.unwrap();

    let port = match toml.get("port") {
        None => DEFAULT_PORT,
        Some(s) => s.as_integer().unwrap()
    };
    let command = toml_read_string(&toml, "execute");
    let message = toml_read_string(&toml, "message");

    let mut args = vec!();
    if let Some(a) = toml.get("args") {
        if let Some(s) = a.as_slice() {
            args.push_all(s);
        }
    };
    let args = args.iter().map(|s| s.as_str().unwrap().to_string()).collect();

    Conf { execute: command, args: args, message: message, port: port }
}

fn handle_client(mut stream: TcpStream, config: &Conf) {
    if config.execute != "" {
        let output = Command::new(config.execute.as_str())
            .args(config.args.as_slice())
            .output()
            .unwrap_or_else( |e| {
                panic!("Failed to execute qotd process: {}", e);
        });
        stream.write(output.stdout.as_ref()).unwrap();
    } else {
        stream.write(config.message.clone().into_bytes().as_slice()).unwrap();
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help message");
    opts.optopt("c", "config", "Specify configuraion file", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let config_file = match matches.opt_str("c") {
        None => DEFAULT_CONFIG.to_string(),
        Some(c) => c
    };

    let config = Arc::new(load_configuration(config_file));

    let listener = match TcpListener::bind(format!("localhost:{}", config.port).as_str()) {
        Err(e) => panic!("Failed to bind to port: {}", e),
        Ok(listener) => listener
    };

    println!("Started rQotd, listening for connections on port {}", config.port);

    for stream in listener.incoming() {
        let config = config.clone();
        match stream {
            Err(e) => println!("Connection failed: {}", e),
            Ok(stream) => {std::thread::spawn(move || {
                handle_client(stream, &*config);
            });}
        }
    }
}

