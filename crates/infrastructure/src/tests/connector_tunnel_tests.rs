use super::*;
use db_pro_core::domain::connection::{ConnectionHandle, DriverType};
use db_pro_core::domain::query::{QueryParam, QueryResult};
use db_pro_core::domain::schema::IntrospectResult;
use db_pro_core::ports::{SqlDialect, TransactionFailure, TransactionStatementResult};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct SocketDisconnectProvider {
    port: u16,
    fail_disconnect: bool,
    observed_live_socket: Arc<AtomicBool>,
}

#[async_trait]
impl DbConnector for SocketDisconnectProvider {
    async fn disconnect(&self, _: &ConnectionHandle) -> Result<(), DbError> {
        // Cross an await boundary before the provider uses its transport again.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let mut socket = tokio::net::TcpStream::connect(("127.0.0.1", self.port)).await.unwrap();
        socket.write_all(b"ping").await.unwrap();
        let mut reply = [0; 4];
        socket.read_exact(&mut reply).await.unwrap();
        assert_eq!(&reply, b"ping");
        self.observed_live_socket.store(true, Ordering::SeqCst);
        if self.fail_disconnect {
            Err(DbError::ConnectionFailed("provider close failed".into()))
        } else {
            Ok(())
        }
    }

    async fn connect(&self, _: &ConnectionConfig, _: &str) -> Result<ConnectionHandle, DbError> {
        unreachable!("disconnect fixture")
    }
    async fn test_connection(&self, _: &ConnectionConfig, _: &str) -> Result<(), DbError> {
        unreachable!("disconnect fixture")
    }
    async fn query(&self, _: &ConnectionHandle, _: &str, _: &[QueryParam]) -> Result<QueryResult, DbError> {
        unreachable!("disconnect fixture")
    }
    async fn execute(&self, _: &ConnectionHandle, _: &str, _: &[QueryParam]) -> Result<u64, DbError> {
        unreachable!("disconnect fixture")
    }
    async fn cancel(&self, _: &ConnectionHandle) -> Result<(), DbError> {
        unreachable!("disconnect fixture")
    }
    async fn execute_batch(&self, _: &ConnectionHandle, _: &[String]) -> Result<u64, DbError> {
        unreachable!("disconnect fixture")
    }
    async fn execute_transaction(
        &self,
        _: &ConnectionHandle,
        _: &[String],
        _: &[bool],
    ) -> Result<Vec<TransactionStatementResult>, TransactionFailure> {
        unreachable!("disconnect fixture")
    }
    async fn introspect(&self, _: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        unreachable!("disconnect fixture")
    }
    async fn explain(
        &self,
        _: &ConnectionHandle,
        _: &str,
        _: &[QueryParam],
        _: bool,
    ) -> Result<serde_json::Value, DbError> {
        unreachable!("disconnect fixture")
    }
    fn dialect(&self, _: &ConnectionHandle) -> Result<Box<dyn SqlDialect>, DbError> {
        unreachable!("disconnect fixture")
    }
}

async fn disconnect_with_local_tunnel(fail_disconnect: bool) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    // Python is a local socket-owning child, not an SSH/database-auth substitute.
    let child = tokio::process::Command::new("python3")
        .args([
            "-c",
            "import socket,sys\ns=socket.socket();s.bind(('127.0.0.1',int(sys.argv[1])));s.listen()\nwhile True:\n c,_=s.accept()\n with c:\n  data=c.recv(4)\n  if data:c.sendall(data)\n",
            &port.to_string(),
        ])
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let tunnel = crate::ssh::SshTunnelHandle::from_test_process(port, child)
        .await
        .unwrap();
    let observed = Arc::new(AtomicBool::new(false));
    let composite = CompositeConnector::new();
    composite
        .connections
        .write()
        .unwrap_or_else(|error| error.into_inner())
        .insert(
            1,
            ActiveConnection {
                connector: Arc::new(SocketDisconnectProvider {
                    port,
                    fail_disconnect,
                    observed_live_socket: Arc::clone(&observed),
                }),
                inner_handle: ConnectionHandle(41),
                driver: DriverType::Postgres,
                tunnel: Some(tunnel),
            },
        );

    let result = composite.disconnect(&ConnectionHandle(1)).await;
    assert!(observed.load(Ordering::SeqCst));
    assert_eq!(result.is_err(), fail_disconnect);
    if let Err(error) = result {
        assert!(matches!(error, DbError::ConnectionFailed(message) if message == "provider close failed"));
    }
    assert!(composite
        .connections
        .read()
        .unwrap_or_else(|error| error.into_inner())
        .is_empty());
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("tunnel child must be cleaned up after provider disconnect settles");
}

#[tokio::test]
async fn disconnect_keeps_tunnel_socket_alive_until_provider_close_finishes() {
    disconnect_with_local_tunnel(false).await;
}

#[tokio::test]
async fn disconnect_error_keeps_tunnel_alive_then_cleans_up_and_preserves_error() {
    disconnect_with_local_tunnel(true).await;
}

#[tokio::test]
async fn connect_and_test_never_bypass_a_failed_tunnel_to_a_reachable_database_port() {
    for driver in [DriverType::Postgres, DriverType::Mysql] {
        let database = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        database.set_nonblocking(true).unwrap();
        let unavailable_ssh = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let ssh_port = unavailable_ssh.local_addr().unwrap().port();
        drop(unavailable_ssh);
        let config = ConnectionConfig {
            name: "unreachable SSH leg".into(),
            host: "127.0.0.1".into(),
            port: database.local_addr().unwrap().port(),
            database: "app".into(),
            username: "app".into(),
            driver,
            ssh_tunnel: Some(db_pro_core::domain::connection::SshTunnelConfig {
                host: "127.0.0.1".into(),
                port: ssh_port,
                user: "fixture".into(),
                private_key_path: "/dev/null".into(),
                password: None,
            }),
            ..Default::default()
        };
        let composite = CompositeConnector::new();
        for test_only in [true, false] {
            let error = tokio::time::timeout(std::time::Duration::from_secs(3), async {
                if test_only {
                    composite.test_connection(&config, "unused").await
                } else {
                    composite.connect(&config, "unused").await.map(|_| ())
                }
            })
            .await
            .expect("local refused SSH connection must fail promptly")
            .expect_err("reachable direct database must never bypass failed SSH");
            assert!(matches!(error, DbError::ConnectionFailed(message) if message.starts_with("SSH tunnel to ")));
            assert_eq!(database.accept().unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
        }
    }
}
