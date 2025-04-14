use std::time::Duration;

use redis::{
    Client as Redis, Commands, Connection, ConnectionInfo, ConnectionLike, ControlFlow, Msg,
    RedisResult, ToRedisArgs,
};

use crate::config::Config;

pub struct Client {
    pub config: Config,
    pub connection: Connection,
    pub redis: Redis,
}

#[cfg(not(tarpaulin_include))]
impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(tarpaulin_include))]
impl Client {
    pub fn generate_conn_url(config: &Config) -> String {
        let host = &config.redis_host;
        let port = config.redis_port.to_string();
        let user = match config.redis_user.as_ref() {
            Some(user) => user,
            None => "",
        };
        let pass = match config.redis_pass.as_ref() {
            Some(pass) => pass,
            None => "",
        };
        let db = match config.redis_db.as_ref() {
            Some(db) => db,
            None => "",
        };

        let use_auth = !user.is_empty() && !pass.is_empty();
        let use_db = !db.is_empty();
        match (use_auth, use_db) {
            (use_auth, use_db) if use_auth && use_db => {
                format!("redis://{user}:{pass}@{host}:{port}/{db}")
            }
            (use_auth, use_db) if use_auth && !use_db => {
                format!("redis://{user}:{pass}@{host}:{port}")
            }
            (use_auth, use_db) if !use_auth && use_db => format!("redis://{host}:{port}/{db}"),
            _ => format!("redis://{host}:{port}"),
        }
    }

    pub fn new() -> Self {
        let config = Config::new();
        let timeout = Duration::from_secs(30);
        let conn_url = Self::generate_conn_url(&config);
        let redis = Redis::open(conn_url).expect("Failed to Locate Redis");
        let connection = redis
            .get_connection_with_timeout(timeout)
            .expect("Failed to Connect to Redis");

        Self {
            config,
            connection,
            redis,
        }
    }

    pub fn check_connection(&mut self) -> bool {
        self.connection.check_connection()
    }

    pub fn get_connection_info(&self) -> &ConnectionInfo {
        self.redis.get_connection_info()
    }

    pub fn set_read_timeout(&mut self, timeout: Duration) -> RedisResult<()> {
        self.connection.set_read_timeout(Some(timeout))
    }

    pub fn publish<K: ToRedisArgs, E: ToRedisArgs>(
        &mut self,
        channel: K,
        message: E,
    ) -> RedisResult<()> {
        self.connection.publish(channel, message)
    }

    pub fn subscribe<T: ToRedisArgs>(&mut self, channel: T) -> RedisResult<()> {
        let mut pubsub = self.connection.as_pubsub();
        pubsub.subscribe(channel)
    }

    pub fn get_message(&mut self) -> RedisResult<Msg> {
        let mut pubsub = self.connection.as_pubsub();
        pubsub.get_message()
    }

    pub fn listen<C, F, U>(&mut self, channels: C, mut func: F)
    where
        C: ToRedisArgs + std::marker::Send + 'static,
        F: FnMut(Msg) -> ControlFlow<U> + std::marker::Send + 'static,
    {
        let mut client = std::mem::take(self);
        rayon::spawn(move || {
            let _ = client.subscribe(channels);
            loop {
                let response = match &client.connection.recv_response() {
                    Ok(res) => Msg::from_value(res),
                    Err(_) => continue,
                };

                let msg = match response {
                    Some(msg) => msg,
                    None => continue,
                };

                match func(msg) {
                    ControlFlow::Continue => continue,
                    ControlFlow::Break(_) => break,
                }
            }
        });
    }

    pub fn unsubscribe<T: ToRedisArgs>(&mut self, channel: T) -> RedisResult<()> {
        let mut pubsub = self.connection.as_pubsub();
        pubsub.unsubscribe(channel)
    }
}
