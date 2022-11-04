use strum::{IntoEnumIterator, EnumCount};
use strum_macros::{EnumIter, EnumCount};

#[derive(Clone, Copy, Debug, EnumIter, EnumCount, PartialEq, Eq)]
pub enum PixelColor {
    Maroon, // #6d001a
    Red, // #be0039
    RedOrange, // #ff4500
    Orange, // #ffa800
    Yellow, // #ffd635
    LightYellow, // #fff8b8
    ForestGreen, // #00a368
    Green, // #00cc78 
    YellowGreen, // #7eed56
    DarkGreen, // #00756f
    BlueGreen, // #009eaa
    Aquamarine, // #00ccc0
    DarkBlue, // #2450a4
    Blue, // #3690ea
    Teal, // #51e9f4
    DarkBluePurple, // #493ac1
    BluePurple, // #6a5cff
    BabyBlue, // #94b3ff
    Grape, // #811e9f
    Purple, // #b44ac0
    Lavender, // #e4abff
    Lipstick, // #de107f
    Salmon, // #ff3881 
    LightSalmon, // #ff99aa
    Mud, // #6d482f
    Brown, // #9c6926
    Tan, // #ffb470
    Black, // #000000
    DarkGrey, // #515252
    Grey, // #898d90
    LightGrey, // #d4d7d9
    White, // #ffffff
}

// Default pixel color is black
impl Default for PixelColor {
    fn default() -> Self {
        PixelColor::Black
    }
}

impl TryFrom<&String> for PixelColor {
    type Error = ();

    fn try_from(item: &String) -> Result<Self, Self::Error> {
        let no_prefix_item = (*item).trim_start_matches("#");
        let int_value = u32::from_str_radix(no_prefix_item, 16);
       
        match int_value {
            Ok(value) => {
                match PixelColor::try_from(value) {
                    Ok(color) => Ok(color),
                    Err(_)    => Err(()),
                }
            },
            _ => Err(()),
        }
    }
}

impl From<PixelColor> for u32 {
    fn from(item: PixelColor) -> Self {
        match item {
            PixelColor::Maroon => 0x6d001a,
            PixelColor::Red => 0xbe0039,
            PixelColor::RedOrange => 0xff4500,
            PixelColor::Orange => 0xffa800,
            PixelColor::Yellow => 0xffd635,
            PixelColor::LightYellow => 0xfff8b8,
            PixelColor::ForestGreen => 0x00a368,
            PixelColor::Green => 0x00cc78,
            PixelColor::YellowGreen => 0x7eed56,
            PixelColor::DarkGreen => 0x00756f,
            PixelColor::BlueGreen => 0x009eaa,
            PixelColor::Aquamarine => 0x00ccc0,
            PixelColor::DarkBlue => 0x2450a4,
            PixelColor::Blue => 0x3690ea,
            PixelColor::Teal => 0x51e9f4,
            PixelColor::DarkBluePurple => 0x493ac1,
            PixelColor::BluePurple => 0x6a5cff,
            PixelColor::BabyBlue => 0x94b3ff,
            PixelColor::Grape => 0x811e9f,
            PixelColor::Purple => 0xb44ac0,
            PixelColor::Lavender => 0xe4abff,
            PixelColor::Lipstick => 0xde107f,
            PixelColor::Salmon => 0xff3881,
            PixelColor::LightSalmon => 0xff99aa,
            PixelColor::Mud => 0x6d482f,
            PixelColor::Brown => 0x9c6926,
            PixelColor::Tan => 0xffb470,
            PixelColor::Black => 0x000000,
            PixelColor::DarkGrey => 0x515252,
            PixelColor::Grey => 0x898d90,
            PixelColor::LightGrey => 0xd4d7d9,
            PixelColor::White => 0xffffff,
        }
    }
}

impl From<PixelColor> for u64 {
    fn from(item: PixelColor) -> Self {
        u32::from(item) as u64
    }
}

impl TryFrom<u32> for PixelColor {
    type Error = ();

    fn try_from(item: u32) -> Result<Self, Self::Error> {
        for color in PixelColor::iter() {
            if item == color.into() {
                return Ok(color);
            }
        }

        return Err(());
    }
}

impl TryFrom<u64> for PixelColor {
    type Error = ();

    fn try_from(item: u64) -> Result<Self, Self::Error> {
        for color in PixelColor::iter() {
            if item == color.into() {
                return Ok(color);
            }
        }

        return Err(());
    }
}

impl From<PixelColor> for speedy2d::color::Color {
    fn from(item: PixelColor) -> Self {
        let int_value: u32 = item.into();
        return speedy2d::color::Color::from_hex_rgb(int_value);
    }
}

pub fn basic_pixel_pattern() -> Vec<Vec<PixelColor>> {
    let mut pixels : Vec<Vec<PixelColor>> = Vec::new();
    for i in 0..PixelColor::COUNT {
        let mut row: Vec<PixelColor> = Vec::new();
        for p in PixelColor::iter() {
            row.push(p);
        }
        
        let mut rotated: Vec<PixelColor> = row.clone();
        for j in 0..PixelColor::COUNT {
            rotated[(i + j) % PixelColor::COUNT] = row[j];
        }

        pixels.push(rotated);
    }
    return pixels;
}