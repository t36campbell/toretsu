#[cfg(feature = "client")]
mod tests {
    use redis::ConnectionAddr::{Tcp, TcpTls, Unix};
    use toretsu::client::Client;
    use toretsu::config::Config;

    #[test]
    fn generate_conn_url() {
        let config = Config::new();
        let conn_url = Client::generate_conn_url(&config);
        assert_eq!(conn_url, "redis://localhost:6379");

        std::env::set_var("REDIS_USERNAME", "user");
        std::env::set_var("REDIS_PASSWORD", "pass");

        let config_two = Config::new();
        let conn_url_two = Client::generate_conn_url(&config_two);
        assert_eq!(conn_url_two, "redis://user:pass@localhost:6379");

        std::env::set_var("REDIS_DATABASE", "test");

        let config_three = Config::new();
        let conn_url_three = Client::generate_conn_url(&config_three);
        assert_eq!(conn_url_three, "redis://user:pass@localhost:6379/test");
    }

    #[test]
    fn client_new() {
        let mut client = Client::new();
        assert!(client.check_connection());

        let info = client.get_connection_info();
        let addr = &info.addr;

        match addr {
            Tcp(host, port) => {
                assert_eq!(host, &client.config.redis_host);
                assert_eq!(port, &client.config.redis_port);
            }
            TcpTls {
                host,
                port,
                insecure,
            } => {
                assert!(!insecure);
                assert_eq!(host, &client.config.redis_host);
                assert_eq!(port, &client.config.redis_port);
            }
            Unix(_) => todo!(),
        }
    }
}
