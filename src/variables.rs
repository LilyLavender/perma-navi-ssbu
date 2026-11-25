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
pub const FLAP_RATE: f32 = 0.7;     // Speed of animation (pct of og)

// Misc
pub const SIDE_TAUNT_NAVI_END_FRAME: f32 = 142.0;   // Frame of side taunt to spawn new Navi
pub const ENTRY_NAVI_FRAME: f32 = 60.0;             // Frame of entry anim to spawn Navi
