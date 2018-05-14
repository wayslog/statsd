const defaultQuantity: isize = 1000;

pub struct ProxyConfig {
    pub nodes: Vec<Node>,
    pub bind: String,
}

pub struct Node {
    pub addr: String,
    pub priority: isize,
}
