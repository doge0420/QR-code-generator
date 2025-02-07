use crate::bit::Bit;
use crate::encoding::Encoding;
use std::fmt;
use std::fmt::Formatter;

pub enum EcLevel {
    H,
    Q,
    M,
    L,
}

pub struct QrCode {
    data: Vec<Bit>,
    version: u8,
    ec_level: EcLevel,
    mask_pattern: u8,
    encoding: Encoding,
}

impl QrCode {
    fn get(&self, x: u32, y: u32) -> Option<Bit> {
        if let Some(index) = self.coords_to_index_from_instance(x, y) {
            self.data.get(index as usize).copied()
        } else {
            None
        }
    }

    fn put(&mut self, x: u32, y: u32, data: Bit) {
        if let Some(index) = self.coords_to_index_from_instance(x, y) {
            self.data[index as usize] = data;
        }
    }

    fn coords_to_index(x: u32, y: u32, size: u32) -> Option<u32> {
        if !(x < size && y < size) {
            None
        } else {
            Some(x + size * y)
        }
    }

    fn coords_to_index_from_version(x: u32, y: u32, version: u8) -> Option<u32> {
        Self::coords_to_index(x, y, Self::size_from_version(version))
    }

    fn coords_to_index_from_instance(&self, x: u32, y: u32) -> Option<u32> {
        Self::coords_to_index(x, y, self.size())
    }

    pub fn new(
        version: u8,
        ec_level: EcLevel,
        mask_pattern: u8,
        encoding: Encoding,
    ) -> Result<QrCode, String> {
        if version > 40 || version == 0 {
            Err("Invalid version.".to_string())
        } else {
            let size = Self::size_from_version(version);
            let data = vec![Bit::Zero(false); (size * size) as usize];
            Ok(QrCode {
                data,
                version,
                ec_level,
                mask_pattern,
                encoding,
            })
        }
    }

    fn size_from_version(version: u8) -> u32 {
        17 + 4 * version as u32
    }

    fn size(&self) -> u32 {
        Self::size_from_version(self.version)
    }

    fn finder_patterns(&mut self) {
        const FINDER_PATTERN: [Bit; 49] = [
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
        ];

        const PATTERN_LENGTH: u32 = 7;

        let size = self.size();
        let corners = [(0, 0), (size - 7, 0), (0, size - 7)];

        for corner in corners {
            let (x, y) = corner;
            for dx in 0..PATTERN_LENGTH {
                for dy in 0..PATTERN_LENGTH {
                    self.put(
                        x + dx,
                        y + dy,
                        FINDER_PATTERN[(dx + PATTERN_LENGTH * dy) as usize],
                    )
                }
            }
        }
    }

    fn separators_patterns(&mut self) {
        let size = self.size();
        let top = [(7, 0), (size - 8, 0), (7, size - 8)];
        let right = [(0, 7), (size - 7, 7), (0, size - 8)];

        for (x, y) in top {
            for dy in 0..8 {
                self.put(x, y + dy, Bit::Zero(true))
            }
        }

        for (x, y) in right {
            for dx in 0..7 {
                self.put(x + dx, y, Bit::Zero(true))
            }
        }
    }

    fn combination(array: &[u8]) -> Vec<(u8, u8)> {
        let mut res: Vec<(u8, u8)> = vec![];

        for elem1 in array {
            for elem2 in array {
                res.push((elem1.clone(), elem2.clone()));
            }
        }

        res
    }

    fn draw_alignment_pattern(&mut self, x: u32, y: u32) {
        let cx = x - 2;
        let cy = y - 2;

        const PATTERN_LENGTH: u32 = 5;

        const ALIGNMENT_PATTERN: [Bit; 25] = [
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::Zero(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
            Bit::One(true),
        ];

        if !self.get(x, y).unwrap().is_functional() {
            for dx in 0..PATTERN_LENGTH {
                for dy in 0..PATTERN_LENGTH {
                    self.put(
                        cx + dx,
                        cy + dy,
                        ALIGNMENT_PATTERN[(dx + PATTERN_LENGTH * dy) as usize],
                    )
                }
            }
        }
    }

    fn alignment_patterns(&mut self) {
        const COORDS: [&[u8]; 39] = [
            &[6, 18],
            &[6, 22],
            &[6, 26],
            &[6, 30],
            &[6, 34],
            &[6, 22, 38],
            &[6, 24, 42],
            &[6, 26, 46],
            &[6, 28, 50],
            &[6, 30, 54],
            &[6, 32, 58],
            &[6, 34, 62],
            &[6, 26, 46, 66],
            &[6, 26, 48, 70],
            &[6, 26, 50, 74],
            &[6, 30, 54, 78],
            &[6, 30, 56, 82],
            &[6, 30, 58, 86],
            &[6, 34, 62, 90],
            &[6, 28, 50, 72, 94],
            &[6, 26, 50, 74, 98],
            &[6, 30, 54, 78, 102],
            &[6, 28, 54, 80, 106],
            &[6, 32, 58, 84, 110],
            &[6, 30, 58, 86, 114],
            &[6, 34, 62, 90, 118],
            &[6, 26, 50, 74, 98, 122],
            &[6, 30, 54, 78, 102, 126],
            &[6, 26, 52, 78, 104, 130],
            &[6, 30, 56, 82, 108, 134],
            &[6, 34, 60, 86, 112, 138],
            &[6, 30, 58, 86, 114, 142],
            &[6, 34, 62, 90, 118, 146],
            &[6, 30, 54, 78, 102, 126, 150],
            &[6, 24, 50, 76, 102, 128, 154],
            &[6, 28, 54, 80, 106, 132, 158],
            &[6, 32, 58, 84, 110, 136, 162],
            &[6, 26, 54, 82, 110, 138, 166],
            &[6, 30, 58, 86, 114, 142, 170],
        ];

        if self.version == 1 {
            self.draw_alignment_pattern(18, 18);
        } else {
            let combinations = Self::combination(COORDS[(self.version - 2) as usize]);
            for (x, y) in combinations {
                self.draw_alignment_pattern(x as u32, y as u32);
            }
        }
    }

    fn timing_patterns(&mut self) {
        let length = self.size() - 16;

        let mut bit;
        for dx in 0..length {
            let x = dx + 8;

            if self.get(x, 6).unwrap().is_functional() {
                continue;
            }

            if x % 2 == 0 {
                bit = Bit::One(true);
            } else {
                bit = Bit::Zero(true);
            }

            self.put(x, 6, bit);
        }

        for dy in 0..length {
            let y = dy + 8;

            if self.get(6, y).unwrap().is_functional() {
                continue;
            }

            if y % 2 == 0 {
                bit = Bit::One(true);
            } else {
                bit = Bit::Zero(true);
            }

            self.put(6, y, bit);
        }
    }

    fn dark_module(&mut self) {
        self.put(8, (4 * self.version + 9) as u32, Bit::One(true))
    }

    fn format_information(&mut self) {
        const FORMAT_BITS: [u32; 32] = [
            0x77C4, 0x72F3, 0x7DAA, 0x789D, 0x662F, 0x6318, 0x6C41, 0x6976, 0x5412, 0x5125, 0x5E7C,
            0x5B4B, 0x45F9, 0x40CE, 0x4F97, 0x4AA0, 0x355F, 0x3068, 0x3F31, 0x3A06, 0x24B4, 0x2183,
            0x2EDA, 0x2BED, 0x1689, 0x13BE, 0x1CE7, 0x19D0, 0x762, 0x255, 0xD0C, 0x83B,
        ];

        let mut index = self.mask_pattern as u32;
        match self.ec_level {
            EcLevel::L => index += 0,
            EcLevel::M => index += 8 * 1,
            EcLevel::Q => index += 8 * 2,
            EcLevel::H => index += 8 * 3,
        }

        let info_bit = FORMAT_BITS[index as usize];
        let bits = Bit::from(info_bit, 15, true, true);

        let mut i = 0;
        for x in 0..9 {
            if self.get(x, 8).unwrap().is_functional() {
                continue;
            }
            self.put(x, 8, bits[i as usize]);
            i += 1;
        }

        let mut i = 7;
        for y in (0..9).rev() {
            if self.get(8, y).unwrap().is_functional() {
                continue;
            }
            self.put(8, y, bits[i as usize]);
            i += 1;
        }

        let mut i = 0;
        for y in ((self.size() - 7)..self.size()).rev() {
            self.put(8, y, bits[i as usize]);
            i += 1;
        }

        let mut i = 7;
        for x in (self.size() - 8)..self.size() {
            self.put(x, 8, bits[i as usize]);
            i += 1;
        }
    }

    fn version_information(&mut self) {
        assert!(
            self.version >= 7,
            "Version information is not available for versions below 7."
        );

        const VERSION_BITS: [u32; 34] = [
            0x07c94, 0x085bc, 0x09a99, 0x0a4d3, 0x0bbf6, 0x0c762, 0x0d847, 0x0e60d, 0x0f928,
            0x10b78, 0x1145d, 0x12a17, 0x13532, 0x149a6, 0x15683, 0x168c9, 0x177ec, 0x18ec4,
            0x191e1, 0x1afab, 0x1b08e, 0x1cc1a, 0x1d33f, 0x1ed75, 0x1f250, 0x209d5, 0x216f0,
            0x228ba, 0x2379f, 0x24b0b, 0x2542e, 0x26a64, 0x27541, 0x28c69,
        ];

        let version_bits = VERSION_BITS[(self.version - 7) as usize];
        let bits = Bit::from(version_bits, 18, true, true);

        // bottom left
        let mut x = 0;
        let mut y = self.size() - 11;
        for i in 0..18 {
            if i % 3 == 0 && i != 0 {
                x += 1;
                y = self.size() - 11;
            }
            self.put(x, y, bits[i as usize]);
            y += 1;
        }

        // top right
        let mut x = self.size() - 11;
        let mut y = 0;
        for i in 0..18 {
            if i % 3 == 0 && i != 0 {
                y += 1;
                x = self.size() - 11;
            }
            self.put(x, y, bits[i as usize]);
            x += 1;
        }
    }

    pub fn all_functional_patterns(&mut self) {
        self.finder_patterns();
        self.separators_patterns();
        self.alignment_patterns();
        self.timing_patterns();
        self.dark_module();
        self.format_information();
        if self.version >= 7 {
            self.version_information();
        }
    }

    pub fn fill(&mut self, bits: Vec<Bit>) {
        let mut x = self.size() as usize - 1;
        let mut y = self.size() as usize - 1;
        let mut up = true;

        let mut i = 0;

        for bit in bits {
            loop {
                match up {
                    true => {
                        // if we are at the top, go down and change column
                        if y <= 0 {
                            up = false;
                            x -= 1;
                            // skip the timing pattern
                            if x == 7 {
                                x -= 1;
                            }
                            y = 0;
                        } else {
                            if i % 2 == 0 {
                                x -= 1;
                            } else {
                                x += 1;
                                y -= 1;
                            }
                            i += 1;
                        }
                    }
                    false => {
                        // if we are at the bottom, go up and change column
                        if y >= self.size() as usize - 1 {
                            up = true;
                            x -= 1;
                            // skip the timing pattern
                            if x == 7 {
                                x -= 1;
                            }
                            y = self.size() as usize - 1;
                        } else {
                            if i % 2 == 0 {
                                x -= 1;
                            } else {
                                x += 1;
                                y += 1;
                            }
                            i += 1;
                        }
                    }
                }

                if !self.get(x as u32, y as u32).unwrap().is_functional() {
                    break;
                }
            }

            self.put(x as u32, y as u32, bit);
        }
    }
}

impl fmt::Display for QrCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut qrcode = String::new();

        for (i, module) in self.data.iter().enumerate() {
            if module.value() {
                qrcode.push_str("  ");
            } else {
                qrcode.push_str("██");
            }

            if (i + 1) % self.size() as usize == 0 {
                qrcode.push('\n')
            }
        }

        let mut version = String::from('\n');
        for _ in 0..(self.size() - 5) {
            version.push(' ');
        }
        version.push_str("Version: ");
        version.push_str(self.version.to_string().as_str());
        version.push('\n');

        write!(f, "{}{}", qrcode, version)
    }
}

static NUMERIC_SIZE: [u32; 160] = [
    41, 34, 27, 17, 77, 63, 48, 34, 127, 101, 77, 58, 187, 149, 111, 82, 255, 202, 144, 106, 322,
    255, 178, 139, 370, 293, 207, 154, 461, 365, 259, 202, 552, 432, 312, 235, 652, 513, 364, 288,
    772, 604, 427, 331, 883, 691, 489, 374, 1022, 796, 580, 427, 1101, 871, 621, 468, 1250, 991,
    703, 530, 1408, 1082, 775, 602, 1548, 1212, 876, 674, 1725, 1346, 948, 746, 1903, 1500, 1063,
    813, 2061, 1600, 1159, 919, 2232, 1708, 1224, 969, 2409, 1872, 1358, 1056, 2620, 2059, 1468,
    1108, 2812, 2188, 1588, 1228, 3057, 2395, 1718, 1286, 3283, 2544, 1804, 1425, 3517, 2701, 1933,
    1501, 3669, 2857, 2085, 1581, 3909, 3035, 2181, 1677, 4158, 3289, 2358, 1782, 4417, 3486, 2473,
    1897, 4686, 3693, 2670, 2022, 4965, 3909, 2805, 2157, 5253, 4134, 2949, 2301, 5529, 4343, 3081,
    2361, 5836, 4588, 3244, 2524, 6153, 4775, 3417, 2625, 6479, 5039, 3599, 2735, 6743, 5313, 3791,
    2927, 7089, 5596, 3993, 3057,
];

static ALPHANUMERIC_SIZE: [u32; 160] = [
    25, 20, 16, 10, 47, 38, 29, 20, 77, 61, 47, 35, 114, 90, 67, 50, 154, 122, 87, 64, 195, 154,
    108, 84, 224, 178, 125, 93, 279, 221, 157, 122, 335, 262, 189, 143, 395, 311, 221, 174, 468,
    366, 259, 200, 535, 419, 296, 227, 619, 483, 352, 259, 667, 528, 376, 283, 758, 600, 426, 321,
    854, 656, 470, 365, 938, 734, 531, 408, 1046, 816, 574, 452, 1153, 909, 644, 493, 1249, 970,
    702, 557, 1352, 1035, 742, 587, 1460, 1134, 823, 640, 1588, 1248, 890, 672, 1704, 1326, 963,
    744, 1853, 1451, 1041, 779, 1990, 1542, 1094, 864, 2132, 1637, 1172, 910, 2223, 1732, 1263,
    958, 2369, 1839, 1322, 1016, 2520, 1994, 1429, 1080, 2677, 2113, 1499, 1150, 2840, 2238, 1618,
    1226, 3009, 2369, 1700, 1307, 3183, 2506, 1787, 1394, 3351, 2632, 1867, 1431, 3537, 2780, 1966,
    1530, 3729, 2894, 2071, 1591, 3927, 3054, 2181, 1658, 4087, 3220, 2298, 1774, 4296, 3391, 2420,
    1852,
];

static BYTE_SIZE: [u32; 160] = [
    17, 14, 11, 7, 32, 26, 20, 14, 53, 42, 32, 24, 78, 62, 46, 34, 106, 84, 60, 44, 134, 106, 74,
    58, 154, 122, 86, 64, 192, 152, 108, 84, 230, 180, 130, 98, 271, 213, 151, 119, 321, 251, 177,
    137, 367, 287, 203, 155, 425, 331, 241, 177, 458, 362, 258, 194, 520, 412, 292, 220, 586, 450,
    322, 250, 644, 504, 364, 280, 718, 560, 394, 310, 792, 624, 442, 338, 858, 666, 482, 382, 929,
    711, 509, 403, 1003, 779, 565, 439, 1091, 857, 611, 461, 1171, 911, 661, 511, 1273, 997, 715,
    535, 1367, 1059, 751, 593, 1465, 1125, 805, 625, 1528, 1190, 868, 658, 1628, 1264, 908, 698,
    1732, 1370, 982, 742, 1840, 1452, 1030, 790, 1952, 1538, 1112, 842, 2068, 1628, 1168, 898,
    2188, 1722, 1228, 958, 2303, 1809, 1283, 983, 2431, 1911, 1351, 1051, 2563, 1989, 1423, 1093,
    2699, 2099, 1499, 1139, 2809, 2213, 1579, 1219, 2953, 2331, 1663, 1273,
];

static KANJI_SIZE: [u32; 160] = [
    10, 8, 7, 4, 20, 16, 12, 8, 32, 26, 20, 15, 48, 38, 28, 21, 65, 52, 37, 27, 82, 65, 45, 36, 95,
    75, 53, 39, 118, 93, 66, 52, 141, 111, 80, 60, 167, 131, 93, 74, 198, 155, 109, 85, 226, 177,
    125, 96, 262, 204, 149, 109, 282, 223, 159, 120, 320, 254, 180, 136, 361, 277, 198, 154, 397,
    310, 224, 173, 442, 345, 243, 191, 488, 384, 272, 208, 528, 410, 297, 235, 572, 438, 314, 248,
    618, 480, 348, 270, 672, 528, 376, 284, 721, 561, 407, 315, 784, 614, 440, 330, 842, 652, 462,
    365, 902, 692, 496, 385, 940, 732, 534, 405, 1002, 778, 559, 430, 1066, 843, 604, 457, 1132,
    894, 634, 486, 1201, 947, 684, 518, 1273, 1002, 719, 553, 1347, 1060, 756, 590, 1417, 1113,
    790, 605, 1496, 1176, 832, 647, 1577, 1224, 876, 673, 1661, 1292, 923, 701, 1729, 1362, 972,
    750, 1817, 1435, 1024, 784,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qrcode::Bit::{One, Zero};

    #[test]
    fn get_returns_correct_bit() {
        let qr = QrCode::new(1, EcLevel::L, 1, Encoding::Alphanumeric).unwrap();
        assert!(matches!(qr.get(0, 0), Some(Zero(_))));
    }

    #[test]
    fn get_returns_none_for_out_of_bounds() {
        let qr = QrCode::new(1, EcLevel::L, 1, Encoding::Alphanumeric).unwrap();
        assert_eq!(qr.get(100, 100), None);
    }

    #[test]
    fn new_returns_error_for_invalid_version() {
        let result = QrCode::new(41, EcLevel::L, 1, Encoding::Alphanumeric);
        assert!(result.is_err());
    }

    #[test]
    fn new_creates_qrcode_with_correct_size() {
        let qr = QrCode::new(1, EcLevel::L, 1, Encoding::Alphanumeric).unwrap();
        assert_eq!(qr.size(), 21);
    }

    #[test]
    fn size_from_version_calculates_correct_size() {
        assert_eq!(QrCode::size_from_version(1), 21);
        assert_eq!(QrCode::size_from_version(40), 177);
    }

    #[test]
    fn new_creates_qrcode_with_valid_version() {
        let qr = QrCode::new(10, EcLevel::M, 1, Encoding::Alphanumeric).unwrap();
        assert_eq!(qr.version, 10);
        assert_eq!(qr.size(), 57);
    }

    #[test]
    fn new_creates_qrcode_with_correct_ec_level() {
        let qr = QrCode::new(5, EcLevel::Q, 1, Encoding::Alphanumeric).unwrap();
        match qr.ec_level {
            EcLevel::Q => assert!(true),
            _ => assert!(false, "Expected EcLevel::Q"),
        }
    }

    #[test]
    fn new_creates_qrcode_with_correct_data_size() {
        let qr = QrCode::new(2, EcLevel::H, 1, Encoding::Alphanumeric).unwrap();
        assert_eq!(qr.data.len(), 625);
    }

    #[test]
    fn new_returns_error_for_zero_version() {
        let result = QrCode::new(0, EcLevel::L, 1, Encoding::Alphanumeric);
        assert!(result.is_err());
        assert_eq!(result.err(), Some("Invalid version.".to_string()));
    }

    #[test]
    fn new_returns_error_for_negative_version() {
        let result = QrCode::new(-1i8 as u8, EcLevel::L, 1, Encoding::Alphanumeric);
        assert!(result.is_err());
        assert_eq!(result.err(), Some("Invalid version.".to_string()));
    }

    #[test]
    fn size_from_version_calculates_size_for_min_version() {
        assert_eq!(QrCode::size_from_version(1), 21);
    }

    #[test]
    fn size_from_version_calculates_size_for_max_version() {
        assert_eq!(QrCode::size_from_version(40), 177);
    }

    #[test]
    fn size_from_version_calculates_size_for_intermediate_version() {
        assert_eq!(QrCode::size_from_version(20), 97);
    }

    #[test]
    fn size_from_version_calculates_size_for_large_version() {
        assert_eq!(QrCode::size_from_version(100), 417);
    }

    #[test]
    fn finder_patterns_creates_correct_patterns() {
        let mut qr = QrCode::new(1, EcLevel::L, 1, Encoding::Alphanumeric).unwrap();
        qr.finder_patterns();
        let expected_pattern = [
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (0, 1),
            (6, 1),
            (0, 2),
            (6, 2),
            (0, 3),
            (6, 3),
            (0, 4),
            (6, 4),
            (0, 5),
            (6, 5),
            (0, 6),
            (1, 6),
            (2, 6),
            (3, 6),
            (4, 6),
            (5, 6),
            (6, 6),
        ];
        for &(x, y) in &expected_pattern {
            assert_eq!(qr.get(x, y), Some(One(true)));
        }
    }

    #[test]
    fn finder_patterns_handles_minimum_size() {
        let mut qr = QrCode::new(1, EcLevel::L, 1, Encoding::Alphanumeric).unwrap();
        qr.finder_patterns();
        assert!(matches!(qr.get(0, 0), Some(One(_))));
        assert!(matches!(qr.get(20, 20), Some(Zero(_))));
    }

    #[test]
    fn finder_patterns_handles_maximum_size() {
        let mut qr = QrCode::new(40, EcLevel::L, 1, Encoding::Alphanumeric).unwrap();
        qr.finder_patterns();
        assert!(matches!(qr.get(0, 0), Some(One(_))));
        assert!(matches!(qr.get(176, 176), Some(Zero(_))));
    }
}
