//! The uniform block shared by every shader, mirrored on the CPU side.

/// Values given to the shader each frame, laid out like the GLSL block
/// `FenetreGlobals` in [`crate::prepare`] (std140 rules).
///
/// | Field          | GLSL    | Offset |
/// |----------------|---------|--------|
/// | `resolution`   | `vec3`  | 0      |
/// | `time`         | `float` | 12     |
/// | `time_delta`   | `float` | 16     |
/// | `frame`        | `int`   | 20     |
/// | (padding)      |         | 24     |
/// | `mouse`        | `vec4`  | 32     |
/// | `date`         | `vec4`  | 48     |
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Globals {
    /// `iResolution`: width, height and pixel aspect ratio (usually 1).
    pub resolution: [f32; 3],
    /// `iTime`: seconds since the shader started.
    pub time: f32,
    /// `iTimeDelta`: seconds since the previous frame.
    pub time_delta: f32,
    /// `iFrame`: number of frames drawn since the shader started.
    pub frame: i32,
    /// `iMouse`: Shadertoy mouse vector (`xy` current, `zw` click).
    pub mouse: [f32; 4],
    /// `iDate`: year, month (0–11), day (1–31), seconds since midnight.
    pub date: [f32; 4],
}

impl Globals {
    /// Size of the uniform block in bytes.
    pub const SIZE: usize = 64;

    /// Encodes the values in the GPU layout (little endian, std140).
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0; Self::SIZE];
        let mut put = |offset: usize, value: [u8; 4]| {
            bytes[offset..offset + 4].copy_from_slice(&value);
        };
        for (i, v) in self.resolution.iter().enumerate() {
            put(i * 4, v.to_le_bytes());
        }
        put(12, self.time.to_le_bytes());
        put(16, self.time_delta.to_le_bytes());
        put(20, self.frame.to_le_bytes());
        for (i, v) in self.mouse.iter().enumerate() {
            put(32 + i * 4, v.to_le_bytes());
        }
        for (i, v) in self.date.iter().enumerate() {
            put(48 + i * 4, v.to_le_bytes());
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f32_at(bytes: &[u8], offset: usize) -> f32 {
        f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    #[test]
    fn fields_land_at_std140_offsets() {
        let globals = Globals {
            resolution: [640.0, 360.0, 1.0],
            time: 2.5,
            time_delta: 0.016,
            frame: 7,
            mouse: [1.0, 2.0, 3.0, 4.0],
            date: [2026.0, 10.0, 19.0, 3600.0],
        };
        let bytes = globals.to_bytes();
        assert_eq!(f32_at(&bytes, 4), 360.0);
        assert_eq!(f32_at(&bytes, 12), 2.5);
        assert_eq!(f32_at(&bytes, 16), 0.016);
        assert_eq!(i32::from_le_bytes(bytes[20..24].try_into().unwrap()), 7);
        assert_eq!(&bytes[24..32], &[0; 8]);
        assert_eq!(f32_at(&bytes, 44), 4.0);
        assert_eq!(f32_at(&bytes, 60), 3600.0);
    }
}
