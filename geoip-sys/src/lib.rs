
#![warn(non_camel_case_types,
        non_upper_case_globals,
        unused_qualifications)]

extern crate libc;

use libc::{c_void, c_char, c_int, c_ulong, c_float};

pub type RawGeoIp = *const c_void;
pub type In6Addr = [u8; 16];

#[repr(C)]
#[derive(Clone)]
pub struct GeoIpLookup {
    pub netmask: c_int
}

impl GeoIpLookup {
    pub fn new() -> GeoIpLookup {
        GeoIpLookup {
            netmask: 0
        }
    }
}

#[link(name = "GeoIP")]
extern {
    pub fn GeoIP_open(dbtype: *const c_char, flags: c_int) -> RawGeoIp;
    pub fn GeoIP_open_type(db_type: c_int, flags: c_int) -> RawGeoIp;
    pub fn GeoIP_delete(db: RawGeoIp);
    pub fn GeoIP_name_by_ipnum_gl(db: RawGeoIp, ipnum: c_ulong, gl: *mut GeoIpLookup) -> *const c_char;
    pub fn GeoIP_name_by_ipnum_v6_gl(db: RawGeoIp, ipnum: In6Addr, gl: *mut GeoIpLookup) -> *const c_char;
    pub fn GeoIP_record_by_ipnum(db: RawGeoIp, ipnum: c_ulong) -> *const GeoIpRecord;
    pub fn GeoIP_record_by_ipnum_v6(db: RawGeoIp, ipnum: In6Addr) -> *const GeoIpRecord;
    pub fn GeoIPRecord_delete(gir: *const GeoIpRecord);
    pub fn GeoIP_set_charset(db: RawGeoIp, charset: c_int) -> c_int;
    pub fn GeoIP_region_name_by_code(country_code: *const c_char, region_code: *const c_char) -> *const c_char;
    pub fn GeoIP_time_zone_by_country_and_region(country_code: *const c_char, region_code: *const c_char) -> *const c_char;
}

#[repr(C)]
#[derive(Clone)]
pub struct GeoIpRecord {
    pub country_code: *const c_char,
    pub country_code3: *const c_char,
    pub country_name: *const c_char,
    pub region: *const c_char,
    pub city: *const c_char,
    pub postal_code: *const c_char,
    pub latitude: c_float,
    pub longitude: c_float,
    pub dma_code: c_int,
    pub area_code: c_int,
    pub charset: c_int,
    pub continent_code: *const c_char,
    pub netmask: c_int
}
