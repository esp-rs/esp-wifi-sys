use core::fmt::Display;

use embedded_nal::Ipv4Addr;
use embedded_svc::{
    ipv4::{ClientSettings, Mask, Subnet},
    wifi::{
        AccessPointInfo, ApStatus, AuthMethod, ClientConnectionStatus, ClientIpStatus,
        ClientStatus, SecondaryChannel, Status,
    },
};
use enumset::EnumSet;
use smoltcp_nal::NetworkStack;

use crate::wifi::{utils::WifiClock, WifiDevice};

extern crate alloc;

const MAX_SCAN_RESULT: u16 = 10;

/// An implementation of `embedded-svc`'s wifi trait.
pub struct Wifi<'a> {
    network_stack: NetworkStack<'a, WifiDevice, WifiClock>,
    current_config: embedded_svc::wifi::Configuration,
}

impl<'a> Wifi<'a> {
    /// Create a new instance from a `NetworkStack`
    pub fn new(network_stack: NetworkStack<'a, WifiDevice, WifiClock>) -> Wifi<'a> {
        Wifi {
            network_stack,
            current_config: embedded_svc::wifi::Configuration::default(),
        }
    }

    /// Get a mutable reference to the `NetworkStack`
    pub fn network_stack(&mut self) -> &mut NetworkStack<'a, WifiDevice, WifiClock> {
        &mut self.network_stack
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WifiError {
    Unknown(i32),
}

impl Display for WifiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> embedded_svc::errors::Errors for Wifi<'a> {
    type Error = WifiError;
}

impl<'a> embedded_svc::wifi::Wifi for Wifi<'a> {
    /// This currently only supports the `Client` capability.
    fn get_capabilities(&self) -> Result<EnumSet<embedded_svc::wifi::Capability>, Self::Error> {
        // for now we only support STA mode
        let mut caps = EnumSet::empty();
        caps.insert(embedded_svc::wifi::Capability::Client);
        Ok(caps)
    }

    /// Get the wifi status.
    /// Please note: To ever get into the state of an assigned IP address you need to make sure
    /// that `poll` is called frequently on the network stack.
    /// Subnet and DNS - while present under the hood - is unsupported for now.
    fn get_status(&self) -> Status {
        match crate::wifi::get_wifi_state() {
            crate::wifi::WifiState::WifiReady => Status(ClientStatus::Stopped, ApStatus::Stopped),
            crate::wifi::WifiState::StaStart => Status(ClientStatus::Starting, ApStatus::Stopped),
            crate::wifi::WifiState::StaStop => Status(ClientStatus::Stopped, ApStatus::Stopped),
            crate::wifi::WifiState::StaConnected => {
                let client_ip_status = if let Some(ip) = self.network_stack.interface().ipv4_addr()
                {
                    if !ip.is_unspecified() {
                        let mut ip_bytes: [u8; 4] = [0; 4];
                        ip_bytes.copy_from_slice(ip.as_bytes());

                        // TODO how to get gateway / mask and nameservers here?
                        ClientIpStatus::Done(ClientSettings {
                            ip: Ipv4Addr::from(ip_bytes),
                            subnet: Subnet {
                                gateway: Ipv4Addr::new(0, 0, 0, 0),
                                mask: Mask(24),
                            },
                            dns: Some(Ipv4Addr::new(0, 0, 0, 0)),
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
    fn scan(&mut self) -> Result<alloc::vec::Vec<AccessPointInfo>, Self::Error> {
        crate::wifi::wifi_start_scan();

        let mut scanned = alloc::vec::Vec::new();

        unsafe {
            let mut bss_total: u16 = 0;
            crate::binary::include::esp_wifi_scan_get_ap_num(&mut bss_total);
            if bss_total > MAX_SCAN_RESULT {
                bss_total = MAX_SCAN_RESULT;
            }

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
            }; MAX_SCAN_RESULT as usize];

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

                let mut ssid = alloc::string::String::new();
                ssid.push_str(ssid_strbuf.as_str_ref());

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

                scanned.push(ap_info);
            }
        }

        Ok(scanned)
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
