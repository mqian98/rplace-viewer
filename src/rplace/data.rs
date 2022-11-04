use speedy2d::dimen::Vec2;

use super::pixel::PixelColor;

// TODO: need to change this to custom type? maybe use u16 for size of coordinate but that can be confusing when doing math. 
// if we use u16, then we always have to make sure we dont overflow
pub type RPlaceCoordinate = Vec2;

#[derive(Copy, Clone, Debug)]
pub struct RPlaceDatapoint {
    pub timestamp: u64,
    pub user_id: u32, 
    pub color: PixelColor,
    pub coordinate: RPlaceCoordinate,

    // indicates if a pixel was placed due to moderation
    // will not be true for all pixels placed by mods
    // only is true for swaths of pixels that mods place
    pub is_mod: bool, 
}

#[derive(Copy, Clone, Debug)]
pub struct RPlaceParquetDatapoint {
    pub timestamp: i64,
    pub user_id: i32, 
    pub rgb: u64,
    pub x1: i16,
    pub y1: i16,
    pub x2: i16,
    pub y2: i16,
}

impl TryFrom<&RPlaceParquetDatapoint> for RPlaceDatapoint {
    type Error = ();

    fn try_from(item: &RPlaceParquetDatapoint) -> Result<Self, Self::Error> {
        if (item.x2, item.y2) != (i16::MIN, i16::MIN) {
            return Err(());
        }

        match PixelColor::try_from(item.rgb) {
            Ok(color) => Ok(RPlaceDatapoint {
                timestamp: item.timestamp as u64,
                user_id: item.user_id as u32,
                color,
                coordinate: Vec2::new(item.x1 as f32, item.y1 as f32),
                is_mod: false,
            }),
            _ => Err(())
        }
    }
}

impl TryFrom<RPlaceParquetDatapoint> for RPlaceDatapoint {
    type Error = ();

    fn try_from(item: RPlaceParquetDatapoint) -> Result<Self, Self::Error> {
        (&item).try_into()
    }
}

impl TryFrom<&RPlaceParquetDatapoint> for Vec<RPlaceDatapoint> {
    type Error = ();
    fn try_from(item: &RPlaceParquetDatapoint) -> Result<Self, Self::Error> {
        let mut vector = Vec::new();
        for y in item.y1..item.y2+1 {
            for x in item.x1..item.x2+1 {
                match PixelColor::try_from(item.rgb) {
                    Ok(color) => vector.push(RPlaceDatapoint {
                        timestamp: item.timestamp as u64,
                        user_id: item.user_id as u32,
                        color,
                        coordinate: Vec2::new(x as f32, y as f32),
                        is_mod: false,
                    }),
                    _ => return Err(()),
                }
            }
        }

        Ok(vector)
    }
}

impl TryFrom<RPlaceParquetDatapoint> for Vec<RPlaceDatapoint> {
    type Error = ();
    fn try_from(item: RPlaceParquetDatapoint) -> Result<Self, Self::Error> {
        (&item).try_into()
    }
}