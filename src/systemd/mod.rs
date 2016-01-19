extern crate regex;
extern crate ini;

use self::regex::Regex;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use self::ini::Ini;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::path::Path;

pub struct IPv4Network {
    pub ip: Ipv4Addr,
    pub prefix: u8
}

//FIXME: This is probably wrong, but at least its descriptive
pub type ValidationErrorString = &'static str;

#[derive(Debug)]
pub enum StaticIPError {
    Io(io::Error),
    Validation(ValidationErrorString),
}

pub trait NetworkConfigLoader {

    fn new(path: &'static str) -> Self;

    fn is_dhcp(&self) -> bool;

    fn ip_address(&self) -> Option<IPv4Network>;

    fn gateway(&self) -> Option<Ipv4Addr>;

    fn get_settings_section(&self) -> Option<String>;

    fn dns(&self) -> Option<Vec<Ipv4Addr>>;

    fn enable_dhcp(&self) -> Result<(), io::Error>;

    fn static_ip(&self, address: &str, gateway: &str) -> Result<(), StaticIPError>;

}

fn ip_from_match(address_match: Option<&str>, network_match: Option<&str> ) -> Option<IPv4Network> {

    let network: u8 = network_match.unwrap_or("0").parse().unwrap_or(0);

    match address_match {
        Some(result) => {
            match Ipv4Addr::from_str(result) {
                Ok(address) => {
                    return Some(IPv4Network{ ip: address, prefix: network })
                }
                Err(why) => {
                    panic!("Could not parse {} into IP address, reason: {}", result, why);
                }
            }


        },
        None => { return None },
    }
}

fn read_to_string(path: &Path) -> String {
    let display = path.display();
    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    let mut s = String::new();
    //FIXME: Add the name of the file in the expect clause
    file.read_to_string(&mut s).ok().expect("Could not read file");
    return s;
}

pub struct SystemdNetworkConfig<'a> {
    path: &'a Path
}

impl<'a> NetworkConfigLoader for SystemdNetworkConfig<'a> {

    fn new(path: &'static str) -> SystemdNetworkConfig {
        let default_path = "/etc/systemd/network/eth0.network";

        if path == "" {
            return SystemdNetworkConfig { path: Path::new(default_path) }
        }

        SystemdNetworkConfig { path: Path::new(path) }
    }

    fn is_dhcp(&self) -> bool {
        let conf = Ini::load_from_file(self.path.to_str().expect("Could to parse path")).unwrap();

        let section = conf.section(Some("Network".to_owned())).unwrap();
        let dhcp = String::new();
        let dhcp = match section.get("DHCP") {
            Some(value) => dhcp + value,
            None => String::new()
        };

        let re = Regex::new(r"(both|yes|ipv4|ipv6)").unwrap();
        return re.is_match(&dhcp[..])
    }

    fn get_settings_section(&self) -> Option<String> {
        let content = read_to_string(self.path);
        let re = Regex::new(r"(?P<settings_section>\[Network\][^\[\0]+)").unwrap();
        return match re.captures(&content[..]) {
            Some(matched) => {
                let mut settings_section = String::new();
                settings_section = settings_section + matched.name("settings_section").unwrap();
                Some(settings_section)
            },
            None => None
        };
    }

    fn ip_address(&self) -> Option<IPv4Network> {
        let content = self.get_settings_section().unwrap_or(String::new());
        //FIXME: This regex will only work on the first address
        let re = Regex::new(r"Address=(?P<address>[^/]+)\b/\b(?P<network>(?:3[0-2]|[12][0-9]|[1-9]))\b").unwrap();
        return match re.captures(&content[..]) {
            Some(matched) => ip_from_match(matched.name("address"), matched.name("network")),
            None => { None }
        };
    }

    fn gateway(&self) -> Option<Ipv4Addr> {
        let content = self.get_settings_section().unwrap_or(String::new());
        //FIXME: This regex will only work on the first address
        let re = Regex::new(r"Gateway=(?P<address>[\d.]+)").unwrap();
        return match re.captures(&content[..]) {
            Some(matched) => {
                let ip_address = ip_from_match(matched.name("address"),Some("0"));
                return Some(ip_address.unwrap().ip);
            },
            None => { None }
        };
    }

    fn dns(&self) -> Option<Vec<Ipv4Addr>> {
        let content = self.get_settings_section().unwrap_or(String::new());
        let re = Regex::new(r"DNS=(?P<address>[\d.]+)").unwrap();
        let mut dns_servers: Vec<Ipv4Addr> = Vec::new();
        for cap in re.captures_iter(&content[..]) {
            let ip_address = ip_from_match(cap.name("address"),Some("0"));
            match ip_address {
                Some(address) => dns_servers.push(address.ip),
                None => continue
            }

        }
        if dns_servers.len() > 0 {
            return Some(dns_servers)
        }
        None
    }

    fn enable_dhcp(&self) -> Result<(), io::Error>  {
        let mut conf = Ini::load_from_file(self.path.to_str().expect("Could to parse path")).unwrap();

        conf.section_mut(Some("Network")).unwrap().insert("DHCP".into(), "both".into());
        conf.delete_from(Some("Network".to_owned()), "Address");
        conf.delete_from(Some("Network".to_owned()), "Gateway");

        conf.write_to_file(self.path.to_str().expect("Could to parse path"))
    }

    fn static_ip(&self, address: &str, gateway: &str) -> Result<(), StaticIPError>  {
        let re_ip_with_cidr = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])(?:/\b(?P<network>(?:3[0-2]|[12][0-9]|[1-9]))\b)$").unwrap();
        let re_ip = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.){3}(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])$").unwrap();

        if !(re_ip_with_cidr.is_match(address) && re_ip.is_match(gateway)) {
            return Err(StaticIPError::Validation("Validation failed"));
        }

        //FIXME: Validate the inputs
        let mut conf = Ini::load_from_file(self.path.to_str().expect("Could to parse path")).unwrap();

        conf.section_mut(Some("Network")).unwrap().insert("DHCP".into(), "no".into());
        conf.section_mut(Some("Network")).unwrap().insert("Address".into(), address.into());
        conf.section_mut(Some("Network")).unwrap().insert("Gateway".into(), gateway.into());

        match conf.write_to_file(self.path.to_str().expect("Could to parse path")) {
            Ok(()) => return Ok(()),
            Err(err) => return Err(StaticIPError::Io(err))
        }
    }

}

struct UbuntuUpstartNetworkConfig<'a> {
    path: &'a Path
}

impl<'a> NetworkConfigLoader for UbuntuUpstartNetworkConfig<'a> {

    fn new(path: &'static str) -> UbuntuUpstartNetworkConfig {
        let default_path = "/etc/network/interfaces";

        if path == "" {
            return UbuntuUpstartNetworkConfig { path: Path::new(default_path) }
        }

        UbuntuUpstartNetworkConfig { path: Path::new(path) }
    }

    fn is_dhcp(&self) -> bool {
        unreachable!("Not implemented yet");
    }

    fn ip_address(&self) -> Option<IPv4Network> {
        unreachable!("Not implemented yet");
    }

    fn gateway(&self) -> Option<Ipv4Addr> {
        unreachable!("Not implemented yet");
    }

    fn get_settings_section(&self) -> Option<String> {
        unreachable!("Not implemented yet");
    }

    fn dns(&self) -> Option<Vec<Ipv4Addr>> {
       unreachable!("Not implemented yet");
    }

    fn enable_dhcp(&self) -> Result<(), io::Error>{
        unreachable!("Not implemented yet");
    }

    fn static_ip(&self, address: &str, gateway: &str) -> Result<(), StaticIPError> {
        unreachable!("Not implemented yet");
    }

}

enum NetworkConfig<'a> {
    Systemd(SystemdNetworkConfig<'a>),
    Ubuntu(UbuntuUpstartNetworkConfig<'a>)
}

#[test]
fn default_config_systemd() {
    let config = SystemdNetworkConfig::new("");
    assert!(config.path.to_str().unwrap() == "/etc/systemd/network/eth0.network");
}

#[test]
fn default_config_ubuntu_upstart() {
    let config = UbuntuUpstartNetworkConfig::new("");
    assert!(config.path.to_str().unwrap() == "/etc/network/interfaces");
}

#[test]
fn config_section_systemd() {
    let config = SystemdNetworkConfig::new("./tests/eth0.network.dhcp");
    let content = config.get_settings_section().unwrap_or(String::new());
    assert!(content.contains("[Network]"));
}

#[test]
fn no_config_section_systemd() {
    let config = SystemdNetworkConfig::new("./tests/eth0.network.nonetwork");
    let content = config.get_settings_section().unwrap_or(String::new());
    assert!(!content.contains("[Network]"));
}

#[test]
fn read_dhcp_config_systemd() {
    let config = SystemdNetworkConfig::new("./tests/eth0.network.dhcp");
    assert!(config.is_dhcp() == true);
}

#[test]
fn read_non_dhcp_config_systemd() {
    let config = SystemdNetworkConfig::new("./tests/eth0.network.static");
    assert!(config.is_dhcp() == false);
}

#[test]
fn read_static_config_systemd() {
    let config = SystemdNetworkConfig::new("./tests/eth0.network.static");
    let comparitor_ip = Ipv4Addr::from_str("192.168.1.3").unwrap();
    let ip_address = config.ip_address().expect("No ip address found");
    assert!(ip_address.ip == comparitor_ip);
    assert!(ip_address.prefix == 24);

    let comparitor_gw = Ipv4Addr::from_str("192.168.1.1").unwrap();
    assert!(config.gateway().expect("No gateway address found") == comparitor_gw);
}

#[test]
fn read_dns_config_systemd() {
    let config = SystemdNetworkConfig::new("./tests/eth0.network.static_with_dns");
    let dns_servers = config.dns().unwrap_or(Vec::new());
    assert!(dns_servers.len() == 4);
}

#[test]
fn read_no_dns_config_systemd() {
    let config = SystemdNetworkConfig::new("./tests/eth0.network.static");
    let dns_servers = config.dns().unwrap_or(Vec::new());
    assert!(dns_servers.len() == 0);
}

#[test]
fn enable_dhcp_systemd() {
    fs::copy("./tests/eth0.network.static", "./tests/eth0.network.static.test").unwrap();

    let config = SystemdNetworkConfig::new("./tests/eth0.network.static.test");
    assert!(config.is_dhcp() == false);
    config.enable_dhcp().unwrap();
    assert!(config.is_dhcp() == true);
}

#[test]
fn static_ip_systemd() {
    fs::copy("./tests/eth0.network.dhcp", "./tests/eth0.network.dhcp.test").unwrap();

    let config = SystemdNetworkConfig::new("./tests/eth0.network.dhcp.test");
    assert!(config.is_dhcp() == true);
    config.static_ip("192.168.1.3/24","192.168.1.1").unwrap();
    assert!(config.is_dhcp() == false);

    let comparitor_ip = Ipv4Addr::from_str("192.168.1.3").unwrap();
    let ip_address = config.ip_address().expect("No ip address found");
    assert!(ip_address.ip == comparitor_ip);
    assert!(ip_address.prefix == 24);

    let comparitor_gw = Ipv4Addr::from_str("192.168.1.1").unwrap();
    assert!(config.gateway().expect("No gateway address found") == comparitor_gw);
}

#[test]
fn static_ip_invalid_values_systemd() {
    fs::copy("./tests/eth0.network.dhcp", "./tests/eth0.network.dhcp.validation").unwrap();

    let config = SystemdNetworkConfig::new("./tests/eth0.network.dhcp.validation");
    assert!(config.is_dhcp() == true);
    assert!(config.static_ip("192.broken.1.3/24","192.168.1.1").is_err());
    assert!(config.static_ip("192.300.1.3/24","192.168.1.1").is_err());
    assert!(config.static_ip("192.300.1.3/24","192.168.1000.1").is_err());
    assert!(config.static_ip("192.300.1.3/33","192.168.1.1").is_err());
    assert!(config.static_ip("192.168.1.3/33","192.168.1.1").is_err());
    assert!(config.static_ip("192.168.1.3","192.168.1.1").is_err());
    assert!(config.static_ip("192.168.1.3/31","192.168.1.1").is_ok());

}
