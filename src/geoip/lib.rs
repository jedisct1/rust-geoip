// (C)opyleft 2013,2014 Frank Denis

/*!
 * Bindings for the GeoIP library
 */
#![desc = "Bindings for the GeoIP library."]
#![license = "BSD"]
#![crate_name = "geoip"]
#![crate_type = "rlib"]

#![warn(non_camel_case_types,
        non_upper_case_globals,
        unused_qualifications)]

extern crate libc;
extern crate serialize;

use libc::{c_void, c_char, c_int, c_ulong, c_float};
use std::c_str::CString;
use std::fmt;
use std::io::net::ip::{IpAddr,Ipv4Addr,Ipv6Addr};

type RawGeoIp = *const c_void;
type In6Addr = [u8, ..16];

#[repr(C)]
struct GeoIpLookup {
    netmask: c_int
}

impl GeoIpLookup {
    fn new() -> GeoIpLookup {
        GeoIpLookup {
            netmask: 0
        }
    }
}

#[link(name = "GeoIP")]
extern {
    fn GeoIP_open(dbtype: *const c_char, flags: c_int) -> RawGeoIp;
    fn GeoIP_delete(db: RawGeoIp);
    fn GeoIP_name_by_ipnum_gl(db: RawGeoIp, ipnum: c_ulong, gl: &GeoIpLookup) -> *const c_char;
    fn GeoIP_name_by_ipnum_v6_gl(db: RawGeoIp, ipnum: In6Addr, gl: &GeoIpLookup) -> *const c_char;
    fn GeoIP_record_by_ipnum(db: RawGeoIp, ipnum: c_ulong) -> *const GeoIPRecord_;
    fn GeoIP_record_by_ipnum_v6(db: RawGeoIp, ipnum: In6Addr) -> *const GeoIPRecord_;
    fn GeoIPRecord_delete(gir: *const GeoIPRecord_);
    fn GeoIP_set_charset(db: RawGeoIp, charset: c_int) -> c_int;
}

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

pub struct GeoIP {
    db: RawGeoIp
}

#[repr(C)]
pub struct GeoIPRecord_ {
    country_code: *const c_char,
    country_code3: *const c_char,
    country_name: *const c_char,
    region: *const c_char,
    city: *const c_char,
    postal_code: *const c_char,
    latitude: c_float,
    longitude: c_float,
    dma_code: c_int,
    area_code: c_int,
    charset: c_int,
    continent_code: *const c_char,
    netmask: c_int
}

#[deriving(Decodable, Encodable)]
pub struct ASInfo {
    pub asn: uint,
    pub name: String,
    pub netmask: uint
}

#[deriving(Decodable, Encodable)]
pub struct CityInfo {
    pub country_code: Option<String>,
    pub country_code3: Option<String>,
    pub country_name: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub latitude: f32,
    pub longitude: f32,
    pub dma_code: uint,
    pub area_code: uint,
    pub charset: uint,
    pub continent_code: Option<String>,
    pub netmask: uint
}

unsafe fn maybe_string(c_str: *const c_char) -> Option<String> {
    c_str.as_ref().and_then(|opt| {
        CString::new(opt, false).as_str().map(|s| s.to_string())
    })
}

impl CityInfo {
    unsafe fn from_geoiprecord(res: &GeoIPRecord_) -> CityInfo {
        CityInfo {
            country_code: maybe_string(res.country_code),
            country_code3: maybe_string(res.country_code3),
            country_name: maybe_string(res.country_name),
            region: maybe_string(res.region),
            city: maybe_string(res.city),
            postal_code: maybe_string(res.postal_code),
            latitude: res.latitude as f32,
            longitude: res.longitude as f32,
            dma_code: res.dma_code as uint,
            area_code: res.area_code as uint,
            charset: res.charset as uint,
            continent_code: maybe_string(res.continent_code),
            netmask: res.netmask as uint
        }
    }
}

impl fmt::Show for ASInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.asn, self.name)
    }
}

impl GeoIP {
    pub fn open(path: &Path, options: Options) -> Result<GeoIP, String> {
        let file = match path.as_str() {
            None => return Err(format!("Invalid path {}", path.display())),
            Some(file) => file
        };
        let db = unsafe {
            GeoIP_open(file.to_c_str().unwrap(), options as c_int)
        };
        if db.is_null() {
            return Err(format!("Can't open {}", file));
        }
        if unsafe { GeoIP_set_charset(db, UTF8 as c_int) } != 0 {
            return Err("Can't set charset to UTF8".to_string());
        }
        Ok(GeoIP { db: db })
    }

    pub fn city_info_by_ip(&self, ip: IpAddr) -> Option<CityInfo> {
        let cres = match ip {
            Ipv4Addr(a, b, c, d) => {
                let ipnum: c_ulong =
                    (a as c_ulong << 24) | (b as c_ulong << 16) |
                    (c as c_ulong << 8)  | (d as c_ulong);
                unsafe {
                    GeoIP_record_by_ipnum(self.db, ipnum)
                }
            },
            Ipv6Addr(a, b, c, d, e, f, g, h) => {
                let in6_addr: In6Addr = [(a >> 8) as u8, a as u8,
                                         (b >> 8) as u8, b as u8,
                                         (c >> 8) as u8, c as u8,
                                         (d >> 8) as u8, d as u8,
                                         (e >> 8) as u8, e as u8,
                                         (f >> 8) as u8, f as u8,
                                         (g >> 8) as u8, g as u8,
                                         (h >> 8) as u8, h as u8];
                unsafe {
                    GeoIP_record_by_ipnum_v6(self.db, in6_addr)
                }
            }
        };

        if cres.is_null() { return None; }

        unsafe {
            let city_info = CityInfo::from_geoiprecord(&*cres);
            GeoIPRecord_delete(cres);
            std::mem::forget(cres);
            Some(city_info)
        }
    }

    pub fn as_info_by_ip(&self, ip: IpAddr) -> Option<ASInfo> {
        let gl = GeoIpLookup::new();
        let cres = match ip {
            Ipv4Addr(a, b, c, d) => {
                let ipnum: c_ulong =
                    (a as c_ulong << 24) | (b as c_ulong << 16) |
                    (c as c_ulong << 8)  | (d as c_ulong);
                unsafe {
                    GeoIP_name_by_ipnum_gl(self.db, ipnum, &gl)
                }
            },
            Ipv6Addr(a, b, c, d, e, f, g, h) => {
                let in6_addr: In6Addr = [(a >> 8) as u8, a as u8,
                                         (b >> 8) as u8, b as u8,
                                         (c >> 8) as u8, c as u8,
                                         (d >> 8) as u8, d as u8,
                                         (e >> 8) as u8, e as u8,
                                         (f >> 8) as u8, f as u8,
                                         (g >> 8) as u8, g as u8,
                                         (h >> 8) as u8, h as u8];
                unsafe {
                    GeoIP_name_by_ipnum_v6_gl(self.db, in6_addr, &gl)
                }
            }
        };
        if cres.is_null() {
            return None;
        }
        let description_cstr = unsafe { CString::new(cres, true) };
        let description = match description_cstr.as_str() {
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
                    from_str::<uint>(asn.slice_from(2)).unwrap()
                }
            }
        };
        let name = di.next().unwrap_or("(none)");
        let as_info = ASInfo {
            asn: asn,
            name: name.to_string(),
            netmask: gl.netmask as uint
        };
        Some(as_info)
    }
}

impl Drop for GeoIP {
    fn drop(&mut self) {
        unsafe {
            GeoIP_delete(self.db);
        }
    }
}

#[test]
fn geoip_test_basic() {
    let geoip = match GeoIP::open(&from_str("/opt/geoip/GeoIPASNum.dat").unwrap(), MemoryCache) {
        Err(err) => panic!(err),
        Ok(geoip) => geoip
    };
    let ip = from_str("91.203.184.192").unwrap();
    let res = geoip.as_info_by_ip(ip).unwrap();
    assert!(res.asn == 41064);
    assert!(res.name.as_slice().contains("Telefun"));
    assert!(res.netmask == 22);
}

#[test]
fn geoip_test_city() {
    let geoip = match GeoIP::open(&from_str("/opt/geoip/GeoLiteCity.dat").unwrap(), MemoryCache) {
        Err(err) => panic!(err),
        Ok(geoip) => geoip
    };
    let ip = from_str("8.8.8.8").unwrap();
    let res = geoip.city_info_by_ip(ip).unwrap();
    assert!(res.city.unwrap().as_slice() == "Mountain View");
}
