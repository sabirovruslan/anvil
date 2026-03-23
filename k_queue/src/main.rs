use std::{
    collections::HashSet,
    io::{self, Read, Result, Write},
    net::TcpStream,
};

use crate::{ffi::Event, poll::Poll};

mod ffi;
mod poll;

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    )
}

fn handle_events(
    events: &[Event],
    streams: &mut [TcpStream],
    handled: &mut HashSet<usize>,
) -> Result<usize> {
    let mut handled_events = 0;
    for event in events {
        let index = event.token();
        let mut data = vec![0u8; 4096];

        loop {
            match streams[index].read(&mut data) {
                Ok(0) => {
                    if !handled.insert(index) {
                        break;
                    }
                    handled_events += 1;
                    break;
                }
                Ok(n) => {
                    let txt = String::from_utf8_lossy(&data[..n]);

                    println!("RECEIVED: {:?}", event);
                    println!("{txt}\n------\n");
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => break,
                Err(e) => return Err(e),
            }
        }
    }

    Ok(handled_events)
}

fn main() -> Result<()> {
    let mut poll = Poll::new()?;
    let n_events = 10;

    let mut streams: Vec<TcpStream> = vec![];
    let addr = String::from("localhost:8080");

    for i in 0..5 {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let requiest = get_req(&url_path);
        let mut stream = TcpStream::connect(&addr)?;
        stream.set_nonblocking(true)?;

        stream.write_all(requiest.as_bytes())?;

        poll.registry()
            .register(&stream, i, (ffi::EV_ADD | ffi::EV_CLEAR) as i32)?;

        streams.push(stream);
    }

    for i in 5..10 {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let requiest = get_req(&url_path);
        let mut stream = TcpStream::connect(&addr)?;
        stream.set_nonblocking(true)?;

        stream.write_all(requiest.as_bytes())?;

        poll.registry()
            .register(&stream, i, (ffi::EV_ADD | ffi::EV_CLEAR) as i32)?;

        streams.push(stream);
    }

    let mut handled_ids: HashSet<usize> = HashSet::new();
    let mut handled_events = 0;

    while handled_events < n_events {
        let mut events: Vec<Event> = Vec::with_capacity(20);
        poll.poll(&mut events, None)?;

        if events.is_empty() {
            print!("timeout or spurios ...");
            continue;
        }

        handled_events += handle_events(&events, &mut streams, &mut handled_ids)?;
    }

    print!("finish");

    Ok(())
}
