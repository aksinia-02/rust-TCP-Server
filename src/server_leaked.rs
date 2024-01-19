use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::env;
use signal_hook::consts::SIGINT;

const DEFAULT_PORT: u16 = 9999;

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn handle_sigint() {
    KEEP_RUNNING.store(false, Ordering::SeqCst);
}

fn error(msg: &str) {
    eprintln!("{}", msg);
    std::process::exit(1);
}

fn capitalize_string(s: &mut String) {
    s.make_ascii_uppercase();
}

fn authenticate(user: &str, password: &str) -> bool {
    let stored_pass = "PASSWORD";
    let mut capitalized_pass = String::new();
    let mut capitalized_user = String::new();

    capitalized_user.push_str(user);
    capitalized_pass.push_str(password);

    capitalize_string(&mut capitalized_user);
    capitalize_string(&mut capitalized_pass);

    if capitalized_pass == stored_pass {
        println!("Congratulations {} user!\n\n The secret code is:", user);

        // TODO: Add code that fetches secret...

        println!("\n\n");
        return true;
    } else {
        println!("WRONG! Dear {}, this was totally wrong!\n", user);
        return false;
    }
}

fn split_at_colon(buffer: &str) -> (&str, Option<String>) {
    if let Some(index) = buffer.find(':') {
        let (username, rest) = buffer.split_at(index);
        let rest = rest[1..].trim().to_string();
        (username.trim(), Some(rest))
    } else {
        ("", None)
    }
}

fn print_help(program_name: &str) {
    println!(
        "Usage: {} [-p port]\n\
        Synopsis: A simple TCP server that listens for incoming connections, receives strings of the form 'username:password', and prints a secret code if the user is worthy.\n\
        The server will capitalize incoming user/password strings, because why not?\n\
        Options:\n\
        -p <port>    Specify the port on which the server should listen. Default is {}.",
        program_name, DEFAULT_PORT
    );
}

fn main() {
	let args: Vec<String> = env::args().collect();

    let mut port: u16 = DEFAULT_PORT;

    let mut i = 1;

    let mut prize = false;

    while i < args.len(){
    	match args[i].as_str(){
    		"-h" => {
	    		print_help(&args[0]);
	    		return;
	    	}
	    	"-p" => {
	    		i += 1;
                if i < args.len() {
                    match args[i].parse::<u16>() {
                        Ok(parsed_port) => port = parsed_port,
                        Err(_) => {
                            eprintln!("Invalid port number: {}", args[i]);
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Expected argument after option.");
                    std::process::exit(1);
                }
	    	}
	    	_ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
    	}
    	i += 1;
    }

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind to address");
    listener.set_nonblocking(true).unwrap();

    println!("Listening on port {}... ", port);

    let mut sigint_handler = signal_hook::iterator::Signals::new(&[SIGINT]).unwrap();
    thread::spawn(move || {
        for _ in sigint_handler.forever() {
            println!("SIGINT received, shutting down.");
            handle_sigint();
        }
    });

    for stream in listener.incoming() {
        if !KEEP_RUNNING.load(Ordering::SeqCst) || prize{
            break;
        }

        match stream {
            Ok(mut stream) => {
                //reads at most 1024 bytes at a time
                let mut buffer = [0u8; 1024]; 
                if let Ok(size) = stream.read(&mut buffer) {
                    let request = String::from_utf8_lossy(&buffer[..size]);
                    let (username, password) = split_at_colon(&request);
                    let response = if let Some(password) = password {
                        if authenticate(username, &password) {
                            prize = true;
                            "Authenticated! Check the server output to receive your prize.\n\n"
                        } else {
                            "Access Denied!\n\n"
                        }
                    } else {
                        "Malformed request! (send me a string like 'username:password')\n\n"
                    };

                    if stream.write_all(response.as_bytes()).is_err() {
                        error("Error writing to socket");
                    }
                } else {
                    error("Error reading from socket");
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => eprintln!("Error accepting connection"),
        }

        thread::sleep(Duration::from_secs(1)); // Simulate the 1-second timeout
    }
}
