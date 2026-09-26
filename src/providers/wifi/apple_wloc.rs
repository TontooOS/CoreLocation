use crate::types::LocationError;
use std::time::Duration;

const WLOC_URL: &str = "https://gs-loc.apple.com/clls/wloc";
const ARPC_VERSION: u16 = 1;
const ARPC_LOCALE: &str = "en-001_001";
const ARPC_APP_IDENTIFIER: &str = "com.apple.locationd";
const ARPC_OS_VERSION: &str = "18.6.2.22G100";
const ARPC_FUNCTION_ID: u32 = 1;
const DEVICE_OS: &str = "iPhone OS17.5/21F79";
const DEVICE_MODEL: &str = "iPhone12,1";
const MAX_RESULTS: i32 = 400;
const RESPONSE_HEADER_LEN: usize = 10;

#[derive(Debug, Clone)]
pub struct WifiApResult {
    pub bssid: String,
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_m: Option<f64>,
}

fn encode_varint(buf: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            buf.push(byte);
            break;
        }
        buf.push(byte | 0x80);
    }
}

fn encode_tag(buf: &mut Vec<u8>, field: u32, wire_type: u32) {
    encode_varint(buf, ((field << 3) | wire_type) as u64);
}

fn zigzag32(n: i32) -> u64 {
    ((n << 1) ^ (n >> 31)) as u32 as u64
}

fn encode_string_field(buf: &mut Vec<u8>, field: u32, s: &str) {
    encode_tag(buf, field, 2);
    encode_varint(buf, s.len() as u64);
    buf.extend_from_slice(s.as_bytes());
}

fn encode_message_field(buf: &mut Vec<u8>, field: u32, msg: &[u8]) {
    encode_tag(buf, field, 2);
    encode_varint(buf, msg.len() as u64);
    buf.extend_from_slice(msg);
}

fn encode_sint32_field(buf: &mut Vec<u8>, field: u32, n: i32) {
    encode_tag(buf, field, 0);
    encode_varint(buf, zigzag32(n));
}

fn push_pascal_string(buf: &mut Vec<u8>, s: &str) {
    buf.extend_from_slice(&(s.len() as u16).to_be_bytes());
    buf.extend_from_slice(s.as_bytes());
}

fn build_request(bssids: &[String]) -> Vec<u8> {
    let mut block = Vec::new();

    for bssid in bssids {
        let mut device = Vec::new();
        encode_string_field(&mut device, 1, bssid);
        encode_message_field(&mut block, 2, &device);
    }

    encode_sint32_field(&mut block, 3, 0);
    encode_sint32_field(&mut block, 4, MAX_RESULTS);

    let mut device_type = Vec::new();
    encode_string_field(&mut device_type, 1, DEVICE_OS);
    encode_string_field(&mut device_type, 2, DEVICE_MODEL);
    encode_message_field(&mut block, 33, &device_type);

    let mut body = Vec::new();
    body.extend_from_slice(&ARPC_VERSION.to_be_bytes());
    push_pascal_string(&mut body, ARPC_LOCALE);
    push_pascal_string(&mut body, ARPC_APP_IDENTIFIER);
    push_pascal_string(&mut body, ARPC_OS_VERSION);
    body.extend_from_slice(&ARPC_FUNCTION_ID.to_be_bytes());
    body.extend_from_slice(&(block.len() as u32).to_be_bytes());
    body.extend_from_slice(&block);

    body
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_varint(&mut self) -> Option<u64> {
        let mut result = 0u64;
        let mut shift = 0;
        loop {
            let byte = *self.data.get(self.pos)?;
            self.pos += 1;
            result |= ((byte & 0x7f) as u64) << shift;
            if byte & 0x80 == 0 {
                return Some(result);
            }
            shift += 7;
            if shift > 63 {
                return None;
            }
        }
    }

    fn read_len_delimited(&mut self) -> Option<&'a [u8]> {
        let len = self.read_varint()? as usize;
        let end = self.pos.checked_add(len)?;
        let slice = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }
}

fn parse_location(data: &[u8]) -> (Option<i64>, Option<i64>, Option<f64>) {
    let mut reader = Reader::new(data);
    let mut lat = None;
    let mut lon = None;
    let mut accuracy = None;

    while reader.pos < reader.data.len() {
        let tag = match reader.read_varint() {
            Some(t) => t,
            None => break,
        };
        let field = (tag >> 3) as u32;
        let wire_type = (tag & 0x07) as u32;

        match (field, wire_type) {
            (1, 0) => lat = reader.read_varint().map(|v| v as i64),
            (2, 0) => lon = reader.read_varint().map(|v| v as i64),
            (3, 0) => accuracy = reader.read_varint().map(|v| v as f64),
            (_, 0) => {
                reader.read_varint();
            }
            (_, 2) => {
                reader.read_len_delimited();
            }
            _ => break,
        }
    }

    (lat, lon, accuracy)
}

fn parse_wifi_device(data: &[u8]) -> Option<WifiApResult> {
    let mut reader = Reader::new(data);
    let mut bssid = String::new();
    let mut lat = None;
    let mut lon = None;
    let mut accuracy = None;

    while reader.pos < reader.data.len() {
        let tag = match reader.read_varint() {
            Some(t) => t,
            None => break,
        };
        let field = (tag >> 3) as u32;
        let wire_type = (tag & 0x07) as u32;

        match (field, wire_type) {
            (1, 2) => {
                let bytes = reader.read_len_delimited()?;
                bssid = String::from_utf8_lossy(bytes).to_string();
            }
            (2, 2) => {
                let loc = reader.read_len_delimited()?;
                let (l, o, a) = parse_location(loc);
                lat = l;
                lon = o;
                accuracy = a;
            }
            (_, 0) => {
                reader.read_varint();
            }
            (_, 2) => {
                reader.read_len_delimited();
            }
            _ => break,
        }
    }

    let lat = lat?;
    let lon = lon?;
    let latitude = lat as f64 * 1e-8;
    let longitude = lon as f64 * 1e-8;

    if latitude <= -180.0 && longitude <= -180.0 {
        return None;
    }

    let accuracy_m = accuracy.filter(|a| *a > 0.0 && *a < 10_000.0);

    Some(WifiApResult {
        bssid,
        latitude,
        longitude,
        accuracy_m,
    })
}

fn parse_response(data: &[u8]) -> Result<Vec<WifiApResult>, LocationError> {
    let mut reader = Reader::new(data);
    let mut results = Vec::new();

    while reader.pos < reader.data.len() {
        let tag = match reader.read_varint() {
            Some(t) => t,
            None => break,
        };
        let field = (tag >> 3) as u32;
        let wire_type = (tag & 0x07) as u32;

        match (field, wire_type) {
            (2, 2) => {
                let device = reader
                    .read_len_delimited()
                    .ok_or_else(|| LocationError::ParseError("Truncated wifi device".into()))?;
                if let Some(ap) = parse_wifi_device(device) {
                    results.push(ap);
                }
            }
            (_, 0) => {
                reader.read_varint();
            }
            (_, 2) => {
                reader.read_len_delimited();
            }
            _ => break,
        }
    }

    Ok(results)
}

pub fn query(bssids: &[String]) -> Result<Vec<WifiApResult>, LocationError> {
    if bssids.is_empty() {
        return Err(LocationError::ProviderFailed("No BSSIDs provided".into()));
    }

    let resp = networkkit::http::HttpRequest::post(WLOC_URL)
        .timeout(Duration::from_secs(10))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "*/*")
        .header("Accept-Language", "en-us")
        .header(
            "User-Agent",
            "locationd/2890.16.16 CFNetwork/1496.0.7 Darwin/23.5.0",
        )
        .body(build_request(bssids))
        .send()?;

    if !resp.is_success() {
        return Err(LocationError::ProviderFailed(format!(
            "Apple WLOC returned status {}",
            resp.status
        )));
    }

    let bytes = resp.bytes().to_vec();

    if bytes.len() <= RESPONSE_HEADER_LEN {
        return Err(LocationError::ParseError(
            "Apple WLOC response too short".into(),
        ));
    }

    parse_response(&bytes[RESPONSE_HEADER_LEN..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_roundtrip() {
        for value in [0u64, 1, 127, 128, 300, 5252000000, u64::MAX] {
            let mut buf = Vec::new();
            encode_varint(&mut buf, value);
            let mut reader = Reader::new(&buf);
            assert_eq!(reader.read_varint(), Some(value), "roundtrip failed for {}", value);
        }
    }

    #[test]
    fn zigzag_known_values() {
        assert_eq!(zigzag32(0), 0);
        assert_eq!(zigzag32(-1), 1);
        assert_eq!(zigzag32(1), 2);
        assert_eq!(zigzag32(-2), 3);
        assert_eq!(zigzag32(400), 800);
    }

    #[test]
    fn request_has_arpc_header_and_payload() {
        let body = build_request(&["AA:BB:CC:DD:EE:FF".to_string()]);

        assert_eq!(&body[0..2], &[0x00, 0x01]);
        assert_eq!(&body[2..4], &[0x00, 0x0A]);
        assert_eq!(&body[4..14], b"en-001_001");
        assert_eq!(&body[14..16], &[0x00, 0x13]);
        assert_eq!(&body[16..35], b"com.apple.locationd");
        assert_eq!(&body[35..37], &[0x00, 0x0D]);
        assert_eq!(&body[37..50], b"18.6.2.22G100");
        assert_eq!(&body[50..54], &[0x00, 0x00, 0x00, 0x01]);

        let payload_len = u32::from_be_bytes([body[54], body[55], body[56], body[57]]) as usize;
        assert_eq!(body.len(), 58 + payload_len);
        assert!(body[58..].windows(17).any(|w| w == b"AA:BB:CC:DD:EE:FF"));
    }

    #[test]
    fn parses_crafted_response() {
        let mut location = Vec::new();
        encode_tag(&mut location, 1, 0);
        encode_varint(&mut location, (52.52 * 1e8) as i64 as u64);
        encode_tag(&mut location, 2, 0);
        encode_varint(&mut location, (13.405 * 1e8) as i64 as u64);
        encode_tag(&mut location, 3, 0);
        encode_varint(&mut location, 42);

        let mut device = Vec::new();
        encode_string_field(&mut device, 1, "aa:bb:cc:dd:ee:ff");
        encode_message_field(&mut device, 2, &location);

        let mut block = Vec::new();
        encode_message_field(&mut block, 2, &device);

        let mut body = vec![0u8; RESPONSE_HEADER_LEN];
        body.extend_from_slice(&block);

        let results = parse_response(&body).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].bssid, "aa:bb:cc:dd:ee:ff");
        assert!((results[0].latitude - 52.52).abs() < 1e-9);
        assert!((results[0].longitude - 13.405).abs() < 1e-9);
        assert_eq!(results[0].accuracy_m, Some(42.0));
    }

    #[test]
    fn drops_invalid_coordinates() {
        let mut location = Vec::new();
        encode_tag(&mut location, 1, 0);
        encode_varint(&mut location, (-180.0 * 1e8) as i64 as u64);
        encode_tag(&mut location, 2, 0);
        encode_varint(&mut location, (-180.0 * 1e8) as i64 as u64);

        let mut device = Vec::new();
        encode_string_field(&mut device, 1, "aa:bb:cc:dd:ee:ff");
        encode_message_field(&mut device, 2, &location);

        let mut block = Vec::new();
        encode_message_field(&mut block, 2, &device);

        let results = parse_response(&block).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn skips_unknown_fields() {
        let mut location = Vec::new();
        encode_tag(&mut location, 1, 0);
        encode_varint(&mut location, (52.52 * 1e8) as i64 as u64);
        encode_tag(&mut location, 2, 0);
        encode_varint(&mut location, (13.405 * 1e8) as i64 as u64);

        let mut device = Vec::new();
        encode_string_field(&mut device, 1, "aa:bb:cc:dd:ee:ff");
        encode_message_field(&mut device, 2, &location);

        let mut block = Vec::new();
        encode_tag(&mut block, 3, 0);
        encode_varint(&mut block, 99);
        encode_string_field(&mut block, 5, "unknown-app-id");
        encode_message_field(&mut block, 2, &device);

        let results = parse_response(&block).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].bssid, "aa:bb:cc:dd:ee:ff");
    }

    #[test]
    fn drops_devices_without_location() {
        let mut device = Vec::new();
        encode_string_field(&mut device, 1, "aa:bb:cc:dd:ee:ff");

        let mut block = Vec::new();
        encode_message_field(&mut block, 2, &device);

        let results = parse_response(&block).unwrap();
        assert!(results.is_empty());
    }
}
