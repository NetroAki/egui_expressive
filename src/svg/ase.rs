use super::*;

#[derive(Debug)]
pub enum AseError {
    InvalidMagic,
    InvalidVersion,
    UnexpectedEof,
    Utf16Error,
}

impl std::fmt::Display for AseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AseError::InvalidMagic => write!(f, "Invalid ASE magic bytes"),
            AseError::InvalidVersion => write!(f, "Invalid or unsupported ASE version"),
            AseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            AseError::Utf16Error => write!(f, "Invalid UTF-16 encoding in ASE file"),
        }
    }
}

impl std::error::Error for AseError {}

/// Parse an Adobe Swatch Exchange (.ase) binary file.
/// Returns a list of (name, Color32) pairs.
pub fn parse_ase(bytes: &[u8]) -> Result<Vec<(String, Color32)>, AseError> {
    if bytes.len() < 12 {
        return Err(AseError::UnexpectedEof);
    }

    // Check magic bytes "ASEF"
    if &bytes[0..4] != b"ASEF" {
        return Err(AseError::InvalidMagic);
    }

    // Read version (big-endian)
    let major_version = u16::from_be_bytes([bytes[4], bytes[5]]);
    let minor_version = u16::from_be_bytes([bytes[6], bytes[7]]);

    if major_version != 1 || minor_version != 0 {
        return Err(AseError::InvalidVersion);
    }

    // Read block count (big-endian u32)
    let block_count = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);

    let mut colors = Vec::new();
    let mut offset = 12;

    for _ in 0..block_count {
        if offset + 6 > bytes.len() {
            return Err(AseError::UnexpectedEof);
        }

        // Read block type (big-endian u16)
        let block_type = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;

        // Read block length (big-endian u32)
        let block_length = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        match block_type {
            0x0001 => {
                // Color block
                if offset + 2 > bytes.len() {
                    return Err(AseError::UnexpectedEof);
                }

                // Read name length (big-endian u16)
                let name_len = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
                offset += 2;

                let name_bytes_len = (name_len as usize) * 2;
                if offset + name_bytes_len > bytes.len() {
                    return Err(AseError::UnexpectedEof);
                }

                // Read name as UTF-16BE
                let mut name_chars = Vec::with_capacity(name_len as usize);
                for i in 0..name_len {
                    let char_bytes = [
                        bytes[offset + i as usize * 2],
                        bytes[offset + i as usize * 2 + 1],
                    ];
                    let c = char::from_u32(u16::from_be_bytes(char_bytes) as u32)
                        .ok_or(AseError::Utf16Error)?;
                    name_chars.push(c);
                }
                let name: String = name_chars.into_iter().collect();
                offset += name_bytes_len;

                if offset + 4 > bytes.len() {
                    return Err(AseError::UnexpectedEof);
                }

                // Read color model (4 bytes)
                let color_model = &bytes[offset..offset + 4];
                offset += 4;

                let color = match color_model {
                    b"RGB " => {
                        if offset + 12 > bytes.len() {
                            return Err(AseError::UnexpectedEof);
                        }

                        let r = f32::from_be_bytes([
                            bytes[offset],
                            bytes[offset + 1],
                            bytes[offset + 2],
                            bytes[offset + 3],
                        ]);
                        let g = f32::from_be_bytes([
                            bytes[offset + 4],
                            bytes[offset + 5],
                            bytes[offset + 6],
                            bytes[offset + 7],
                        ]);
                        let b = f32::from_be_bytes([
                            bytes[offset + 8],
                            bytes[offset + 9],
                            bytes[offset + 10],
                            bytes[offset + 11],
                        ]);

                        offset += 12;

                        let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
                        let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
                        let b = (b.clamp(0.0, 1.0) * 255.0) as u8;

                        Color32::from_rgb(r, g, b)
                    }
                    b"CMYK" => {
                        if offset + 16 > bytes.len() {
                            return Err(AseError::UnexpectedEof);
                        }

                        let c = f32::from_be_bytes([
                            bytes[offset],
                            bytes[offset + 1],
                            bytes[offset + 2],
                            bytes[offset + 3],
                        ]);
                        let m = f32::from_be_bytes([
                            bytes[offset + 4],
                            bytes[offset + 5],
                            bytes[offset + 6],
                            bytes[offset + 7],
                        ]);
                        let y = f32::from_be_bytes([
                            bytes[offset + 8],
                            bytes[offset + 9],
                            bytes[offset + 10],
                            bytes[offset + 11],
                        ]);
                        let k = f32::from_be_bytes([
                            bytes[offset + 12],
                            bytes[offset + 13],
                            bytes[offset + 14],
                            bytes[offset + 15],
                        ]);

                        offset += 16;

                        // CMYK to RGB conversion
                        let c = c.clamp(0.0, 1.0);
                        let m = m.clamp(0.0, 1.0);
                        let y = y.clamp(0.0, 1.0);
                        let k = k.clamp(0.0, 1.0);

                        let r = (255.0 * (1.0 - c) * (1.0 - k)) as u8;
                        let g = (255.0 * (1.0 - m) * (1.0 - k)) as u8;
                        let b = (255.0 * (1.0 - y) * (1.0 - k)) as u8;

                        Color32::from_rgb(r, g, b)
                    }
                    b"Gray" => {
                        if offset + 4 > bytes.len() {
                            return Err(AseError::UnexpectedEof);
                        }

                        let gray = f32::from_be_bytes([
                            bytes[offset],
                            bytes[offset + 1],
                            bytes[offset + 2],
                            bytes[offset + 3],
                        ]);
                        offset += 4;

                        let gray = (gray.clamp(0.0, 1.0) * 255.0) as u8;
                        Color32::from_gray(gray)
                    }
                    b"LAB " => {
                        // LAB is not supported, skip the color data (3 floats = 12 bytes)
                        if offset + 12 > bytes.len() {
                            return Err(AseError::UnexpectedEof);
                        }
                        offset += 12;
                        continue; // Don't add this color
                    }
                    _ => {
                        // Unknown color model, skip the block content
                        let skip = (block_length as usize).saturating_sub(10 + name_bytes_len);
                        offset += skip;
                        continue;
                    }
                };

                colors.push((name, color));
            }
            0xC001 => {
                // Group start - skip content
                offset += block_length as usize;
            }
            0xC002 => {
                // Group end - skip content
                offset += block_length as usize;
            }
            _ => {
                // Unknown block type, skip
                offset += block_length as usize;
            }
        }

        // Align to even byte boundary (ASE spec requires this)
        if offset % 2 != 0 {
            offset += 1;
        }
    }

    Ok(colors)
}

/// Convert ASE parse result to a flat Vec of Color32 values (names discarded).
pub fn ase_to_colors(bytes: &[u8]) -> Result<Vec<Color32>, AseError> {
    let colors = parse_ase(bytes)?;
    Ok(colors.into_iter().map(|(_, color)| color).collect())
}
