#[cfg(feature = "pubsub")]
mod tests {
    use redis::ConnectionAddr::{Tcp, TcpTls, Unix};
    use toretsu::client::Client;
    use toretsu::config::Config;

    #[test]
    fn pubsub_listen() {
        let mut client = Client::default();
        let timeout = std::time::Duration::from_millis(250);
        client.set_read_timeout(timeout).unwrap();

        let mut c = std::mem::take(&mut client);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            c.publish("test", "Hello World!")
        });
        let m = client.listen("test");
        assert!(m.is_ok());
        let msg = m.unwrap();
        let payload: redis::RedisResult<String> = msg.get_payload();
        let channel = msg.get_channel_name();
        let content = payload.unwrap();
        assert_eq!(channel, "test");
        assert_eq!(content, "Hello World!");
    }

    #[test]
    fn pubsub_unsub() {
        let mut client = Client::new();
        let publish = client.publish("test", "Hello World!");
        assert!(publish.is_ok());

        let unsub = client.unsubscribe("test");
        assert!(unsub.is_ok());
    }
}
