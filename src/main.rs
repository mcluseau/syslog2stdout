use std::os::unix::net;
use std::sync::Arc;

type Mutex = std::sync::Mutex<fn(u32, &[u8])>;

fn main() -> std::io::Result<()> {
    let socket_path = std::env::args().nth(1).unwrap();

    std::thread::spawn(|| {
        use signal_hook::{consts::SIGTERM, iterator::Signals};
        let mut signals = Signals::new(&[SIGTERM]).expect("failed to setup signal handler");
        let sig = signals.forever().next().unwrap();
        println!("received signal {sig}, exiting");
        std::process::exit(0);
    });

    let _ = std::fs::remove_file(&socket_path);

    let listener = net::UnixListener::bind(socket_path)?;

    let mutex = Arc::new(Mutex::new(handle_msg));

    let mut next_id = 0;
    loop {
        let (stream, addr) = listener.accept()?;

        let id = next_id;
        next_id += 1;
        println!("connection {id} from {addr:?}");

        let mutex = mutex.clone();

        std::thread::spawn(move || {
            if let Err(e) = handle_stream(id, stream, mutex) {
                println!("[{id}] stream error: {e}");
            } else {
                println!("[{id}] stream closed");
            }
        });
    }
}

fn handle_stream(id: u32, mut stream: net::UnixStream, mutex: Arc<Mutex>) -> std::io::Result<()> {
    use std::io::Read;

    let mut buf = [0; 1024];

    loop {
        let n = stream.read(&mut buf)?;

        if n == 0 {
            return Ok(());
        }

        mutex.lock().unwrap()(id, &buf[0..n]);
    }
}

fn handle_msg(id: u32, msg: &[u8]) {
    let msg = String::from_utf8_lossy(msg);
    println!("[{id}] {msg}");
}
