pub struct ImageDownloader {
    pub ip: String,
    pub port: String,
}

impl ImageDownloader {
    fn new(ip: String, port: String) -> Self {



        ImageDownloader { ip, port }
    }
}