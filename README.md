rust-geoip
==========

[GeoIP](http://www.maxmind.com/en/geolocation_landing) bindings for Rust.

Currently only supports:
* [Autonomous System Numbers Legacy Database](http://dev.maxmind.com/geoip/legacy/geolite/#Autonomous_System_Numbers)
* [GeoIP Legacy City Database](http://dev.maxmind.com/geoip/legacy/install/city/)

Installation: use [Cargo](http://crates.io).

Tested with GeoIP v1.6.9.

Usage:
------

City Database:
```rust
// Open by DBType
let geoip = GeoIp::open_type(DBType::CityEditionRev1, Options::MemoryCache).unwrap();
// Open by Path
let geoip = GeoIp::open(&Path::new("/opt/geoip/GeoLiteCity.dat"),
						Options::MemoryCache)
	.unwrap();
/*
GeoIp {
	info: Ok(
		"GEO-133 20160621 Build 1 Copyright (c) 2016 MaxMind Inc All Rights Re"
	)
}
*/

// Query by IP
let ip = IpAddr::V4("8.8.8.8".parse().unwrap());
let res = geoip.city_info_by_ip(ip).unwrap();
/*
CityInfo {
	country_code: Some(
		"US"
	),
	country_code3: Some(
		"USA"
	),
	country_name: Some(
		"United States"
	),
	region: Some(
		"CA"
	),
	city: Some(
		"Mountain View"
	),
	postal_code: Some(
		"94035"
	),
	latitude: 37.386,
	longitude: -122.0838,
	dma_code: Some(
		807
	),
	area_code: Some(
		650
	),
	continent_code: Some(
		"NA"
	),
	netmask: 24
}
*/

// Get additional information (as compiled in the C library)
let region_name = GeoIp::region_name_by_code("US", "CA");
// Some("California")

// Get time zone inforamtion (as compiled in the C library)
let time_zone = GeoIp::time_zone_by_country_and_region("US", "CA");
// Some("America/Los_Angeles")
```


AS Database:

```rust
// Open by Path
let geoip = GeoIp::open(&Path::new("/opt/geoip/GeoIPASNum.dat"),
						Options::MemoryCache)
	.unwrap();
/*
GeoIp {
	info: Ok(
		"GEO-117 20160627 Build 1 Copyright (c) 2016 MaxMind Inc All Rights Re"
	)
}
*/

// Query by IP
let ip = IpAddr::V4("8.8.8.8".parse().unwrap());
let res = geoip.as_info_by_ip(ip).unwrap();
/*
ASInfo {
	asn: 15169,
	name: "Google Inc.",
	netmask: 24
}
*/
```
