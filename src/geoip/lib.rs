// (C)opyleft 2013 Frank Denis

/*!
 * Bindings for the GeoIP library
 */
#![desc = "Bindings for the GeoIP library."]
#![license = "BSD"]
#![crate_id = "geoip#0.1"]
#![crate_type = "rlib"]

#![warn(non_camel_case_types,
        non_uppercase_statics,
        unnecessary_qualification,
        managed_heap_memory)]

use std::c_str::CString;
use std::fmt;
use std::io::net::ip::{IpAddr,Ipv4Addr,Ipv6Addr};
use std::libc::{c_void, c_char, c_int, c_ulong};

type GeoIP_ = *c_void;
type In6Addr = [u8, ..16];

struct GeoIPLookup {
    netmask: c_int
}

impl GeoIPLookup {
    fn new() -> GeoIPLookup {
        GeoIPLookup {
            netmask: 0
        }
    }
}

#[link(name = "GeoIP")]
extern {
    fn GeoIP_open(dbtype: *c_char, flags: c_int) -> GeoIP_;
    fn GeoIP_delete(db: GeoIP_);
    fn GeoIP_name_by_ipnum_gl(db: GeoIP_, ipnum: c_ulong, gl: &GeoIPLookup) -> *c_char;
    fn GeoIP_name_by_ipnum_v6_gl(db: GeoIP_, ipnum: In6Addr, gl: &GeoIPLookup) -> *c_char;
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
    db: GeoIP_
}

pub struct ASInfo {
    pub asn: uint,
    pub name: ~str,
    pub netmask: uint
}

impl fmt::Show for ASInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "{}\t{}", self.asn, self.name)
    }
}

impl GeoIP {
    pub fn open(path: &Path, options: Options) -> Result<GeoIP, ~str> {
        let file = match path.as_str() {
            None => return Err(format!("Invalid path {}", path.display())),
            Some(file) => file
        };
        let db = unsafe {
            GeoIP_open(file.to_c_str().unwrap(), options as c_int)
        };
        match db.is_null() {
            true => Err(format!("Can't open {}", file)),
            false => Ok(GeoIP {
                    db: db
                })
        }
    }

    pub fn as_info_by_ip(&self, ip: IpAddr) -> Option<ASInfo> {
        let gl = GeoIPLookup::new();
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
        let mut di = description.splitn(' ', 1);
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
        let asinfo = ASInfo {
            asn: asn,
            name: name.to_owned(),
            netmask: gl.netmask as uint
        };
        Some(asinfo)
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
        Err(err) => fail!(err),
        Ok(geoip) => geoip
    };
    let ip = from_str("91.203.184.192").unwrap();
    let res = geoip.as_info_by_ip(ip).unwrap();
    assert!(res.asn == 41064);
    assert!(res.name.contains("Telefun"));
    assert!(res.netmask == 22);
}
