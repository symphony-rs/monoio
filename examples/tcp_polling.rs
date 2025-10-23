//! An example to show how to use TcpStream.

#[cfg(not(target_os = "linux"))]
fn main() {}

#[cfg(target_os = "linux")]
fn main() {
    println!("Will run with IoUringDriver(you must be on linux and enable iouring feature)");
    run::<monoio::time::TimeDriver<monoio::IoUringDriver>>();
}

#[cfg(target_os = "linux")]
fn run<D>()
where
    D: monoio::Buildable + monoio::Driver,
{
    use futures::channel::oneshot;
    use monoio::{
        io::{AsyncReadRent, AsyncWriteRentExt},
        net::{TcpListener, TcpStream},
    };

    const ADDRESS: &str = "127.0.0.1:50000";

    let (mut tx, rx) = oneshot::channel::<()>();
    let client_thread = std::thread::spawn(|| {
        monoio::start::<D, _>(async move {
            println!("[Client] Waiting for server ready");
            tx.cancellation().await;

            loop {
                use std::time::{Duration, Instant};

                let ins = Instant::now();
                println!("[Client] Server is ready, will connect and send data");
                let mut conn = TcpStream::connect(ADDRESS)
                    .await
                    .expect("[Client] Unable to connect to server");
                let buf: Vec<u8> = vec![97; 10];
                let (r, _) = conn.write_all(buf).await;
                println!(
                    "[Client] Written {} bytes data and leave, cost {:?}",
                    r.unwrap(),
                    ins.elapsed()
                );
                monoio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    });

    let server_thread = std::thread::spawn(|| {
        monoio::start::<D, _>(async move {
            let listener = TcpListener::bind(ADDRESS)
                .unwrap_or_else(|_| panic!("[Server] Unable to bind to {ADDRESS}"));
            println!("[Server] Bind ready");

            let addr: &str = "127.0.0.1:50001";
            let listene2 = TcpListener::bind(addr)
                .unwrap_or_else(|_| panic!("[Server] Unable to bind to {addr}"));
            println!("[Server] Bind ready");
            drop(rx);

            loop {
                monoio::select! {
                    conn = listener.accept() => {
                        if let Ok((mut conn, _addr)) = conn {
                            println!("[Server] Accepted a new connection, will read form it");

                            let buf = vec![0; 64];
                            let (r, buf) = conn.read(buf).await;

                            let read_len = r.unwrap();
                            println!(
                                "[Server] Read {} bytes data: {:?}",
                                read_len,
                                &buf[..read_len]
                            );
                        }
                    }
                    conn = listene2.accept() => {
                        if let Ok((mut conn, _addr)) = conn {
                            println!("[Server] Accepted a new connection, will read form it");

                            let buf = vec![0; 64];
                            let (r, buf) = conn.read(buf).await;

                            let read_len = r.unwrap();
                            println!(
                                "[Server] Read {} bytes data: {:?}",
                                read_len,
                                &buf[..read_len]
                            );
                        }
                    }
                }
            }
        });
    });

    server_thread.join().unwrap();
    client_thread.join().unwrap();
}
