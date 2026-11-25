use super::*;

// Follow
pub const FOLLOW_OFFSET_X: f32 = 6.0;       // Target position x relative to Young Link's head
pub const FOLLOW_OFFSET_Y: f32 = 4.0;       // Target position y relative to Young Link's head
pub const FOLLOW_EASE: f32 = 0.10;          // Percent of remaining distance Navi moves every frame
pub const BOB_AMPLITUDE: f32 = 0.8;         // Y height of bobbing "animation"
pub const BOB_PERIOD_FRAMES: f32 = 90.0;    // Frames a full bob anim takes

// Facing
pub const FACE_ANGLE: f32 = 65.0;       // Direction facing when flying (0 = camera, 90 = forward)
pub const FACE_EASE: f32 = 0.15;        // Percent of remaining rot Navi rots each frame
pub const FACE_MIN_SPEED: f32 = 0.2;    // Speed of travel required for Navi to face the same direction as Young Link

// Glow
pub const GLOW_REFRESH_FRAMES: i32 = 40;    // Frames glow effect takes to respawn

// Animation
pub const FLAP_RATE: f32 = 0.55;    // Speed of animation (pct of og)

// Tuck
pub const TUCK_EASE: f32 = 0.35;        // Percent of remaining distance Navi moves every frame during tuck
pub const TUCK_DISTANCE: f32 = 1.5;     // Distance from head at which tuck occurs
pub const TUCK_MAX_FRAMES: i32 = 12;    // Frames to force a tuck

// Voice
pub const HELLO_COOLDOWN_FRAMES: i32 = 1200;    // Frames after "hello" a warning line can play
pub const WARN_RANGE: f32 = 40.0;               // Distance from a fighter to trigger the warning line
pub const WARN_DAMAGE_MIN: f32 = 40.0;          // Minimum damage for warning lines
pub const WARN_DAMAGE_MAX: f32 = 120.0;         // Frequency of warning lines stops increasing
pub const WARN_COOLDOWN_LOW: f32 = 1200.0;      // Frames between warning lines at min
pub const WARN_COOLDOWN_HIGH: f32 = 800.0;      // Frames between warning lines at max
pub const WARN_CHANCE_LOW: f32 = 0.5;           // Chance a warning line plays at min
pub const WARN_CHANCE_HIGH: f32 = 0.9;          // Chance a warning line plays at max

// Idle targetting
pub const IDLE_FRAMES: i32 = 300;               // Frames idle for before targetting occurs
pub const IDLE_MOVE_DISTANCE: f32 = 3.0;        // Distance to move to not count as idle
pub const TARGET_OFFSET_Y: f32 = 14.0;          // Height above hip to hover at when targetting
pub const TARGET_EASE: f32 = 0.08;              // Percent of remaining distance Navi moves every frame during target
pub const LISTEN_COOLDOWN_FRAMES: i32 = 500;    // Frames after "listen" before a warning line can play
pub const LISTEN_CHANCE: f32 = 0.5;             // Chance she says "listen" for a given target
pub const LISTEN_ARRIVE_DISTANCE: f32 = 4.0;    // Distance away from target location to say "listen" line

// Misc
pub const SIDE_TAUNT_NAVI_END_FRAME: f32 = 142.0;   // Frame of side taunt to spawn new Navi
pub const ENTRY_NAVI_FRAME: f32 = 60.0;             // Frame of entry anim to spawn Navi
