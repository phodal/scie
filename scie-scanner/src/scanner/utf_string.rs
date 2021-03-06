#[derive(Clone, Debug, Serialize)]
pub struct UtfString {
    pub utf16length: i32,
    pub utf8length: i32,
    pub utf16value: String,
    pub utf8value: Vec<u8>,
    pub utf16offset_to_utf8: Vec<u32>,
    pub utf8offset_to_utf16: Vec<u32>,
}

impl UtfString {
    // todo: find better way for performance
    // TypeScript default use utf16, Rust default use UTF8
    pub fn new(str: &str) -> Self {
        let utf16_vec: Vec<u16> = str.encode_utf16().collect();
        let utf16length = utf16_vec.len();
        let utf8length = str.len();
        let mut utf8value = str.to_string().into_bytes();

        let compute_indices_mapping = utf8length != utf16length;

        let mut utf16offset_to_utf8: Vec<u32> = vec![];
        let mut utf8offset_to_utf16: Vec<u32> = vec![];

        if compute_indices_mapping {
            utf16offset_to_utf8 = vec![0; utf16length + 1];
            utf16offset_to_utf8[utf16length] = utf8length as u32;

            utf8offset_to_utf16 = vec![0; utf8length + 1];
            utf8offset_to_utf16[utf8length] = utf16length as u32;
        }

        let mut i8: usize = 0;
        let mut i16 = 0;
        while i16 < utf16_vec.len() {
            let char_code = utf16_vec[i16];
            let mut code_point = char_code as usize;
            let mut was_surrogate_pair = false;
            if char_code >= 0xd800 && char_code <= 0xdbff {
                if i16 + 1 <= utf16length {
                    let next_char_code = utf16_vec[i16 + 1];
                    if next_char_code >= 0xdc00 && next_char_code <= 0xdfff {
                        let temp = ((char_code - 0xd800) << 10) as usize + 0x10000;
                        code_point = (temp as usize) | (next_char_code as usize - 0xdc00);
                        was_surrogate_pair = true;
                    }
                }
            }

            if compute_indices_mapping {
                utf16offset_to_utf8[i16] = i8 as u32;

                if was_surrogate_pair {
                    utf16offset_to_utf8[i16 + 1] = i8 as u32;
                }

                if code_point <= 0x7f {
                    utf8offset_to_utf16[i8 + 0] = i16 as u32;
                } else if code_point <= 0x7ff {
                    utf8offset_to_utf16[i8 + 0] = i16 as u32;
                    utf8offset_to_utf16[i8 + 1] = i16 as u32;
                } else if code_point <= 0xffff {
                    utf8offset_to_utf16[i8 + 0] = i16 as u32;
                    utf8offset_to_utf16[i8 + 1] = i16 as u32;
                    utf8offset_to_utf16[i8 + 2] = i16 as u32;
                } else {
                    utf8offset_to_utf16[i8 + 0] = i16 as u32;
                    utf8offset_to_utf16[i8 + 1] = i16 as u32;
                    utf8offset_to_utf16[i8 + 2] = i16 as u32;
                    utf8offset_to_utf16[i8 + 3] = i16 as u32;
                }
            }

            if code_point <= 0x7f {
                utf8value[i8] = code_point as u8;
                i8 = i8 + 1;
            } else if code_point <= 0x7ff {
                utf8value[i8] =
                    (0b11000000 | ((code_point & 0b00000000000000000000011111000000) >> 6)) as u8;
                i8 = i8 + 1;
                utf8value[i8] =
                    (0b10000000 | ((code_point & 0b00000000000000000000000000111111) >> 0)) as u8;
                i8 = i8 + 1;
            } else if code_point <= 0xffff {
                utf8value[i8] =
                    (0b11100000 | ((code_point & 0b00000000000000001111000000000000) >> 12)) as u8;
                i8 = i8 + 1;
                utf8value[i8] =
                    (0b10000000 | ((code_point & 0b00000000000000000000111111000000) >> 6)) as u8;
                i8 = i8 + 1;
                utf8value[i8] =
                    (0b10000000 | ((code_point & 0b00000000000000000000000000111111) >> 0)) as u8;
                i8 = i8 + 1;
            } else {
                utf8value[i8] =
                    (0b11110000 | ((code_point & 0b00000000000111000000000000000000) >> 18)) as u8;
                i8 = i8 + 1;
                utf8value[i8] =
                    (0b10000000 | ((code_point & 0b00000000000000111111000000000000) >> 12)) as u8;
                i8 = i8 + 1;
                utf8value[i8] =
                    (0b10000000 | ((code_point & 0b00000000000000000000111111000000) >> 6)) as u8;
                i8 = i8 + 1;
                utf8value[i8] =
                    (0b10000000 | ((code_point & 0b00000000000000000000000000111111) >> 0)) as u8;
                i8 = i8 + 1;
            }

            if was_surrogate_pair {
                i16 = i16 + 1;
            }

            i16 = i16 + 1;
        }

        UtfString {
            utf16length: utf16length as i32,
            utf8length: utf8length as i32,
            utf16value: String::from(str),
            utf8value,
            utf16offset_to_utf8,
            utf8offset_to_utf16,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::utf_string::UtfString;

    #[test]
    fn should_convert_utf_string_success() {
        let onig_string = UtfString::new("a💻bYX");

        assert_eq!(6, onig_string.utf16length);
        assert_eq!(8, onig_string.utf8length);
        assert_eq!(
            vec![97, 240, 159, 146, 187, 98, 89, 88],
            onig_string.utf8value
        );
        assert_eq!(vec![0, 1, 1, 5, 6, 7, 8], onig_string.utf16offset_to_utf8);
        assert_eq!(
            vec![0, 1, 1, 1, 1, 3, 4, 5, 6],
            onig_string.utf8offset_to_utf16
        );
    }

    #[test]
    fn should_handle_normal_string() {
        let onig_string = UtfString::new("12");
        assert_eq!(2, onig_string.utf16length.clone());
        assert_eq!(2, onig_string.utf8length.clone());
    }
}
