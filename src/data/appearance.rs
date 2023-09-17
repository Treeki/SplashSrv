use super::CharID;
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;
use serde::{Deserialize, Serialize};

// Assumed to always represent a Player (character type: 0).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Appearance {
    pub character_id: CharID,

    pub head: Option<u16>,
    pub face: Option<u16>,
    pub glasses: Option<u16>,
    pub tops: Option<u16>,
    pub bottoms: Option<u16>,
    pub shoes: Option<u16>,
    pub gloves: Option<u16>,
    pub wing: Option<u16>,
    pub club: Option<u16>,
    pub skirt: Option<u16>,

    pub hair_style: u16,
    pub hair_color: u16,
    pub eye_color: u16,
    pub skin_color: u16,
    pub face_paint: u16,

    pub default_tops: Option<u16>,
    pub default_bottoms: Option<u16>,
    pub default_shoes: Option<u16>,
    pub default_hair_color: u16,
    pub default_eye_color: u16,
    pub default_skin_color: u16,
}

fn unpack_optional(input: u32) -> Option<u16> {
    if (input >= 1) && (input <= 0x3FF) {
        Some((input - 1) as u16)
    } else {
        None
    }
}

fn pack_optional(input: Option<u16>) -> Result<u32, DekuError> {
    match input {
        None => Ok(0),
        Some(num) if (num <= 0x3FE) => Ok((num + 1) as u32),
        _ => Err(DekuError::InvalidParam(
            "Appearance value out of range".to_string(),
        )),
    }
}

impl DekuRead<'_> for Appearance {
    fn read(input: &BitSlice<u8, Msb0>, ctx: ()) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        // Offset 0
        let (input, val) = u32::read(input, ctx)?;

        let character_id = match CharID::from_index((val >> 2) & 0x3F) {
            Some(id) => id,
            None => return Err(DekuError::InvalidParam("Invalid character ID".to_string())),
        };
        let face_paint = ((val >> 8) & 0x3FF) as u16;
        let head = unpack_optional((val >> 18) & 0x3FF);

        // Offset 4
        let (input, val) = u32::read(input, ctx)?;

        let glasses = unpack_optional(val & 0x3FF);
        let tops = unpack_optional((val >> 10) & 0x3FF);
        let bottoms = unpack_optional((val >> 20) & 0x3FF);

        // Offset 8
        let (input, val) = u32::read(input, ctx)?;

        let shoes = unpack_optional(val & 0x3FF);
        let gloves = unpack_optional((val >> 10) & 0x3FF);
        let wing = unpack_optional((val >> 20) & 0x3FF);

        // Offset C
        let (input, val) = u32::read(input, ctx)?;

        let club = unpack_optional(val & 0x3FF);
        let face = unpack_optional((val >> 10) & 0x3FF);
        let skirt = unpack_optional((val >> 20) & 0x3FF);

        // Offset 10 - unused
        let (input, _val) = u32::read(input, ctx)?;

        // Offset 14
        let (input, val) = u32::read(input, ctx)?;

        let hair_style = ((val >> 10) & 0x3FF) as u16;
        let hair_color = ((val >> 20) & 0x3FF) as u16;

        // Offset 18
        let (input, val) = u32::read(input, ctx)?;

        let eye_color = (val & 0xFF) as u16;
        let skin_color = ((val >> 8) & 0xFF) as u16;
        let default_tops = unpack_optional((val >> 16) & 0x3FF);

        // Offset 1C
        let (input, val) = u32::read(input, ctx)?;

        let default_bottoms = unpack_optional(val & 0x3FF);
        let default_shoes = unpack_optional((val >> 10) & 0x3FF);
        let default_hair_color = ((val >> 20) & 0x3FF) as u16;

        // Offset 20
        let (input, val) = u32::read(input, ctx)?;

        let default_eye_color = (val & 0xFF) as u16;
        let default_skin_color = ((val >> 8) & 0xFF) as u16;

        let app = Appearance {
            character_id,
            head,
            face,
            glasses,
            tops,
            bottoms,
            shoes,
            gloves,
            wing,
            club,
            skirt,
            hair_style,
            hair_color,
            eye_color,
            skin_color,
            face_paint,
            default_tops,
            default_bottoms,
            default_shoes,
            default_hair_color,
            default_eye_color,
            default_skin_color,
        };
        Ok((input, app))
    }
}

impl DekuWrite for Appearance {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: ()) -> Result<(), DekuError> {
        // Offset 0
        let val: u32 = (self.character_id.to_index() << 2)
            | ((self.face_paint as u32) << 8)
            | (pack_optional(self.head)? << 18);
        val.write(output, ctx)?;

        // Offset 4
        let val: u32 = pack_optional(self.glasses)?
            | (pack_optional(self.tops)? << 10)
            | (pack_optional(self.bottoms)? << 20);
        val.write(output, ctx)?;

        // Offset 8
        let val: u32 = pack_optional(self.shoes)?
            | (pack_optional(self.gloves)? << 10)
            | (pack_optional(self.wing)? << 20);
        val.write(output, ctx)?;

        // Offset C
        let val: u32 = pack_optional(self.club)?
            | (pack_optional(self.face)? << 10)
            | (pack_optional(self.skirt)? << 20);
        val.write(output, ctx)?;

        // Offset 10 - unused
        let val: u32 = 0;
        val.write(output, ctx)?;

        // Offset 14
        let val: u32 = ((self.hair_style as u32) << 10) | ((self.hair_color as u32) << 20);
        val.write(output, ctx)?;

        // Offset 18
        let val: u32 = (self.eye_color as u32)
            | ((self.skin_color as u32) << 8)
            | (pack_optional(self.default_tops)? << 16);
        val.write(output, ctx)?;

        // Offset 1C
        let val: u32 = pack_optional(self.default_bottoms)?
            | (pack_optional(self.default_shoes)? << 10)
            | ((self.default_hair_color as u32) << 20);
        val.write(output, ctx)?;

        // Offset 20
        let val: u32 = (self.default_eye_color as u32) | ((self.default_skin_color as u32) << 8);
        val.write(output, ctx)?;

        Ok(())
    }
}
