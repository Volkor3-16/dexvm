//! `java.net.URLEncoder` / `URLDecoder` form encoding shims.

use crate::vm::native::*;

const WINDOWS_1252: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}',
    '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
];

fn form_bytes(value: &str, charset: &str) -> Vec<u8> {
    match charset {
        "UTF-16" | "UTF-16BE" => {
            let mut bytes = Vec::new();
            if charset == "UTF-16" {
                bytes.extend_from_slice(&[0xFE, 0xFF]);
            }
            for unit in value.encode_utf16() {
                bytes.extend_from_slice(&unit.to_be_bytes());
            }
            bytes
        }
        "UTF-16LE" => value.encode_utf16().flat_map(u16::to_le_bytes).collect(),
        "UTF-32" => {
            let mut bytes = vec![0, 0, 0xFE, 0xFF];
            for ch in value.chars() {
                bytes.extend_from_slice(&(ch as u32).to_be_bytes());
            }
            bytes
        }
        "ISO-8859-1" | "US-ASCII" | "WINDOWS-1252" => value
            .chars()
            .map(|ch| {
                let code = ch as u32;
                if charset == "WINDOWS-1252" {
                    if let Some(index) = WINDOWS_1252.iter().position(|&entry| entry == ch) {
                        return (index + 0x80) as u8;
                    }
                }
                let max = if charset == "US-ASCII" { 0x7F } else { 0xFF };
                if code <= max && (charset != "WINDOWS-1252" || !(0x80..=0x9F).contains(&code)) {
                    code as u8
                } else {
                    b'?'
                }
            })
            .collect(),
        _ => value.as_bytes().to_vec(),
    }
}

fn form_text(bytes: &[u8], charset: &str) -> String {
    match charset {
        "UTF-16" | "UTF-16BE" | "UTF-16LE" => {
            let (little_endian, bytes) = match charset {
                "UTF-16" if bytes.starts_with(&[0xFF, 0xFE]) => (true, &bytes[2..]),
                "UTF-16" if bytes.starts_with(&[0xFE, 0xFF]) => (false, &bytes[2..]),
                "UTF-16LE" => (true, bytes),
                _ => (false, bytes),
            };
            let units = bytes.as_chunks::<2>().0.iter().map(|pair| {
                if little_endian {
                    u16::from_le_bytes([pair[0], pair[1]])
                } else {
                    u16::from_be_bytes([pair[0], pair[1]])
                }
            });
            String::from_utf16_lossy(&units.collect::<Vec<_>>())
        }
        "UTF-32" => {
            let bytes = bytes.strip_prefix(&[0, 0, 0xFE, 0xFF]).unwrap_or(bytes);
            bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|part| {
                    char::from_u32(u32::from_be_bytes([part[0], part[1], part[2], part[3]]))
                        .unwrap_or('\u{FFFD}')
                })
                .collect()
        }
        "ISO-8859-1" => bytes.iter().map(|&byte| char::from(byte)).collect(),
        "US-ASCII" => bytes
            .iter()
            .map(|&byte| {
                if byte <= 0x7F {
                    char::from(byte)
                } else {
                    '\u{FFFD}'
                }
            })
            .collect(),
        "WINDOWS-1252" => bytes
            .iter()
            .map(|&byte| {
                if (0x80..=0x9F).contains(&byte) {
                    WINDOWS_1252[(byte - 0x80) as usize]
                } else {
                    char::from(byte)
                }
            })
            .collect(),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

fn form_encode(value: &[u8]) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'*' => {
                out.push(*b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn form_decode(value: &str) -> Vec<u8> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    out.push((hi * 16 + lo) as u8);
                    i += 3;
                } else {
                    out.push(b'%');
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    out
}

/// `URLEncoder.encode(s, charset)` and `URLDecoder.decode(s, charset)` take the
/// text as the *first* argument. Reading it from the wrong slot encoded the
/// wrong string and indexed past the end of the argument list.
fn form_value(vm: &mut Vm, args: &[JValue]) -> Result<String, NatErr> {
    let Some(value) = args.first() else {
        return Err(aioobe(vm, 0, args.len() as i32));
    };
    jstr(vm, *value)
}

/// Charset argument: UTF-8 when it is absent or not a string value.
fn form_charset(vm: &Vm, args: &[JValue]) -> String {
    args.get(1)
        .and_then(|v| peek_str(vm, *v).map(|s| s.to_string()))
        .unwrap_or_else(|| "UTF-8".to_string())
}

pub(crate) fn url_encoder_encode(vm: &mut Vm, args: &[JValue]) -> R {
    let value = form_value(vm, args)?;
    let charset = form_charset(vm, args);
    let Some(charset) = super::super::nio::normalize_charset(&charset) else {
        return Err(iae(vm, format!("Unsupported charset: {charset}")));
    };
    let encoded = form_encode(&form_bytes(&value, &charset));
    Ok(new_str(vm, &encoded))
}

pub(crate) fn url_decoder_decode(vm: &mut Vm, args: &[JValue]) -> R {
    let value = form_value(vm, args)?;
    let charset = form_charset(vm, args);
    let Some(charset) = super::super::nio::normalize_charset(&charset) else {
        return Err(iae(vm, format!("Unsupported charset: {charset}")));
    };
    let decoded = form_text(&form_decode(&value), &charset);
    Ok(new_str(vm, &decoded))
}

fn url_init(vm: &mut Vm, args: &[JValue]) -> R {
    let value = jstr(vm, args[1])?;
    let JValue::Obj(id) = args[0] else {
        return Err(npe(vm));
    };
    vm.arena.objects[id as usize].native = Some(Native::URI(value));
    Ok(JValue::Null)
}

fn url_get_host(vm: &mut Vm, args: &[JValue]) -> R {
    let value = match payload(vm, args[0]) {
        Some(Native::URI(value)) => value.clone(),
        _ => return Err(npe(vm)),
    };
    if value.starts_with("resource:/") {
        return Ok(new_str(vm, ""));
    }
    let authority = value
        .split("://")
        .last()
        .and_then(|part| part.strip_prefix("//").or(Some(part)))
        .unwrap_or(&value);
    Ok(new_str(
        vm,
        authority.split(['/', ':']).next().unwrap_or(""),
    ))
}

fn url_get_path(vm: &mut Vm, args: &[JValue]) -> R {
    let value = match payload(vm, args[0]) {
        Some(Native::URI(value)) => value.clone(),
        _ => return Err(npe(vm)),
    };
    if let Some(path) = value.strip_prefix("resource:") {
        return Ok(new_str(vm, path));
    }
    let after_scheme = value.split("://").last().unwrap_or(&value);
    let path = if let Some(rest) = after_scheme.strip_prefix("//") {
        rest.find('/').map(|i| &rest[i..]).unwrap_or("")
    } else {
        after_scheme
    };
    Ok(new_str(vm, path.split(['?', '#']).next().unwrap_or("")))
}

fn url_open_stream(vm: &mut Vm, args: &[JValue]) -> R {
    let key = match payload(vm, args[0]) {
        Some(Native::URI(value)) => value.strip_prefix("resource:/").map(str::to_owned),
        _ => None,
    };
    let Some(bytes) = key.and_then(|key| vm.resources.get(&key).cloned()) else {
        return Err(iae(vm, "URL resource unavailable"));
    };
    alloc(
        vm,
        "Ljava/io/ByteArrayInputStream;",
        Native::ByteArrayInputStream { bytes, pos: 0 },
    )
}

pub(crate) const TABLE: &[NativeEntry] = &[
    ne!(
        "Ljava/net/URL;",
        "openStream",
        "()Ljava/io/InputStream;",
        true,
        url_open_stream
    ),
    ne!(
        "Ljava/net/URLEncoder;",
        "encode",
        "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
        false,
        url_encoder_encode
    ),
    ne!(
        "Ljava/net/URLEncoder;",
        "encode",
        "(Ljava/lang/String;)Ljava/lang/String;",
        false,
        url_encoder_encode
    ),
    ne!(
        "Ljava/net/URLDecoder;",
        "decode",
        "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
        false,
        url_decoder_decode
    ),
    ne!(
        "Ljava/net/URLDecoder;",
        "decode",
        "(Ljava/lang/String;)Ljava/lang/String;",
        false,
        url_decoder_decode
    ),
    ne!(
        "Ljava/net/URL;",
        "<init>",
        "(Ljava/lang/String;)V",
        true,
        url_init
    ),
    ne!(
        "Ljava/net/URL;",
        "getHost",
        "()Ljava/lang/String;",
        true,
        url_get_host
    ),
    ne!(
        "Ljava/net/URL;",
        "getPath",
        "()Ljava/lang/String;",
        true,
        url_get_path
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_encoding_round_trips_utf8() {
        let encoded = form_encode("a b+c/你好".as_bytes());
        assert_eq!(encoded, "a+b%2Bc%2F%E4%BD%A0%E5%A5%BD");
        assert_eq!(form_text(&form_decode(&encoded), "UTF-8"), "a b+c/你好");
    }

    fn bare_vm() -> Vm {
        Vm::new(vec![], Box::new(std::io::sink())).expect("bare vm")
    }

    fn text(vm: &Vm, v: JValue) -> String {
        peek_str(vm, v).expect("string value").to_string()
    }

    /// The exception a rejected call raises, named the way real Java names it.
    fn thrown(vm: &mut Vm, e: NatErr) -> String {
        let NatErr::Throw(id) = e else {
            panic!("expected a java throwable, got {e:?}");
        };
        let class = vm.arena.objects[id as usize].class;
        vm.class_desc_str(class).to_string()
    }

    const RAW: &str = "a b&c=d/e+é";
    const ENCODED: &str = "a+b%26c%3Dd%2Fe%2B%C3%A9";

    #[test]
    fn encode_takes_the_value_from_the_first_argument() {
        let mut vm = bare_vm();
        let value = new_str(&mut vm, RAW);
        let charset = new_str(&mut vm, "UTF-8");
        let out = url_encoder_encode(&mut vm, &[value, charset]).expect("encode");
        assert_eq!(text(&vm, out), ENCODED, "encode the value, not the charset");
    }

    #[test]
    fn both_platform_encode_overloads_are_registered() {
        let sigs: Vec<&str> = TABLE
            .iter()
            .filter(|e| e.class == "Ljava/net/URLEncoder;" && e.name == "encode")
            .map(|e| e.sig)
            .collect();
        assert_eq!(
            sigs,
            vec![
                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
                "(Ljava/lang/String;)Ljava/lang/String;",
            ],
            "java.net.URLEncoder needs encode(s) as well as encode(s, charset)"
        );
    }

    #[test]
    fn single_argument_call_encodes_the_value_with_the_default_charset() {
        let mut vm = bare_vm();
        let value = new_str(&mut vm, RAW);
        let out = url_encoder_encode(&mut vm, &[value]).expect("encode");
        assert_eq!(text(&vm, out), ENCODED);
    }

    #[test]
    fn decode_round_trips_the_encoded_value() {
        let mut vm = bare_vm();
        let encoded = new_str(&mut vm, ENCODED);
        let charset = new_str(&mut vm, "UTF-8");
        let out = url_decoder_decode(&mut vm, &[encoded, charset]).expect("decode");
        assert_eq!(text(&vm, out), RAW);
        let out = url_decoder_decode(&mut vm, &[encoded]).expect("decode");
        assert_eq!(text(&vm, out), RAW);
    }

    #[test]
    fn missing_and_mistyped_arguments_raise_java_exceptions() {
        let mut vm = bare_vm();

        let err = url_encoder_encode(&mut vm, &[]).expect_err("no argument at all");
        assert_eq!(
            thrown(&mut vm, err),
            "java.lang.ArrayIndexOutOfBoundsException"
        );

        // A DEX null reference in the value slot.
        let err = url_encoder_encode(&mut vm, &[JValue::Int(0)]).expect_err("null value");
        assert_eq!(thrown(&mut vm, err), "java.lang.NullPointerException");
    }

    #[test]
    fn a_charset_that_is_not_a_string_falls_back_to_utf8() {
        let mut vm = bare_vm();
        let value = new_str(&mut vm, RAW);
        let out = url_encoder_encode(&mut vm, &[value, JValue::Int(7)]).expect("encode");
        assert_eq!(text(&vm, out), ENCODED);
    }
}
