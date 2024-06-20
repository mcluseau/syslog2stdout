use tokio::io::{AsyncReadExt, BufReader};
use tokio::net;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let socket_path = std::env::args().nth(1).expect("missing socket path arg");

    use tokio::signal::unix::{signal, SignalKind};
    let mut signals = signal(SignalKind::terminate())?;
    tokio::spawn(async move {
        loop {
            signals.recv().await;
            println!("received signal, exiting");
            std::process::exit(0);
        }
    });

    let _ = std::fs::remove_file(&socket_path);

    let listener = net::UnixListener::bind(socket_path)?;

    let mut next_id = 0;
    loop {
        let (stream, _) = listener.accept().await?;

        let id = next_id;
        next_id += 1;
        // println!("connection {id} from {addr:?}");

        tokio::spawn(async move {
            if let Err(e) = handle_stream(id, stream).await {
                println!("{id}| stream error: {e}");
            }
        });
    }
}

async fn handle_stream(id: u32, stream: net::UnixStream) -> std::io::Result<()> {
    let mut buf = vec![];

    let mut reader = BufReader::with_capacity(1024, stream);

    loop {
        loop {
            let b = match reader.read_u8().await {
                Ok(b) => b,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        return Ok(());
                    }
                    return Err(e);
                }
            };
            if b == b'\n' || b == b'\0' {
                break;
            }
            buf.push(b);
        }

        handle_msg(id, &buf);
        buf.clear();
    }
}

fn handle_msg(id: u32, mut msg: &[u8]) {
    let mut prival = 0u8;
    if msg.len() >= 3 && msg[0] == b'<' {
        if let Some(pos) = msg.iter().position(|b| *b == b'>') {
            if let Some(pri) = String::from_utf8(msg[1..pos].to_vec())
                .ok()
                .and_then(|s| s.parse::<u8>().ok())
            {
                prival = pri;
                msg = &msg[pos + 1..];
            };
        };
    }

    let (prio, facility) = (prival & 0b111, prival >> 3);

    let prio = match prio {
        0 => "emergency", // Emergency: system is unusable
        1 => "alert",     // Alert: action must be taken immediately
        2 => "critical",  // Critical: critical conditions
        3 => "error",     // Error: error conditions
        4 => "warning",   // Warning: warning conditions
        5 => "notice",    // Notice: normal but significant condition
        6 => "info",      // Informational: informational messages
        7 => "debug",     // Debug: debug-level messages
        _ => unreachable!(),
    };

    let facility = match facility {
        0 => "kern",      // kernel messages
        1 => "user",      // user-level messages
        2 => "mail",      // mail system
        3 => "daemon",    // system daemons
        4 => "auth",      // security/authorization messages
        5 => "syslog",    // messages generated internally by syslogd
        6 => "lpr",       // line printer subsystem
        7 => "news",      // network news subsystem
        8 => "uucp",      // UUCP subsystem
        9 => "cron",      // clock daemon
        10 => "authpriv", // security/authorization messages
        11 => "ftp",      // FTP daemon
        12 => "ntp",      // NTP subsystem
        13 => "audit",    // log audit
        14 => "alert",    // log alert
        15 => "cron2",    // clock daemon (note 2)
        16 => "local0",   // local use 0  (local0)
        17 => "local1",   // local use 1  (local1)
        18 => "local2",   // local use 2  (local2)
        19 => "local3",   // local use 3  (local3)
        20 => "local4",   // local use 4  (local4)
        21 => "local5",   // local use 5  (local5)
        22 => "local6",   // local use 6  (local6)
        23 => "local7",   // local use 7  (local7)
        _ => "?",
    };

    let msg = String::from_utf8_lossy(msg);
    println!("{id} {facility} {prio}| {msg}");
}
