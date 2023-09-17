use deku::bitvec::{BitSlice, BitVec, Msb0};
use deku::prelude::*;

use crate::packets::Outcome;

/// The result of a game.
#[derive(Debug, Clone)]
pub struct GameReport {
    pub outcome: Outcome,
    pub num_strokes: u32,
    pub num_cup_ins: u32,
    pub maximum_distance: u32,        // max_drive?
    pub longest_putt_distance: u32,   // max_putt?
    pub maximum_tip_in_distance: u32, // max_chip_in?
    pub num_putts: u32,
    pub num_nice_shots: u32,
    pub num_tip_ins: u32,
    pub num_fairway_keep: u32,
    pub num_ob: u32,
    pub num_water_hazard: u32,
    pub num_4_or_more: u32,
    pub num_t_bogeys: u32,
    pub num_d_bogeys: u32,
    pub num_bogeys: u32,
    pub num_pars: u32,
    pub num_birdies: u32,
    pub num_eagles: u32,
    pub num_albatross: u32,
    pub num_hole_in_ones: u32,
    pub num_total_distance: u32,
    pub play_time: u32, // in seconds
    pub obtained_gp_round: u32,
    pub obtained_gp_all: u32,
    pub acquired_experience: u32, // not used
    pub num_direct_tip_ins: u32,
    pub num_rough: u32,
    pub num_bunkers: u32,
    pub num_obstacle_hits: u32,
    pub num_pinshots: u32,
    pub num_flag_wraps: u32,
    pub num_consumable_item_usage: u32,
    pub longest_tee_shot: u32, // max_tee_shot?
    pub total_putt_distance_at_cup_in: u32,
    pub num_top_or_backspin_successes: u32,
    pub num_fade_or_draw_usage: u32,
    pub num_clubs_used: u32,
    pub num_times_cooperating_with_caddy: u32,
    pub num_special_shots_used: u32,
    pub vs_rank: u32,
    pub halfway_score: i8, // (for competition extra prize)
    pub score: i8,
    pub holes: [HoleReport; 18],
}

impl DekuRead<'_> for GameReport {
    fn read(input: &BitSlice<u8, Msb0>, ctx: ()) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let (input, val1) = u32::read(input, ctx)?;
        let (input, val2) = u32::read(input, ctx)?;
        let (input, val3) = u32::read(input, ctx)?;
        let (input, val4) = u32::read(input, ctx)?;
        let (input, val5) = u32::read(input, ctx)?;
        let (input, val6) = u32::read(input, ctx)?;
        let (input, val7) = u32::read(input, ctx)?;
        let (input, val8) = u32::read(input, ctx)?;
        let (input, val9) = u32::read(input, ctx)?;
        let (input, val10) = u32::read(input, ctx)?;
        let (input, val11) = u32::read(input, ctx)?;
        let (input, val12) = u32::read(input, ctx)?;
        let (input, halfway_score) = i8::read(input, ctx)?;
        let (input, score) = i8::read(input, ctx)?;
        let (input, holes) = <[HoleReport; 18]>::read(input, ctx)?;

        let report = GameReport {
            outcome: Outcome::from_u32(val1 & 7),
            num_strokes: (val1 >> 3) & 0xFF,
            num_cup_ins: (val1 >> 11) & 0x1F,
            maximum_distance: val2 & 0x3FFFF,
            longest_putt_distance: val2 >> 18,
            maximum_tip_in_distance: val3 & 0x3FFFF,
            num_putts: (val3 >> 18) & 0x7F,
            num_nice_shots: val3 >> 25,
            num_tip_ins: val4 & 0x1F,
            num_fairway_keep: (val4 >> 5) & 0xFF,
            num_ob: (val4 >> 13) & 0x3F,
            num_water_hazard: (val4 >> 19) & 0x3F,
            num_4_or_more: (val4 >> 25) & 0x1F,
            num_t_bogeys: val5 & 0x1F,
            num_d_bogeys: (val5 >> 5) & 0x1F,
            num_bogeys: (val5 >> 10) & 0x1F,
            num_pars: (val5 >> 15) & 0x1F,
            num_birdies: (val5 >> 20) & 0x1F,
            num_eagles: (val5 >> 25) & 0xF,
            num_albatross: val5 >> 29,
            num_hole_in_ones: val6 & 7,
            num_total_distance: (val6 >> 3) & 0x3FFFF,
            play_time: val7 & 0x3FFF,
            obtained_gp_round: (val7 >> 14) & 0x7FFF,
            obtained_gp_all: val8 & 0x7FFF,
            acquired_experience: (val8 >> 15) & 0xFF,
            num_direct_tip_ins: (val8 >> 23) & 0x1F,
            num_rough: val9 & 0xFF,
            num_bunkers: (val9 >> 8) & 0x7F,
            num_obstacle_hits: (val9 >> 15) & 0x7F,
            num_pinshots: (val9 >> 22) & 0x7F,
            num_flag_wraps: val9 >> 27,
            num_consumable_item_usage: val10 & 0xFF,
            longest_tee_shot: (val10 >> 8) & 0x3FFFF,
            total_putt_distance_at_cup_in: val11 & 0xFFFF,
            num_top_or_backspin_successes: (val11 >> 16) & 0x7F,
            num_fade_or_draw_usage: (val11 >> 23) & 0x7F,
            num_clubs_used: val12 & 0xF,
            num_times_cooperating_with_caddy: (val12 >> 4) & 0x7F,
            num_special_shots_used: (val12 >> 11) & 0x1F,
            vs_rank: (val12 >> 16) & 7,
            halfway_score,
            score,
            holes,
        };

        Ok((input, report))
    }
}

impl DekuWrite for GameReport {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: ()) -> Result<(), DekuError> {
        let val1: u32 = self.outcome.to_u32()
            | ((self.num_strokes & 0xFF) << 3)
            | ((self.num_cup_ins & 0x1F) << 11);

        let val2: u32 = (self.maximum_distance & 0x3FFFF) | (self.longest_putt_distance << 18);

        let val3: u32 = (self.maximum_tip_in_distance & 0x3FFFF)
            | ((self.num_putts & 0x7F) << 18)
            | self.num_nice_shots << 25;

        let val4: u32 = (self.num_tip_ins & 0x1F)
            | ((self.num_fairway_keep & 0xFF) << 5)
            | ((self.num_ob & 0x3F) << 13)
            | ((self.num_water_hazard & 0x3F) << 19)
            | ((self.num_4_or_more & 0x1F) << 25);

        let val5: u32 = (self.num_t_bogeys & 0x1F)
            | ((self.num_d_bogeys & 0x1F) << 5)
            | ((self.num_bogeys & 0x1F) << 10)
            | ((self.num_pars & 0x1F) << 15)
            | ((self.num_birdies & 0x1F) << 20)
            | ((self.num_eagles & 0xF) << 25)
            | (self.num_albatross << 29);

        let val6: u32 = (self.num_hole_in_ones & 7) | ((self.num_total_distance & 0x3FFFF) << 3);

        let val7: u32 = (self.play_time & 0x3FFF) | ((self.obtained_gp_round & 0x7FFF) << 14);

        let val8: u32 = (self.obtained_gp_all & 0x7FFF)
            | ((self.acquired_experience & 0xFF) << 15)
            | ((self.num_direct_tip_ins & 0x1F) << 23);

        let val9: u32 = (self.num_rough & 0xFF)
            | ((self.num_bunkers & 0x7F) << 8)
            | ((self.num_obstacle_hits & 0x7F) << 15)
            | ((self.num_pinshots & 0x7F) << 22)
            | (self.num_flag_wraps << 27);

        let val10: u32 =
            (self.num_consumable_item_usage & 0xFF) | ((self.longest_tee_shot & 0x3FFFF) << 8);

        let val11: u32 = (self.total_putt_distance_at_cup_in & 0xFFFF)
            | ((self.num_top_or_backspin_successes & 0x7F) << 16)
            | ((self.num_fade_or_draw_usage & 0x7F) << 23);

        let val12: u32 = (self.num_clubs_used & 0xF)
            | ((self.num_times_cooperating_with_caddy & 0x7F) << 4)
            | ((self.num_special_shots_used & 0x1F) << 11)
            | ((self.vs_rank & 7) << 16);

        val1.write(output, ctx)?;
        val2.write(output, ctx)?;
        val3.write(output, ctx)?;
        val4.write(output, ctx)?;
        val5.write(output, ctx)?;
        val6.write(output, ctx)?;
        val7.write(output, ctx)?;
        val8.write(output, ctx)?;
        val9.write(output, ctx)?;
        val10.write(output, ctx)?;
        val11.write(output, ctx)?;
        val12.write(output, ctx)?;
        self.halfway_score.write(output, ctx)?;
        self.score.write(output, ctx)?;
        self.holes.write(output, ctx)?;

        Ok(())
    }
}

/// The result of a specific hole during a round.
#[derive(Debug, Clone)]
pub struct HoleReport {
    /// Score for this hole
    pub score: i8,
    /// Amount of GP gained
    pub gp: u32,
    /// Was this a Hole in One?
    pub is_hole_in_one: bool,
    /// Maximum flight distance (1/100y)
    pub maximum_flight_distance: u32,
    /// Longest chip-in (1/100y)
    pub longest_chip_in: u32,
    /// Longest putt (1/100y)
    pub longest_putt: u32,
    /// What happened
    pub outcome: Outcome,
    /// In point rules
    pub vs_point: u32,
}

impl DekuRead<'_> for HoleReport {
    fn read(input: &BitSlice<u8, Msb0>, ctx: ()) -> Result<(&BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let (input, score) = i8::read(input, ctx)?;
        let (input, val1) = u32::read(input, ctx)?;
        let (input, val2) = u32::read(input, ctx)?;
        let (input, val3) = u32::read(input, ctx)?;

        let report = HoleReport {
            score,
            gp: val1 & 0xFFF,
            is_hole_in_one: (val1 & 0x1000) != 0,
            maximum_flight_distance: (val1 >> 13) & 0x3FFFF,
            longest_chip_in: val2 & 0x3FFFF,
            longest_putt: val2 >> 18,
            outcome: Outcome::from_u32(val3 & 7),
            vs_point: (val3 >> 3) & 7,
        };

        Ok((input, report))
    }
}

impl DekuWrite for HoleReport {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: ()) -> Result<(), DekuError> {
        let val1: u32 = (self.gp & 0xFFF)
            | if self.is_hole_in_one { 0x1000 } else { 0 }
            | ((self.maximum_flight_distance & 0x3FFFF) << 13);

        let val2: u32 = (self.longest_chip_in & 0x3FFFF) | (self.longest_putt << 18);

        let val3: u32 = self.outcome.to_u32() | ((self.vs_point & 7) << 3);

        self.score.write(output, ctx)?;
        val1.write(output, ctx)?;
        val2.write(output, ctx)?;
        val3.write(output, ctx)?;

        Ok(())
    }
}
