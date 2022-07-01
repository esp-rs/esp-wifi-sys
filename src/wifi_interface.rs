use core::fmt::Display;
use embedded_svc::ipv4::Ipv4Addr;

use embedded_svc::{
    ipv4::{ClientSettings, Mask, Subnet},
    wifi::{
        AccessPointInfo, ApStatus, AuthMethod, ClientConnectionStatus, ClientIpStatus,
        ClientStatus, SecondaryChannel, Status,
    },
};
use enumset::EnumSet;
use smoltcp::iface::{Interface, SocketHandle};
use smoltcp::socket::{Dhcpv4Socket, Socket};
use smoltcp::time::Instant;
use smoltcp::wire::{IpAddress, IpCidr};

use crate::current_millis;
use crate::wifi::WifiDevice;

extern crate alloc;

/// An implementation of `embedded-svc`'s wifi trait.
pub struct Wifi<'a> {
    network_interface: Interface<'a, WifiDevice>,
    current_config: embedded_svc::wifi::Configuration,
    network_config: Option<smoltcp::socket::Dhcpv4Config>,
    dhcp_socket_handle: Option<SocketHandle>,
}

impl<'a> Wifi<'a> {
    /// Create a new instance from a `NetworkStack`
    pub fn new(mut network_interface: Interface<'a, WifiDevice>) -> Wifi<'a> {
        let mut dhcp_socket_handle: Option<SocketHandle> = None;

        for (handle, socket) in network_interface.sockets_mut() {
            match socket {
                Socket::Dhcpv4(_) => dhcp_socket_handle = Some(handle),
                _ => {}
            }
        }

        Wifi {
            network_interface,
            current_config: embedded_svc::wifi::Configuration::default(),
            network_config: None,
            dhcp_socket_handle,
        }
    }

    /// Get a mutable reference to the `NetworkStack`
    pub fn network_interface(&mut self) -> &mut Interface<'a, WifiDevice> {
        &mut self.network_interface
    }

    /// Convenience function to poll the DHCP socket.
    pub fn poll_dhcp(&mut self) -> Result<(), WifiError> {
        if let Some(dhcp_handle) = self.dhcp_socket_handle {
            let dhcp_socket = self
                .network_interface
                .get_socket::<Dhcpv4Socket>(dhcp_handle);
            let event = dhcp_socket.poll();
            if let Some(event) = event {
                match event {
                    smoltcp::socket::Dhcpv4Event::Deconfigured => {
                        self.network_config = None;
                        self.network_interface
                            .routes_mut()
                            .remove_default_ipv4_route();
                    }
                    smoltcp::socket::Dhcpv4Event::Configured(config) => {
                        self.network_config = Some(config);
                        let address = config.address;
                        self.network_interface.update_ip_addrs(|addrs| {
                            let addr = addrs
                                .iter_mut()
                                .filter(|cidr| match cidr.address() {
                                    IpAddress::Ipv4(_) => true,
                                    _ => false,
                                })
                                .next()
                                .unwrap();

                            *addr = IpCidr::Ipv4(address);
                        });
                        if let Some(route) = config.router {
                            self.network_interface
                                .routes_mut()
                                .add_default_ipv4_route(route)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WifiError {
    Unknown(i32),
    SmolTcpError(smoltcp::Error),
}

impl From<smoltcp::Error> for WifiError {
    fn from(error: smoltcp::Error) -> Self {
        WifiError::SmolTcpError(error)
    }
}

impl Display for WifiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> embedded_svc::wifi::Wifi for Wifi<'a> {
    type Error = WifiError;

    /// This currently only supports the `Client` capability.
    fn get_capabilities(&self) -> Result<EnumSet<embedded_svc::wifi::Capability>, Self::Error> {
        // for now we only support STA mode
        let mut caps = EnumSet::empty();
        caps.insert(embedded_svc::wifi::Capability::Client);
        Ok(caps)
    }

    /// Get the wifi status.
    /// Please note: To ever get into the state of an assigned IP address you need to make sure
    /// that `poll` is called frequently on the network stack and dhcp socket.
    fn get_status(&self) -> Status {
        match crate::wifi::get_wifi_state() {
            crate::wifi::WifiState::WifiReady => Status(ClientStatus::Stopped, ApStatus::Stopped),
            crate::wifi::WifiState::StaStart => Status(ClientStatus::Starting, ApStatus::Stopped),
            crate::wifi::WifiState::StaStop => Status(ClientStatus::Stopped, ApStatus::Stopped),
            crate::wifi::WifiState::StaConnected => {
                let client_ip_status = if let Some(ip) = self.network_interface.ipv4_addr() {
                    if !ip.is_unspecified() {
                        let mut ip_bytes: [u8; 4] = [0; 4];
                        ip_bytes.copy_from_slice(ip.as_bytes());

                        let mut gw_bytes: [u8; 4] = [0; 4];
                        let mut dns_bytes: [u8; 4] = [0; 4];
                        if let Some(config) = self.network_config {
                            if let Some(router) = config.router {
                                gw_bytes.copy_from_slice(router.as_bytes());
                            }

                            if let Some(dns_server) = config.dns_servers[0] {
                                dns_bytes.copy_from_slice(dns_server.as_bytes());
                            }
                        }

                        ClientIpStatus::Done(ClientSettings {
                            ip: Ipv4Addr::from(ip_bytes),
                            subnet: Subnet {
                                gateway: Ipv4Addr::from(gw_bytes),
                                mask: Mask(24), // where to get this from?
                            },
                            dns: Some(Ipv4Addr::from(dns_bytes)),
                            secondary_dns: Some(Ipv4Addr::new(0, 0, 0, 0)),
                        })
                    } else {
                        ClientIpStatus::Waiting
                    }
                } else {
                    ClientIpStatus::Waiting
                };

                Status(
                    ClientStatus::Started(ClientConnectionStatus::Connected(client_ip_status)),
                    ApStatus::Stopped,
                )
            }
            crate::wifi::WifiState::StaDisconnected => Status(
                ClientStatus::Started(ClientConnectionStatus::Disconnected),
                ApStatus::Stopped,
            ),
            crate::wifi::WifiState::Invalid => Status(ClientStatus::Stopped, ApStatus::Stopped),
        }
    }

    /// A blocking wifi network scan.
    fn scan_n<const N: usize>(
        &mut self,
    ) -> Result<(heapless::Vec<AccessPointInfo, N>, usize), Self::Error> {
        crate::wifi::wifi_start_scan();

        let mut scanned = heapless::Vec::<AccessPointInfo, N>::new();
        let mut bss_total: u16 = N as u16;

        unsafe {
            crate::binary::include::esp_wifi_scan_get_ap_num(&mut bss_total);

            let mut records = [crate::binary::include::wifi_ap_record_t {
                bssid: [0u8; 6],
                ssid: [0u8; 33],
                primary: 0u8,
                second: 0u32,
                rssi: 0i8,
                authmode: 0u32,
                pairwise_cipher: 0u32,
                group_cipher: 0u32,
                ant: 0u32,
                _bitfield_align_1: [0u32; 0],
                _bitfield_1: crate::binary::include::__BindgenBitfieldUnit::new([0u8; 4usize]),
                country: crate::binary::include::wifi_country_t {
                    cc: [0; 3],
                    schan: 0u8,
                    nchan: 0u8,
                    max_tx_power: 0i8,
                    policy: 0u32,
                },
            }; N];

            crate::binary::include::esp_wifi_scan_get_ap_records(
                &mut bss_total,
                &mut records as *mut crate::binary::include::wifi_ap_record_t,
            );

            for i in 0..bss_total {
                let record = records[i as usize];
                let ssid_strbuf = crate::compat::common::StrBuf::from(&record.ssid as *const u8);

                let auth_method = match record.authmode {
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_OPEN => AuthMethod::None,
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WEP => AuthMethod::WEP,
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA_PSK => AuthMethod::WPA,
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA2_PSK => {
                        AuthMethod::WPA2Personal
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA_WPA2_PSK => {
                        AuthMethod::WPAWPA2Personal
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA2_ENTERPRISE => {
                        AuthMethod::WPA2Enterprise
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA3_PSK => {
                        AuthMethod::WPA3Personal
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WPA2_WPA3_PSK => {
                        AuthMethod::WPA2WPA3Personal
                    }
                    crate::binary::include::wifi_auth_mode_t_WIFI_AUTH_WAPI_PSK => {
                        AuthMethod::WAPIPersonal
                    }
                    _ => panic!(),
                };

                let mut ssid = heapless::String::<30>::new();
                ssid.push_str(ssid_strbuf.as_str_ref()).ok();

                let ap_info = AccessPointInfo {
                    ssid: ssid,
                    bssid: record.bssid,
                    channel: record.primary,
                    secondary_channel: match record.second {
                        crate::binary::include::wifi_second_chan_t_WIFI_SECOND_CHAN_NONE => {
                            SecondaryChannel::None
                        }
                        crate::binary::include::wifi_second_chan_t_WIFI_SECOND_CHAN_ABOVE => {
                            SecondaryChannel::Above
                        }
                        crate::binary::include::wifi_second_chan_t_WIFI_SECOND_CHAN_BELOW => {
                            SecondaryChannel::Below
                        }
                        _ => panic!(),
                    },
                    signal_strength: record.rssi.abs() as u8,
                    protocols: EnumSet::empty(), // TODO
                    auth_method: auth_method,
                };

                scanned.push(ap_info).ok();
            }
        }

        Ok((scanned, bss_total as usize))
    }

    /// Get the currently used configuration.
    fn get_configuration(&self) -> Result<embedded_svc::wifi::Configuration, Self::Error> {
        Ok(self.current_config.clone())
    }

    /// Set the configuration and start connecting.
    /// Currently only `ssid` and `password` is used. Trying anything but `Configuration::Client` will result in a panic!
    fn set_configuration(
        &mut self,
        conf: &embedded_svc::wifi::Configuration,
    ) -> Result<(), Self::Error> {
        self.current_config = conf.clone();

        let res = match conf {
            embedded_svc::wifi::Configuration::None => panic!(),
            embedded_svc::wifi::Configuration::Client(conf) => {
                crate::wifi::wifi_connect(&conf.ssid, &conf.password)
            }
            embedded_svc::wifi::Configuration::AccessPoint(_) => panic!(),
            embedded_svc::wifi::Configuration::Mixed(_, _) => panic!(),
        };

        if res != 0 {
            Err(WifiError::Unknown(res))
        } else {
            Ok(())
        }
    }
}

pub fn timestamp() -> Instant {
    Instant::from_millis(current_millis() as i64)
}
