use jsonrpc::{
    client::Client,
    Error
};

pub struct Network {
    pub client: Client
}

impl Network {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            client: Client::new("http://localhost:21150".to_owned(), None, None)
        })
    }
}
