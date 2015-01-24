
#![crate_name = "geoip"]
#![crate_type = "rlib"]

#![warn(non_camel_case_types,
        non_upper_case_globals,
        unused_qualifications)]
#![allow(unstable)]

extern crate libc;
extern crate "rustc-serialize" as rustc_serialize;
extern crate "geoip-sys" as geoip_sys;

use libc::{c_char, c_int, c_ulong};
use std::ffi;
use std::fmt;
use std::io::net::ip::{IpAddr,Ipv4Addr,Ipv6Addr};

#[cfg(test)]
use std::str::FromStr;

enum Charset {
    UTF8 = 1
}

pub enum Options {
    Standard = 0,
    MemoryCache = 1,
    CheckCache = 2,
    IndexCache = 4,
    MmapCache = 8
}

impl Copy for Options { }

pub enum DBType {
    CountryEdition = 1,
    RegionEditionRev0 = 7,
    CityEditionRev0 = 6,
    ORGEdition = 5,
    ISPEdition = 4,
    CityEditionRev1 = 2,
    RegionEditionRev1 = 3,
    ProxyEdition = 8,
    ASNUMEdition = 9,
    NetSpeedEdition = 10,
    DomainEdition = 11,
    CountryEditionV6 = 12,
    LocationAEdition = 13,
    AccuracyRadiusEdition = 14,
    LargeCountryEdition = 17,
    LargeCountryEditionV6 = 18,
    ASNumEditionV6 = 21,
    ISPEditionV6 = 22,
    ORGEditionV6 = 23,
    DomainEditionV6 = 24,
    LoctionAEditionV6 = 25,
    RegistrarEdition = 26,
    RegistrarEditionV6 = 27,
    UserTypeEdition = 28,
    UserTypeEditionV6 = 29,
    CityEditionRev1V6 = 30,
    CityEditionRev0V6 = 31,
    NetSpeedEditionRev1 = 32,
    NetSpeedEditionRev1V6 = 33,
    CountryConfEdition = 34,
    CityConfEdition = 35,
    RegionConfEdition = 36,
    PostalConfEdition = 37,
    AccuracyRadiusEditionV6 = 38
}

impl Copy for DBType { }

pub struct GeoIp {
    db: geoip_sys::RawGeoIp
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct ASInfo {
    pub asn: u32,
    pub name: String,
    pub netmask: u32
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct CityInfo {
    pub country_code: Option<String>,
    pub country_code3: Option<String>,
    pub country_name: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub latitude: f32,
    pub longitude: f32,
    pub dma_code: u32,
    pub area_code: u32,
    pub charset: u32,
    pub continent_code: Option<String>,
    pub netmask: u32
}

fn maybe_string(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        String::from_utf8(unsafe { ffi::c_str_to_bytes(&c_str) }.to_vec()).ok()
    }
}

impl CityInfo {
    unsafe fn from_geoiprecord(res: &geoip_sys::GeoIpRecord) -> CityInfo {
        CityInfo {
            country_code: maybe_string(res.country_code),
            country_code3: maybe_string(res.country_code3),
            country_name: maybe_string(res.country_name),
            region: maybe_string(res.region),
            city: maybe_string(res.city),
            postal_code: maybe_string(res.postal_code),
            latitude: res.latitude as f32,
            longitude: res.longitude as f32,
            dma_code: res.dma_code as u32,
            area_code: res.area_code as u32,
            charset: res.charset as u32,
            continent_code: maybe_string(res.continent_code),
            netmask: res.netmask as u32
        }
    }
}

impl fmt::Debug for ASInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.asn, self.name)
    }
}

enum CNetworkIp {
    V4(c_ulong),
    V6(geoip_sys::In6Addr)
}

impl CNetworkIp {
    fn new(ip: IpAddr) -> CNetworkIp {
        match ip {
            Ipv4Addr(a, b, c, d) => {
                CNetworkIp::V4(((a as c_ulong) << 24) | ((b as c_ulong) << 16) |
                               ((c as c_ulong) << 8)  | ((d as c_ulong)))
            },
            Ipv6Addr(a, b, c, d, e, f, g, h) => {
                CNetworkIp::V6([(a >> 8) as u8, a as u8,
                                (b >> 8) as u8, b as u8,
                                (c >> 8) as u8, c as u8,
                                (d >> 8) as u8, d as u8,
                                (e >> 8) as u8, e as u8,
                                (f >> 8) as u8, f as u8,
                                (g >> 8) as u8, g as u8,
                                (h >> 8) as u8, h as u8])
            }
        }
    }
}

impl GeoIp {
    pub fn open(path: &Path, options: Options) -> Result<GeoIp, String> {
        let file = match path.as_str() {
            None => return Err(format!("Invalid path {}", path.display())),
            Some(file) => file
        };
        let db = unsafe {
            geoip_sys::GeoIP_open(ffi::CString::from_slice(file.as_bytes()).as_ptr(),
                                  options as c_int)
        };
        if db.is_null() {
            return Err(format!("Can't open {}", file));
        }
        if unsafe { geoip_sys::GeoIP_set_charset(db, Charset::UTF8 as c_int)
        } != 0 {
            return Err("Can't set charset to UTF8".to_string());
        }
        Ok(GeoIp { db: db })
    }

    pub fn city_info_by_ip(&self, ip: IpAddr) -> Option<CityInfo> {
        let cres = match CNetworkIp::new(ip) {
            CNetworkIp::V4(ip) => unsafe {
                geoip_sys::GeoIP_record_by_ipnum(self.db, ip) },
            CNetworkIp::V6(ip) => unsafe {
                geoip_sys::GeoIP_record_by_ipnum_v6(self.db, ip) }
        };

        if cres.is_null() { return None; }

        unsafe {
            let city_info = CityInfo::from_geoiprecord(&*cres);
            geoip_sys::GeoIPRecord_delete(cres);
            std::mem::forget(cres);
            Some(city_info)
        }
    }

    pub fn as_info_by_ip(&self, ip: IpAddr) -> Option<ASInfo> {
        let mut gl = geoip_sys::GeoIpLookup::new();
        let cres = match CNetworkIp::new(ip) {
            CNetworkIp::V4(ip) => unsafe {
                geoip_sys::GeoIP_name_by_ipnum_gl(self.db, ip, &mut gl) },
            CNetworkIp::V6(ip) => unsafe {
                geoip_sys::GeoIP_name_by_ipnum_v6_gl(self.db, ip, &mut gl) }
        };

        if cres.is_null() {
            return None;
        }
        let description = match maybe_string(cres) {
            None => return None,
            Some(description) => description
        };
        let mut di = description.splitn(1, ' ');
        let asn = match di.next() {
            None => return None,
            Some(asn) => {
                if ! asn.starts_with("AS") {
                    return None
                } else {
                    asn[2..].parse::<u32>().unwrap()
                }
            }
        };
        let name = di.next().unwrap_or("(none)");
        let as_info = ASInfo {
            asn: asn,
            name: name.to_string(),
            netmask: gl.netmask as u32
        };
        Some(as_info)
    }
}

impl Drop for GeoIp {
    fn drop(&mut self) {
        unsafe {
            geoip_sys::GeoIP_delete(self.db);
        }
    }
}

#[test]
fn geoip_test_basic() {
    let geoip = match GeoIp::open(&Path::new("/opt/geoip/GeoIPASNum.dat"), Options::MemoryCache) {
        Err(err) => panic!(err),
        Ok(geoip) => geoip
    };
    let ip = FromStr::from_str("91.203.184.192").unwrap();
    let res = geoip.as_info_by_ip(ip).unwrap();
    assert!(res.asn == 41064);
    assert!(res.name.as_slice().contains("Telefun"));
    assert!(res.netmask == 22);
}

#[test]
fn geoip_test_city() {
    let geoip = match GeoIp::open(&Path::new("/opt/geoip/GeoLiteCity.dat"), Options::MemoryCache) {
        Err(err) => panic!(err),
        Ok(geoip) => geoip
    };
    let ip = FromStr::from_str("8.8.8.8").unwrap();
    let res = geoip.city_info_by_ip(ip).unwrap();
    assert!(res.city.unwrap().as_slice() == "Mountain View");
}
