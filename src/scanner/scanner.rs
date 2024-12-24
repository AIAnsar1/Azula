
use log::debug;
use crate::config::base::{SocketIterator};
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::{io, net::UdpSocket};
use colored::Colorize;
use futures::stream::FuturesUnordered;
use std::collections::BTreeMap;
use std::{collections::HashSet, net::{IpAddr, Shutdown, SocketAddr}, num::NonZeroU8, time::Duration};
use itertools::Itertools;
use azula::net::strategy::PortStrategy;
use crate::support::generate::get_parsed_data;



#[cfg(not(tarpaulin_include))]
#[derive(Debug)]
pub struct Scanner {
    ips: Vec<IpAddr>,
    batch_size: u16,
    timeout: Duration,
    tries: NonZeroU8,
    greppable: bool,
    port_strategy: PortStrategy,
    accessible: bool,
    exclude_ports: Vec<u16>,
    udp: bool,
}




#[allow(clippy::too_many_arguments)]
impl Scanner {
    pub fn new(ips: &[IpAddr], batch_size: u16, timeout: Duration, tries: u8, greppable: bool, port_strategy: PortStrategy, accessible: bool, exclude_ports: Vec<u16>, udp: bool) -> Self {
        Self {
            batch_size,
            timeout,
            tries: NonZeroU8::new(std::cmp::max(tries, 1)).unwrap(),
            greppable,
            port_strategy,
            ips: ips.iter().map(ToOwned::to_owned).collect(),
            accessible,
            exclude_ports,
            udp,
        }
    }


    pub async fn run(&self) -> Vec<SocketAddr> {
        let ports: Vec<u16> = self.port_strategy.order().iter().filter(|&port| !self.exclude_ports.contains(port)).copied().collect();
        let mut socket_iteraotr: SocketIterator = SocketIterator::new(&self.ips, &ports);
        let mut open_socket: Vec<SocketAddr> = Vec::new();
        let mut ftrs = FuturesUnordered::new();
        let mut errors: HashSet<String> = HashSet::new();
        let udp_map = get_parsed_data();

        for _ in 0..self.batch_size {
            if let Some(socket) = socket_iteraotr.next() {
                ftrs.push(self.scan_socket(socket, udp_map.clone()));
            } else  {
                break;
            }
        }
        debug!("Start Scaning Sockets. \nBatch Size: [ {} ]\nNumber Of IP-S [ {} ]\nNumber Of Ports [ {} ]\nTargets All Together [ {} ]\n",
        self.batch_size, self.ips.len(), &ports.len(), (self.ips.len() * ports.len()));

        while let Some(result) = ftrs.next().await {
            if let Some(socket) = socket_iteraotr.next() {
                ftrs.push(self.scan_socket(socket, udp_map.clone()));
            }

            match result {
                Ok(socket) => open_socket.push(socket),
                Err(e) => {
                    let error_string = e.to_string();
                    if errors.len() < self.ips.len() * 1000 {
                        errors.insert(error_string);
                    }
                }
            }
            debug!("[ ERROR ]: Typical Socket Connection [ {:?} ]", errors);
            debug!("Open Sockets Found: [ {:?} ]", &open_socket);
            open_socket
        }
    }




    async fn scan_socket(&self, socket: SocketAddr, udp_map: BTreeMap<Vec<u16>, Vec<u8>>) -> io::Result<SocketAddr> {
        if self.udp {
            return self.scan_udp_socket(socket, udp_map).await;
        }
        let tries = self.tries.get();

        for nr_try in 1..=tries {
            match self.connect(socket).await {
                Ok(tcp_stream) => {
                    debug!("Connection Was Successfully, Shutting Down Stream {}", &socket);

                    if let Err(e) = tcp_stream.shutdown(Shutdown::Both) {
                        debug!("[ ERROR ]:Shutdown Stream {}", &e);
                    }
                    self.fmt_ports(socket);
                    debug!("Return OK after {} tries", nr_try);
                    return Ok(socket);
                }
                Err(e) => {
                    let mut error_string = e.to_string();
                    assert!(!error_string.to_lowercase().contains("Too Many Open Files"), "Too Many Open Files, Please Reduce Batch Size. The Default is 5000 Try -b 2500.");

                    if nr_try == tries {
                        error_string.push(' ');
                        error_string.push_str(&socket.ip().to_string());
                        return Err(io::ErrorKind::Other, error_string)
                    }
                }
            }
        }
    }


    async fn scan_udp_socket(&self, socket: SocketAddr, udp_map: BTreeMap<Vec<u16>, Vec<u8>>) -> io::Result<SocketAddr> {
        let mut payload: Vec<u8> = Vec::new();

        for (key, value) in udp_map {
            if key.contains(&socket.port()) {
                payload = value
            }
        }
        let tries = self.tries.get();

        for _ in 1..=tries {
            match self.udp_scan(socket, &payload, self.timeout).await {
                Ok(true) => return Ok(socket),
                Ok(false) => continue,
                Err(e) =>  return Err(e),
            }
        }
        Ok(socket)
    }


    async fn connect(&self, socket: SocketAddr) -> io::Result<TcpStream> {
        let stream = io::timeout(self.timeout, async move {
            TcpStream::connect(socket).await
        }).await?;
        Ok(stream)
    }

    async fn udp_bind(&self, socket: SocketAddr) -> io::Result<UdpSocket> {
        let local_address = match socket {
            SocketAddr::V4(_) => "0.0.0.0".parse::<SocketAddr>().unwrap(),
            SocketAddr::V6(_) => "[::]:0".parse::<SocketAddr>().unwrap(),
        };
        UdpSocket::bind(local_address).await
    }

    async fn udp_scan(&self, socket: SocketAddr, payload: &[u8], wait: Duration) -> io::Result<bool> {
        match self.udp_bind(socket).await {
            Ok(udp_socket) => {
                let mut buf = [0u8; 1024];
                udp_socket.connect(socket).await?;
                udp_socket.send(payload).await?;

                match io::timeout(wait, udp_socket.recv(&mut buf)).await {
                    Ok(size) => {
                        debug!("Received {} bytes", size);
                        self.fmt_ports(socket);
                        Ok(true)
                    }
                    Err(e) => {
                        if e.kind() == io::ErrorKind::TimedOut {
                            Ok(false)
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            Err(e) => {
                println!("[ ERROR ]: E Binding Socket {:?}", e);
                Err(e)
            }
        }
    }

    fn fmt_ports(&self, socket: SocketAddr) {
        if !self.greppable {
            if self.accessible {
                println!("Open {Socket}");
            } else {
                println!("Open {}", socket.to_string().purple());
            }
        }
    }

}

