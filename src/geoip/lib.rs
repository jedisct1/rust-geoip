#![crate_name = "geoip"]
#![crate_type = "rlib"]
#![warn(non_camel_case_types, non_upper_case_globals, unused_qualifications)]

use geoip_sys;
#[macro_use]
extern crate lazy_static;
use libc;

use libc::{c_char, c_int, c_ulong, c_void};
use std::error::Error;
use std::ffi;
use std::fmt::{self, Debug};
use std::net::IpAddr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::str::Utf8Error;
use std::sync::Mutex;

lazy_static! {
    static ref LOCK: Mutex<()> = Mutex::new(());
}

#[derive(Debug, Clone)]
pub enum Charset {
    Utf8 = 1,
}

#[derive(Debug, Clone)]
pub enum Options {
    Standard = 0,
    MemoryCache = 1,
    CheckCache = 2,
    IndexCache = 4,
    MmapCache = 8,
}

#[derive(Debug, Clone)]
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
    AccuracyRadiusEditionV6 = 38,
}

pub struct GeoIp {
    db: geoip_sys::RawGeoIp,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct ASInfo {
    pub asn: u32,
    pub name: String,
    pub netmask: u32,
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct CityInfo {
    pub country_code: Option<String>,
    pub country_code3: Option<String>,
    pub country_name: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub latitude: f32,
    pub longitude: f32,
    pub dma_code: Option<u32>,
    pub area_code: Option<u32>,
    pub continent_code: Option<String>,
    pub netmask: u32,
}

fn maybe_string(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        String::from_utf8(unsafe { ffi::CStr::from_ptr(c_str).to_bytes() }.to_vec()).ok()
    }
}

fn maybe_code(code: u32) -> Option<u32> {
    if code == 0 {
        None
    } else {
        Some(code)
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
            latitude: res.latitude,
            longitude: res.longitude,
            dma_code: maybe_code(res.dma_code as u32),
            area_code: maybe_code(res.area_code as u32),
            continent_code: maybe_string(res.continent_code),
            netmask: res.netmask as u32,
        }
    }
}

enum CNetworkIp {
    V4(c_ulong),
    V6(geoip_sys::In6Addr),
}

impl CNetworkIp {
    fn new(ip: IpAddr) -> CNetworkIp {
        match ip {
            IpAddr::V4(addr) => {
                let b = addr.octets();
                CNetworkIp::V4(
                    ((b[0] as c_ulong) << 24)
                        | ((b[1] as c_ulong) << 16)
                        | ((b[2] as c_ulong) << 8)
                        | (b[3] as c_ulong),
                )
            }
            IpAddr::V6(addr) => {
                let b = addr.segments();
                CNetworkIp::V6([
                    (b[0] >> 8) as u8,
                    b[0] as u8,
                    (b[1] >> 8) as u8,
                    b[1] as u8,
                    (b[2] >> 8) as u8,
                    b[2] as u8,
                    (b[3] >> 8) as u8,
                    b[3] as u8,
                    (b[4] >> 8) as u8,
                    b[4] as u8,
                    (b[5] >> 8) as u8,
                    b[5] as u8,
                    (b[6] >> 8) as u8,
                    b[6] as u8,
                    (b[7] >> 8) as u8,
                    b[7] as u8,
                ])
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum OpenPathError {
    InvalidPath(ffi::NulError),
    OpenFailed(PathBuf),
    SetCharsetFailed(Charset),
}

impl From<ffi::NulError> for OpenPathError {
    fn from(err: ffi::NulError) -> Self {
        OpenPathError::InvalidPath(err)
    }
}

impl fmt::Display for OpenPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OpenPathError::InvalidPath(ref e) => write!(f, "Given path was invalid: {}", e),
            OpenPathError::OpenFailed(ref path) => {
                write!(f, "Failed to open database from path '{}'", path.display())
            }
            OpenPathError::SetCharsetFailed(ref charset) => {
                write!(f, "Failed to set database charset {:?}", charset)
            }
        }
    }
}

impl Error for OpenPathError {
    fn description(&self) -> &str {
        match *self {
            OpenPathError::InvalidPath(_) => "invalid database path",
            OpenPathError::OpenFailed(_) => "failed to open database from path",
            OpenPathError::SetCharsetFailed(_) => "failed to set database charset",
        }
    }
}

#[derive(Debug, Clone)]
pub enum OpenTypeError {
    OpenFailed(DBType),
    SetCharsetFailed(Charset),
}

impl fmt::Display for OpenTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OpenTypeError::OpenFailed(ref t) => {
                write!(f, "Failed to open database of type {:?}", t)
            }
            OpenTypeError::SetCharsetFailed(ref charset) => {
                write!(f, "Failed to set database charset {:?}", charset)
            }
        }
    }
}

impl Error for OpenTypeError {
    fn description(&self) -> &str {
        match *self {
            OpenTypeError::OpenFailed(_) => "failed to open database of type",
            OpenTypeError::SetCharsetFailed(_) => "failed to set database charset",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReadInfoError {
    InfoFailed,
    InvalidData(Utf8Error),
}

impl From<Utf8Error> for ReadInfoError {
    fn from(err: Utf8Error) -> Self {
        ReadInfoError::InvalidData(err)
    }
}

impl fmt::Display for ReadInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ReadInfoError::InfoFailed => write!(f, "Failed to get database info"),
            ReadInfoError::InvalidData(ref err) => write!(f, "Invalid info data: {}", err),
        }
    }
}

impl Error for ReadInfoError {
    fn description(&self) -> &str {
        match *self {
            ReadInfoError::InfoFailed => "failed to get database info",
            ReadInfoError::InvalidData(_) => "invalid info data",
        }
    }
}

impl GeoIp {
    pub fn open(path: &Path, options: Options) -> Result<GeoIp, OpenPathError> {
        let db = unsafe {
            geoip_sys::GeoIP_open(
                ffi::CString::new(path.as_os_str().as_bytes())?.as_ptr(),
                options as c_int,
            )
        };
        if db.is_null() {
            return Err(OpenPathError::OpenFailed(path.to_owned()));
        }
        if unsafe { geoip_sys::GeoIP_set_charset(db, Charset::Utf8 as c_int) } != 0 {
            return Err(OpenPathError::SetCharsetFailed(Charset::Utf8));
        }
        Ok(GeoIp { db })
    }

    pub fn open_type(db_type: DBType, options: Options) -> Result<GeoIp, OpenTypeError> {
        let db = unsafe {
            // GeoIP_open_type initialises global state causing races
            let _lock = LOCK.lock().unwrap();
            geoip_sys::GeoIP_open_type(db_type.clone() as c_int, options as c_int)
        };
        if db.is_null() {
            return Err(OpenTypeError::OpenFailed(db_type));
        }
        if unsafe { geoip_sys::GeoIP_set_charset(db, Charset::Utf8 as c_int) } != 0 {
            return Err(OpenTypeError::SetCharsetFailed(Charset::Utf8));
        }
        Ok(GeoIp { db })
    }

    pub fn info(&self) -> Result<String, ReadInfoError> {
        let c_string = unsafe { geoip_sys::GeoIP_database_info(self.db) };

        if c_string.is_null() {
            Err(ReadInfoError::InfoFailed)
        } else {
            match unsafe { ffi::CStr::from_ptr(c_string) }.to_str() {
                Ok(str) => {
                    let ret = str.to_string();
                    unsafe { libc::free(c_string as *mut c_void) };
                    Ok(ret)
                }
                Err(err) => {
                    unsafe { libc::free(c_string as *mut c_void) };
                    Err(From::from(err))
                }
            }
        }
    }

    pub fn city_info_by_ip(&self, ip: IpAddr) -> Option<CityInfo> {
        let cres = match CNetworkIp::new(ip) {
            CNetworkIp::V4(ip) => unsafe { geoip_sys::GeoIP_record_by_ipnum(self.db, ip) },
            CNetworkIp::V6(ip) => unsafe { geoip_sys::GeoIP_record_by_ipnum_v6(self.db, ip) },
        };

        if cres.is_null() {
            return None;
        }

        unsafe {
            let city_info = CityInfo::from_geoiprecord(&*cres);
            geoip_sys::GeoIPRecord_delete(cres);
            std::mem::forget(cres);
            Some(city_info)
        }
    }

    pub fn city_info_by_name(&self, name: &str) -> Option<CityInfo> {
        let name = name.as_ptr() as *const _;
        let cres_v4 = unsafe { geoip_sys::GeoIP_record_by_name(self.db, name) };
        let cres_v6 = unsafe { geoip_sys::GeoIP_record_by_name_v6(self.db, name) };

        let cres = if cres_v6.is_null() { cres_v4 } else { cres_v6 };

        if cres.is_null() {
            return None;
        }

        unsafe {
            let city_info = CityInfo::from_geoiprecord(&*cres);
            geoip_sys::GeoIPRecord_delete(cres);
            std::mem::forget(cres);
            Some(city_info)
        }
    }

    pub fn region_name_by_code(country_code: &str, region_code: &str) -> Option<&'static str> {
        unsafe {
            let cstr = geoip_sys::GeoIP_region_name_by_code(
                ffi::CString::new(country_code).unwrap().as_ptr(),
                ffi::CString::new(region_code).unwrap().as_ptr(),
            );

            if cstr.is_null() {
                return None;
            }

            Some(
                ffi::CStr::from_ptr(cstr)
                    .to_str()
                    .expect("invalid region name data"),
            )
        }
    }

    pub fn time_zone_by_country_and_region(
        country_code: &str,
        region_code: &str,
    ) -> Option<&'static str> {
        unsafe {
            let cstr = geoip_sys::GeoIP_time_zone_by_country_and_region(
                ffi::CString::new(country_code).unwrap().as_ptr(),
                ffi::CString::new(region_code).unwrap().as_ptr(),
            );

            if cstr.is_null() {
                return None;
            }

            Some(
                ffi::CStr::from_ptr(cstr)
                    .to_str()
                    .expect("invalid time zone data"),
            )
        }
    }

    pub fn as_info_by_ip(&self, ip: IpAddr) -> Option<ASInfo> {
        let mut gl = geoip_sys::GeoIpLookup::new();
        let cres = match CNetworkIp::new(ip) {
            CNetworkIp::V4(ip) => unsafe {
                geoip_sys::GeoIP_name_by_ipnum_gl(self.db, ip, &mut gl)
            },
            CNetworkIp::V6(ip) => unsafe {
                geoip_sys::GeoIP_name_by_ipnum_v6_gl(self.db, ip, &mut gl)
            },
        };

        if cres.is_null() {
            return None;
        }
        let description = match maybe_string(cres) {
            None => return None,
            Some(description) => description,
        };
        let mut di = description.splitn(2, ' ');
        let asn = match di.next() {
            None => return None,
            Some(asn) => {
                if !asn.starts_with("AS") {
                    return None;
                } else {
                    asn[2..]
                        .splitn(2, ' ')
                        .next()
                        .unwrap()
                        .parse::<u32>()
                        .unwrap()
                }
            }
        };
        let name = di.next().unwrap_or("(none)");
        let as_info = ASInfo {
            asn,
            name: name.to_string(),
            netmask: gl.netmask as u32,
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

impl Debug for GeoIp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeoIp").field("info", &self.info()).finish()
    }
}

#[test]
fn geoip_test_basic() {
    let geoip = GeoIp::open(
        &Path::new("/opt/geoip/GeoIPASNum.dat"),
        Options::MemoryCache,
    )
    .unwrap();

    let ip = IpAddr::V4("8.8.8.8".parse().unwrap());
    let res = geoip.as_info_by_ip(ip).unwrap();
    assert_eq!(res.asn, 15169);
    assert_eq!(res.name, "Google Inc.".to_string());
    assert_eq!(res.netmask, 24);
}

#[test]
fn geoip_test_city() {
    let geoip = GeoIp::open(
        &Path::new("/opt/geoip/GeoLiteCity.dat"),
        Options::MemoryCache,
    )
    .unwrap();

    let ip = IpAddr::V4("8.8.8.8".parse().unwrap());
    let res = geoip.city_info_by_ip(ip).unwrap();
    assert_eq!(res.city, Some("Mountain View".to_string()));
}

#[test]
fn geoip_test_city_open_fail() {
    let geoip = GeoIp::open(&Path::new("foobar.baz"), Options::MemoryCache);

    assert_eq!(
        "Failed to open database from path 'foobar.baz'",
        &format!("{}", geoip.unwrap_err())
    );
}

#[test]
fn geoip_test_city_maybe_code() {
    let geoip = GeoIp::open(
        &Path::new("/opt/geoip/GeoLiteCity.dat"),
        Options::MemoryCache,
    )
    .unwrap();

    let ip = IpAddr::V4("8.8.8.8".parse().unwrap());
    let res = geoip.city_info_by_ip(ip).unwrap();
    assert!(res.city.is_some());
    assert_eq!(res.dma_code, Some(807));
    assert_eq!(res.area_code, Some(650));

    let ip = IpAddr::V4("95.144.124.132".parse().unwrap());
    let res = geoip.city_info_by_ip(ip).unwrap();
    assert!(res.city.is_some());
    assert!(res.dma_code.is_none());
    assert!(res.area_code.is_none());
}

#[test]
fn geoip_test_city_type() {
    let geoip = GeoIp::open_type(DBType::CityEditionRev1, Options::MemoryCache).unwrap();
    let ip = IpAddr::V4("8.8.8.8".parse().unwrap());
    let res = geoip.city_info_by_ip(ip).unwrap();
    assert!(res.city.unwrap() == "Mountain View");
}

#[test]
fn geoip_test_info() {
    let geoip = GeoIp::open_type(DBType::CityEditionRev1, Options::MemoryCache).unwrap();
    assert!(geoip.info().unwrap().contains("GEO-133"));
}

#[test]
fn geoip_region_name_by_code() {
    assert_eq!(GeoIp::region_name_by_code("foo", "bar"), None);
    assert_eq!(GeoIp::region_name_by_code("US", "CA"), Some("California"));
}

#[test]
fn geoip_time_zone_by_country_and_region() {
    assert_eq!(GeoIp::time_zone_by_country_and_region("foo", "bar"), None);
    assert_eq!(
        GeoIp::time_zone_by_country_and_region("US", "CA"),
        Some("America/Los_Angeles")
    );
}

#[test]
fn geoip_test_city_type_race() {
    use std::sync::{Arc, Barrier};
    use std::thread;
    const N: usize = 20;

    let barrier = Arc::new(Barrier::new(N));

    (0..N)
        .map(|_| {
            let c = barrier.clone();
            thread::spawn(move || {
                // hopefully this will exercise a race condition
                c.wait();
                let geoip =
                    GeoIp::open_type(DBType::CityEditionRev1, Options::MemoryCache).unwrap();
                let ip = IpAddr::V4("8.8.8.8".parse().unwrap());
                let res = geoip.city_info_by_ip(ip).unwrap();
                assert_eq!(res.city.as_ref().map(String::as_str), Some("Mountain View"));
            })
        })
        .collect::<Vec<_>>() // spawn all treads
        .into_iter()
        .map(|t| t.join()) // wait for treads to finish and get their results
        .collect::<Result<Vec<_>, _>>() // will be Err(Any) if one of the Result was Err
        .map_err(|any| any.downcast_ref::<String>().unwrap().to_owned())
        .expect("one of the threads failed");
}
