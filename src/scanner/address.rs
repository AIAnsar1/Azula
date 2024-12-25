use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{prelude::*, BufReader};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::ptr::read;
use std::str::FromStr;
use cidr_utils::cidr::IpCidr;
use hickory_resolver::{config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts}, Resolver};
use azula::warning;
use log::debug;
use crate::config::configuration::*;
use crate::config::base::{Opts};
use crate::support::*;




/*


pub fn parse_addresses(input: &Opts) -> Vec<IpAddr> {
    let mut ips: Vec<IpAddr> = Vec::new();
    let mut unresolved_addresses: Vec<&str> = Vec::new();
    let backup_resolver = get_resolver(&input.resolver);

    for address in &input.addresses {
        let parse_ips = parse_address(address, &backup_resolver);

        if !parse_ips.is_empty() {
            ips.extend(parse_ips);
        } else {
            unresolved_addresses.push(address);
        }
    }

    for file_path in unresolved_addresses {
        let file_path = Path::new(file_path);

        if !file_path.is_file() {
            warning!(format!("Host {file_path:?} Could Not Be Resolved."), input.greppable, input.accessible);
            continue;
        }

        if let Ok(X) = read_ips_from_file(file_path, &backup_resolver) {
            ips.extend(X);
        } else {
            warning!(format!("Host {file_path:?} Could Not Be Resolved."), input.greppable, input.accessible);
        }
    }
    ips.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

*/



pub fn parse_addresses(input: &Opts) -> Vec<IpAddr> {
    let backup_resolver = get_resolver(&input.resolver);
    let mut ips: BTreeSet<IpAddr> = BTreeSet::new();
    let mut unresolved_addresses = Vec::new();

    for address in &input.addresses {
        let parsed_ips = parse_address(address, &backup_resolver);

        if parsed_ips.is_empty() {
            unresolved_addresses.push(address);
        } else {
            ips.extend(parsed_ips);
        }
    }

    for unresolved in unresolved_addresses {
        let file_path = Path::new(unresolved);

        if !file_path.is_file() {
            warning!(format!("Host [ {file_path:?} ] Cloud Not Be Resolved."), input.greppable, input.accessible);
            continue;
        }

        match read_ips_from_file(file_path, &backup_resolver) {
            Ok(file_ips) => ips.extend(file_ips),
            Err(_) => warning!(format!("Host [ {file_path:?} ] Cloud Not Be Resolved."), input.greppable, input.accessible),
        }
    }

    ips.into_iter().collect()
}




/*
pub fn parse_address(address: &str, resolver: &Resolver) -> Vec<IpAddr> {
    IpCidr::from_str(address).map(|cidr| cidr.iter().map(|c| c.address()).collect()).ok().or_else(|| {
        format!("[ {}:80 ]", &address).to_scoket_addrs().ok().map(|mut iter| vec![iter.next().unwrap().ip()])
    }).unwrap_or_else(|| resolve_ips_from_host(address, resolver))
}
*/


pub fn parse_address(address: &str, resolver: &Resolver) -> Vec<IpAddr>  {
    if let Ok(cidr) = IpCidr::from_str(address) {
        return cidr.iter().map(|c| c.address()).collect();
    }

    if let Ok(mut socket_address) = format!("[ {}:80 ]", address).to_socket_addrs() {
        if let Some(address) = socket_address.next() {
            return vec![address.ip()]
        }
    }
    resolve_ips_from_host(address, resolver)
}


/*

fn resolve_ips_from_host(source: &str, backup_resolver: &Resolver) -> Vec<IpAddr>  {
    let mut ips: Vec<IpAddr> = Vec::new();

    if let Ok(address) = source.to_socket_addrs() {
        for ip in address{
            ips.push(ip.ip())
        }
    } else if let Ok(address) = backup_resolver.lookup_ip(source) {
        ips.extend(address.iter());
    }
    ips
}
*/

fn resolve_ips_from_host(source: &str, backup_resolver: &Resolver) -> Vec<IpAddr> {
    if let Ok(addresses) = source.to_socket_addrs() {
        addresses.map(|address| address.ip()).collect()
    } else if let Ok(lookup) = backup_resolver.lookup_ip(source) {
        lookup.iter().collect()
    } else {
        Vec::new()
    }
}

/*
fn get_resolver(resolver: &Option<String>) -> Resolver {
    match resolver {
        Some(r) => {
            let mut config = ResolverConfig::new();
            let resolver_ips = match read_resolver_from_file(r) {
                Ok(ips) => ips,
                Err(_) => r.split(',').filter_map(|r| IpAddr::from_str(r).ok()).collect::<Vec<_>>()
            };


            for ip in resolver_ips {
                config.add_name_server(NameServerConfig::new(SocketAddr::new(ip, 53), Protocol::Udp));
            }
            Resolver::new(config, ResolverOpts::default()).unwrap()
        }
        None => match Resolver::from_system_conf() {
            Ok(resolver) => resolver,
            Err(_) => {
                Resolver::new(ResolverConfig::cloudflare_tls(), ResolverOpts::default()).unwrap()
            }
        },
    }
}
*/

pub fn get_resolver(resolver: &Option<String>) -> Resolver {
    match resolver {
        Some(R) => {
            let resolver_ips = read_resolver_from_file(R).unwrap_or_else(|_| {
                R.split(',').filter_map(|ip| IpAddr::from_str(ip.trim()).ok()).collect()
            });

            let config = ResolverConfig::from_parts(None, vec![], resolver_ips.into_iter().map(|ip|
                NameServerConfig::new(SocketAddr::new(ip, 53), Protocol::Udp)).collect());

            Resolver::new(config, ResolverOpts::default()).unwrap_or_else(|_| {
                panic!("Failed To Create Resolver With The Provide Configuration")
            });
        }
        None => Resolver::from_system_conf().unwrap_or_else(|_| {
            Resolver::new(ResolverConfig::cloudflare_tls(), ResolverOpts::default()).unwrap()
        })
    }
}


/*
fn read_resolver_from_file(path: &str) -> Result<Vec<IpAddr>, std::io::Error> {
    let ips = fs::read_to_string(path)?.lines().filter_map(|line| IpAddr::from_str(line.trim()).ok()).collect()
}

*/

fn read_resolver_from_file(path: &str) -> Result<Vec<IpAddr>, std::io::Error> {
    // считаем файл обрабатываем строки сразу фильтруя и парся IP адреса
    let ips: Vec<IpAddr> = fs::read_to_string(path)?.lines().map(str::trim).filter_map(|line| IpAddr::from_str(line).ok()).collect();
    Ok(ips)
}


/*
#[cfg(not(tarpaulin_include))]
/// Parses an input file of IPs and uses those
fn read_ips_from_file(ips: &std::path::Path,backup_resolver: &Resolver) -> Result<Vec<IpAddr>, std::io::Error> {
    let file = File::open(ips)?;
    let reader = BufReader::new(file);

    let mut ips: Vec<IpAddr> = Vec::new();

    for address_line in reader.lines() {
        if let Ok(address) = address_line {
            ips.extend(parse_address(&address, backup_resolver));
        } else {
            debug!("Line in file is not valid");
        }
    }

    Ok(ips)
}
*/
#[cfg(not(tarpaulin_include))]
fn read_ips_from_file(path: &std::path::Path, backup_resolver: &Resolver) -> Result<Vec<IpAddr>, std::io::Error> {
    // Открытие файла и оборачивание в буферизированый ридер
    let reader = BufReader::new(File::open(path)?);

    // переход по строкам файла и парсинг адресов
    let ips: Vec<IpAddr>  = reader.lines().filter_map(|line| match line {
        Ok(address) => Some(parse_address(&address, backup_resolver)),
        Err(_) => {
            debug!("Line In FIle Is Not Valid.");
            None
        }
    }).flatten().collect(); // Разворачивание Вектор в Последовательность
    Ok(ips)
}


















