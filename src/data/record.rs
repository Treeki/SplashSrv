use crate::packets::UID;
use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;
use serde::{Deserialize, Serialize};

/// A specific player's records for a specific course.
/// Keyed on UID, course, season, hole_idx(0-3).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CRecord {
    /// Best score
    pub mx_score: i8,
    /// Lowest score (default is -99)
    pub lowest_score: i8,
    /// Total score (default is 99)
    pub total_score: i8,
    /// Unknown
    pub array: [i8; 18],
    /// Number of rounds
    pub num_rounds: u16,
    /// Maximum round GP
    pub max_gp: u16,
    /// Total round GP
    pub total_gp: u32,
    /// Maximum experience earned
    pub max_exp: u32,
    /// Total experience earned
    pub total_exp: u32,
    /// Longest distance
    pub max_drive: u32,
    /// Longest chip-in distance
    pub max_chipin: u32,
    /// Longest putt distance
    pub max_putt: u32,
    /// Unknown
    pub unk: u32,
}

impl Default for CRecord {
    fn default() -> Self {
        CRecord {
            mx_score: 0,
            lowest_score: -99,
            total_score: 99,
            array: Default::default(),
            num_rounds: 0,
            max_gp: 0,
            total_gp: 0,
            max_exp: 0,
            total_exp: 0,
            max_drive: 0,
            max_chipin: 0,
            max_putt: 0,
            unk: 0,
        }
    }
}

impl DekuRead<'_> for CRecord {
    fn read(input: &BitSlice<u8, Msb0>, ctx: ()) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let (input, mx_score) = i8::read(input, ctx)?;
        let (input, lowest_score) = i8::read(input, ctx)?;
        let (input, total_score) = i8::read(input, ctx)?;
        let (input, array) = <[i8; 18]>::read(input, ctx)?;
        let (input, num_rounds) = u16::read(input, ctx)?;
        let (input, max_gp) = u16::read(input, ctx)?;
        let (input, total_gp) = u32::read(input, ctx)?;
        let (input, max_exp) = u32::read(input, ctx)?;
        let (input, total_exp) = u32::read(input, ctx)?;
        let (input, max_drive) = u32::read(input, ctx)?;
        let (input, val) = u32::read(input, ctx)?;
        let (input, val2) = u32::read(input, ctx)?;

        let record = CRecord {
            mx_score,
            lowest_score,
            total_score,
            array,
            num_rounds,
            max_gp,
            total_gp,
            max_exp: max_exp & 0xFF,
            total_exp,
            max_drive: max_drive & 0x3FFFF,
            max_chipin: val & 0x3FFFF,
            max_putt: val >> 18,
            unk: val2 & 0xFFFF,
        };

        Ok((input, record))
    }
}

impl DekuWrite for CRecord {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: ()) -> Result<(), DekuError> {
        self.mx_score.write(output, ctx)?;
        self.lowest_score.write(output, ctx)?;
        self.total_score.write(output, ctx)?;
        self.array.write(output, ctx)?;
        self.num_rounds.write(output, ctx)?;
        self.max_gp.write(output, ctx)?;
        self.total_gp.write(output, ctx)?;
        self.max_exp.write(output, ctx)?;
        self.total_exp.write(output, ctx)?;
        self.max_drive.write(output, ctx)?;

        let val: u32 = (self.max_chipin & 0x3FFFF) | (self.max_putt << 18);
        val.write(output, ctx)?;

        let val2: u32 = self.unk & 0xFFFF;
        val2.write(output, ctx)?;

        Ok(())
    }
}

/// A specific player's records.
/// Keyed on UID.
#[derive(Debug, Clone, Deserialize, Serialize, DekuRead, DekuWrite)]
pub struct URecord {
    /// Number of rounds played
    pub num_rounds: i16,
    /// Total number of strokes, including putts
    pub total_strokes: i32,
    /// Total number of cup-ins
    pub total_cup_ins: i32,
    /// Maximum distance (in yards)
    pub max_drive: i32,
    /// Longest putt distance (in metres)
    pub max_putt: i16,
    /// Maximum chip-in distance (in yards)
    pub max_chip_in: i32,
    /// Total number of putts
    pub total_putts: i32,
    /// Amount of nice shots (divide by (total_strokes - total_putts) for nice shot percentage)
    pub num_nice_shots: i32,
    /// Number of chip-ins
    pub num_chip_in: i16,
    /// Number of non-cup-in strokes that hit the fairway
    pub num_fairway_keep: i32,
    /// Number of non-cup-in strokes that hit OB
    pub num_ob: i16,
    /// Number of non-cup-in strokes that hit a water hazard
    pub num_water_hazard: i16,
    /// Number of 4-or-more holes
    pub num_4_or_more: u32,
    /// Number of triple bogies
    pub num_triple_bogies: i16,
    /// Number of double bogies
    pub num_double_bogies: i16,
    /// Number of bogies
    pub num_bogies: i16,
    /// Number of pars
    pub num_pars: i32,
    /// Number of birdies
    pub num_birdies: i32,
    /// Number of eagles
    pub num_eagles: i16,
    /// Number of albatrosses
    pub num_albatross: i16,
    /// Number of hole-in-ones
    pub num_hoi: i16,
    /// Total distance (in yards)
    pub total_distance: i32,
    /// Total playtime in seconds
    pub total_playtime: i32,
    // Total number of holes
    pub total_holes: i32,
    /// Highest score
    pub highest_score: i8,
    /// Lowest score
    pub lowest_score: i8,
    /// Total score (divide by total_holes for average score)
    pub total_score: i16,
    /// Number of retirements
    pub num_retirements: i16,
    /// Number of "direct chip-ins"
    pub num_direct_chip_ins: i16,
    /// Number of non-cup-in strokes that hit the rough
    pub num_rough: i32,
    /// Number of non-cup-in strokes that hit the bunker
    pub num_bunker: i32,
    /// Number of obstacle hits
    pub num_obstacle_hits: i32,
    /// Number of pinshots
    pub num_pinshots: i16,
    /// Number of flagshots
    pub num_flagshots: i16,
    /// Total amount of VS battles
    pub total_vs_participation: i32,
    /// Total amount of tournament participation
    pub total_tournament_participation: i32,
    /// Total amount of Quick Battles
    pub total_quick_participation: i32,
    /// Number of consumable items used
    pub num_consumable_item_usage: i32,
    /// ??? Triggers the "chat memorial" photo
    pub x_74: i32,
    /// ???
    pub x_78: i32,
    /// Number of logins
    pub num_logins: i16,
    /// ???
    pub x_7e: i32,
    /// Number of 1st places in a normal tournament
    pub num_1st: i16,
    /// Number of 2nd places in a normal tournament
    pub num_2nd: i16,
    /// Number of 3rd places in a normal tournament
    pub num_3rd: i16,
    /// Number of 1st places in a cafe tournament
    pub num_1st_cafe: i16,
    /// Number of 2nd places in a cafe tournament
    pub num_2nd_cafe: i16,
    /// Number of 3rd places in a cafe tournament
    pub num_3rd_cafe: i16,
    /// Total round GP
    pub total_round_gp: i32,
    /// ???
    pub x_92: [u8; 14],
}

/// Global records (across all players) for a specific course.
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct GCRecord {
    pub course: i8,
    pub season: i8,
    pub unk: i8, // probably -1, 0, 1, 2 ???
    pub max_score: i32,
    pub max_score_uid: UID,
    pub max_score_title: i16,
    pub max_gp: i32,
    pub max_gp_uid: UID,
    pub max_gp_title: i16,
}

/// Global records (across all players) for a specific hole on a course.
#[derive(Debug, Clone, DekuRead, DekuWrite)]
pub struct GHRecord {
    pub score: i8,
    pub score_uid: UID,
    pub score_title: i16,
    pub gp: i32,
    pub gp_uid: UID,
    pub gp_title: i16,
    pub hio_uid: UID,
    pub hio_title: i16,
    pub drive: i16,
    pub drive_uid: UID,
    pub drive_title: i16,
    pub chipin: i16,
    pub chipin_uid: UID,
    pub chipin_title: i16,
    pub putt: i16,
    pub putt_uid: UID,
    pub putt_title: i16,
}
