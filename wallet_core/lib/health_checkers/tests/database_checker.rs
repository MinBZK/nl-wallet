use std::net::SocketAddr;
use std::time::Duration;

use rstest::rstest;
use sea_orm::ConnectOptions;
use sea_orm::Database;
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;
use tokio::net::lookup_host;
use tokio::select;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task::JoinHandle;

use db_test::DbSetup;
use db_test::default_connection_options;
use health_checkers::postgres::DatabaseChecker;
use http_utils::health::HealthChecker;
use http_utils::health::HealthStatus;

struct Proxy {
    listener: tokio::net::TcpListener,
    server_addr: SocketAddr,
    port: u16,
}

struct ProxyHandle(UnboundedSender<()>, JoinHandle<()>);

impl ProxyHandle {
    async fn stop(self) {
        self.0.send(()).unwrap();
        self.1.await.unwrap();
    }
}

/// Simple test TCP proxy that can be aborted
impl Proxy {
    /// Create a server that connects to `server_addr` and binds to localhost on a random port.
    pub async fn new(server_addr: impl ToSocketAddrs) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let server_addr = lookup_host(server_addr).await.unwrap().next().unwrap();
        let port = listener.local_addr().unwrap().port();
        Self {
            listener,
            server_addr,
            port,
        }
    }

    /// Spawn the Proxy server in a separate task and return a handle to stop the task.
    ///
    /// Starts a simple loop that puts every new connection in a separate task
    /// that handles the proxying.  When a signal is sent via the ProxyHandle,
    /// it aborts all started tasks.
    pub fn spawn(self) -> ProxyHandle {
        let (abort_tx, mut abort_rx) = unbounded_channel::<()>();
        let handle = tokio::spawn(async move {
            let mut abort_handles = Vec::new();
            loop {
                let abort = abort_rx.recv();
                let accept = self.listener.accept();
                select! { biased;
                    _ = abort => break,
                    result = accept => {
                        let stream = match result {
                            Ok((stream, _)) => stream,
                            Err(err) => {
                                println!("Failed to accept connection: {err}");
                                break;
                            }
                        };
                        let server_addr = self.server_addr;
                        let handle = tokio::spawn(async move {
                            match Self::handle(server_addr, stream).await {
                                Ok(_) => println!("Connection closed"),
                                Err(err) => println!("Error while handling connection: {err}")
                            }
                        });
                        abort_handles.push(handle.abort_handle());
                    },
                }
            }
            for handle in abort_handles {
                handle.abort();
            }
        });
        ProxyHandle(abort_tx, handle)
    }

    /// Handles a single client connected to the proxy.
    ///
    /// It connects to the configured server and simply passes what is received
    /// from one end to the other end via a small buffer. If one connections
    /// closes, it closes the other connection as well.
    async fn handle(server_addr: SocketAddr, mut client_stream: TcpStream) -> anyhow::Result<()> {
        let mut server_stream = TcpStream::connect(server_addr).await?;
        copy_bidirectional(&mut client_stream, &mut server_stream).await?;
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_db_check_up() {
    let db_setup = DbSetup::create().await;
    let connection = Database::connect(db_setup.connect_url()).await.unwrap();
    let checker = DatabaseChecker::new("db", &connection);
    let result = checker.status().await;
    assert_eq!(result.expect("checker should return ok"), HealthStatus::UP);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[rstest]
async fn test_db_check_down(#[values(false, true)] test_before_acquire: bool) {
    let db_setup = DbSetup::create().await;
    let connect_options = db_setup.connect_options();

    // Create proxy, this is needed because the pool tests the connection on startup
    let proxy = Proxy::new((connect_options.get_host(), connect_options.get_port())).await;
    let port = proxy.port;
    let handle = proxy.spawn();

    // Create pool
    let mut url = db_setup.connect_url();
    url.set_host(Some("127.0.0.1")).unwrap();
    url.set_port(Some(port)).unwrap();
    let mut connection_options = ConnectOptions::new(url);
    default_connection_options(&mut connection_options);
    connection_options.connect_timeout(Duration::from_secs(1));
    connection_options.test_before_acquire(test_before_acquire);
    let connection = Database::connect(connection_options).await.unwrap();

    // Stop proxy
    handle.stop().await;

    // Check
    let checker = DatabaseChecker::new("db", &connection);
    let result = checker.status().await;
    assert!(result.is_err());
}
