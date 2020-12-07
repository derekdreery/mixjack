use jack::Client;

#[derive(Debug)]
pub struct Info {
    name: String,
    sample_rate: usize,
    buffer_size: u32,
}

impl Info {
    pub fn from_client(client: &Client) -> Self {
        Self {
            name: String::from(client.name()),
            sample_rate: client.sample_rate(),
            buffer_size: client.buffer_size(),
        }
    }

    pub fn log(&self) {
        log::info!("\"{}\" jack audio client", self.name);
        log::info!("  sample rate: {}", self.sample_rate);
        log::info!("  buffer size: {}", self.buffer_size);
    }
}
