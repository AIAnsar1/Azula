use itertools::{iproduct, Product};
use std::net::{IpAddr, SocketAddr};
use crate::config::base::SocketIterator;





impl <'s>SocketIterator<'s> {
    pub fn new(ips: &'s [IpAddr], ports: &'s [u16]) -> Self {
        let ports_it = Box::new(ports.iter());
        let ips_it = Box::new(ips.iter());

        Self {
            product_it: iproduct!(ports_it, ips_it)
        }
    }
}


#[allow(clippy::doc_link_with_quotes)]
impl Iterator for SocketIterator<'_> {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.product_it.next().map(|(port, ip)| SocketAddr::new(*ip, *port))
    }
}

#[cfg(test)]
mod test {
    use super::SocketIterator;
    use std::net::{IpAddr, SocketAddr};

    #[test]
    fn goes_through_every_ip_port_combination_test() {
        let address = vec!["127.0.0.1".parse::<IpAddr>().unwrap(), "192.168.0.1".parse::<IpAddr>().unwrap()];
        let ports: Vec<u16> = vec![22, 80, 443];
        let mut it = SocketIterator::new(&address, &ports);

        assert_eq!(Some(SocketAddr::new(address[0], ports[0])), it.next());
        assert_eq!(Some(SocketAddr::new(address[1], ports[0])), it.next());
        assert_eq!(Some(SocketAddr::new(address[0], ports[1])), it.next());
        assert_eq!(Some(SocketAddr::new(address[1], ports[1])), it.next());
        assert_eq!(Some(SocketAddr::new(address[0], ports[2])), it.next());
        assert_eq!(Some(SocketAddr::new(address[1], ports[2])), it.next());
        assert_eq!(None, it.next())
    }

}




