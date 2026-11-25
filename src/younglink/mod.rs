use super::*;

const ZERO: Vector3f = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
const NONE: u32 = 0;

#[derive(Clone, Copy)]
struct NaviState {
    owner: u32,
    active: bool,
    pos: [f32; 2],
    bob_frame: f32,
    facing: f32,            // 0° is towards camera, 90 right, -90 left
    glow_wait: i32,         // Timer for next glow eff spawn
    glow_handle: u32,
    tuck_frames: i32,       // Tuck timer
    tucked: bool,
    frames: u32,            // Frame counter for rand
    voice_cooldown: i32,    // Frames until next voice line can play
    greet: bool,            // When set to true, she'll say the "hello" line and then set back to false
    idle_pos: [f32; 2],     // YL position (if it hasn't moved at least IDLE_MOVE_DISTANCE units)
    idle_frames: i32,       // ^ Frames YL has stayed at idle_pos for
    target: u32,            // boid of fighter Navi is hovering over
    listen_pending: bool,   // True if Navi should say "listen" when reaching target
}

const NAVI_BLANK: NaviState = NaviState {
    owner: NONE,
    active: false,
    pos: [0.0; 2],
    bob_frame: 0.0,
    facing: 0.0,
    glow_wait: 0,
    glow_handle: 0,
    tuck_frames: 0,
    tucked: false,
    frames: 0,
    voice_cooldown: 0,
    greet: false,
    idle_pos: [0.0; 2],
    idle_frames: 0,
    target: NONE,
    listen_pending: false,
};

static mut NAVI: [NaviState; 8] = [NAVI_BLANK; 8];

// Get slot number in NAVI array for given Young Link boma
// Sets slot if one hasn't been claimed yet
unsafe fn navi_state(boma: *mut BattleObjectModuleAccessor) -> *mut NaviState {
    let id = (*boma).battle_object_id;
    let slots = std::ptr::addr_of_mut!(NAVI) as *mut NaviState;
    
    // Get slot num
    for i in 0..8 {
        if (*slots.add(i)).owner == id {
            return slots.add(i);
        }
    }

    // Set slot num
    for i in 0..8 {
        let slot = slots.add(i);
        if (*slot).owner == NONE || !sv_battle_object::is_active((*slot).owner) {
            *slot = NAVI_BLANK;
            (*slot).owner = id;
            return slot;
        }
    }

    // No available slots
    std::ptr::null_mut()
}

// fixes an annoying crash
#[repr(C, align(16))]
struct JointOut {
    v: Vector3f,
    _pad: f32,
}

// Returns position of specified joint
// Done to handle padding with JointOut
unsafe fn joint_position(boma: *mut BattleObjectModuleAccessor, joint: &str) -> Vector3f {
    let mut out = JointOut { v: Vector3f { x: 0.0, y: 0.0, z: 0.0 }, _pad: 0.0 };
    ModelModule::joint_global_position(boma, Hash40::new(joint), &mut out.v, true);
    out.v
}

// Gets Navi boma for given Young Link boma
unsafe fn navi_boma(boma: *mut BattleObjectModuleAccessor) -> *mut BattleObjectModuleAccessor {
    // Ret null if no Navi exists for given YL
    if !ArticleModule::is_exist(boma, *FIGHTER_YOUNGLINK_GENERATE_ARTICLE_NAVY) {
        return std::ptr::null_mut();
    }

    // Ensure article exists
    let article = ArticleModule::get_article(boma, *FIGHTER_YOUNGLINK_GENERATE_ARTICLE_NAVY);
    if article.is_null() {
        return std::ptr::null_mut();
    }

    // Make sure battle object is active
    let id = smash::app::lua_bind::Article::get_battle_object_id(article) as u32;
    if !sv_battle_object::is_active(id) {
        return std::ptr::null_mut();
    }

    // Ret boma
    sv_battle_object::module_accessor(id)
}

// Summon glow effect
unsafe fn navi_glow(navi: *mut BattleObjectModuleAccessor) -> u32 {
    EffectModule::req_follow(
        navi, Hash40::new("younglink_navy"), Hash40::new("rot"),
        &ZERO, &ZERO, 1.0, true, 0, 0, 0, 0, 0, true, true
    ) as u32
}

// Ret hover location near YL
unsafe fn navi_hover_pos(boma: *mut BattleObjectModuleAccessor) -> Vector3f {
    let head = joint_position(boma, "head");
    Vector3f {
        x: head.x - FOLLOW_OFFSET_X * PostureModule::lr(boma),
        y: head.y + FOLLOW_OFFSET_Y,
        z: head.z,
    }
}

// remove_exist Navi
unsafe fn navi_remove(boma: *mut BattleObjectModuleAccessor, state: &mut NaviState) {
    if ArticleModule::is_exist(boma, *FIGHTER_YOUNGLINK_GENERATE_ARTICLE_NAVY) {
        ArticleModule::remove_exist(boma, *FIGHTER_YOUNGLINK_GENERATE_ARTICLE_NAVY, ArticleOperationTarget(*ARTICLE_OPE_TARGET_ALL));
    }
    state.active = false;
    state.glow_handle = NONE;
}

// Summon Navi
unsafe fn navi_spawn(boma: *mut BattleObjectModuleAccessor, state: &mut NaviState) {
    navi_remove(boma, state);
    ArticleModule::generate_article(boma, *FIGHTER_YOUNGLINK_GENERATE_ARTICLE_NAVY, false, -1);
    let navi = navi_boma(boma);
    if navi.is_null() {
        return;
    }

    // NECESSARY SO SHE'S NOT STUCK AT YL'S ORIGIN!!!!!
    LinkModule::remove_model_constraint(navi, true);

    // Motion
    ArticleModule::change_motion(boma, *FIGHTER_YOUNGLINK_GENERATE_ARTICLE_NAVY, Hash40::new("hover"), false, -1.0);
    MotionModule::set_rate(navi, FLAP_RATE);

    // Init NaviState
    let head = joint_position(boma, "head");
    state.pos = [head.x, head.y];
    state.facing = FACE_ANGLE * PostureModule::lr(boma);
    state.glow_wait = 0;
    state.active = true;
}

// "opff" for glow, handles when a new one will get summonned and summons it
unsafe fn navi_glow_update(navi: *mut BattleObjectModuleAccessor, state: &mut NaviState) {
    // Tick glow_wait timer
    if state.glow_wait > 0 {
        state.glow_wait -= 1;
    }

    // ret if timer isn't finished and effect exists
    if state.glow_wait > 0
    && state.glow_handle != NONE
    && EffectModule::is_exist_effect(navi, state.glow_handle) {
        return;
    }

    // Spawn new effect
    state.glow_handle = navi_glow(navi);
    state.glow_wait = GLOW_REFRESH_FRAMES;
}

// Main follow logic
unsafe fn navi_follow(boma: *mut BattleObjectModuleAccessor, navi: *mut BattleObjectModuleAccessor, state: &mut NaviState, hover: Vector3f, ease: f32) {
    // Travel a percentage towards target position
    let speed_x = (hover.x - state.pos[0]) * ease;
    state.pos[0] += speed_x;
    state.pos[1] += (hover.y - state.pos[1]) * ease;

    // Bob up/down
    state.bob_frame = (state.bob_frame + 1.0) % BOB_PERIOD_FRAMES;
    let bob = (state.bob_frame / BOB_PERIOD_FRAMES * std::f32::consts::TAU).sin() * BOB_AMPLITUDE;

    // Facing direction
    let side = if speed_x.abs() >= FACE_MIN_SPEED {
        // If moving, use direction of travel
        speed_x.signum()
    } else if state.target != NONE {
        // If targetting opponent, face towards YL
        (PostureModule::pos_x(boma) - state.pos[0]).signum()
    } else {
        // Follow facing of YL
        PostureModule::lr(boma)
    };
    state.facing += (FACE_ANGLE * side - state.facing) * FACE_EASE;

    // Set rot
    let turn = state.facing - 90.0 * PostureModule::lr(navi);
    PostureModule::set_rot(navi, &Vector3f { x: 0.0, y: turn, z: 0.0 }, 0);

    // Set pos
    let pos = Vector3f { x: state.pos[0], y: state.pos[1] + bob, z: hover.z };
    PostureModule::set_pos(navi, &pos);
}

// Play sound effect w randomized pitch
unsafe fn play_navi_vc(boma: *mut BattleObjectModuleAccessor, state: &mut NaviState, line: &str, cooldown: i32) {
    // ret if voice is off or cooldown is active
    if !VOICE_ENABLED
    || state.voice_cooldown > 0 {
        return;
    }

    let se = format!("se_younglink_navi_{}{:02}", line, state.frames % 5 + 1);
    SoundModule::play_se(boma, Hash40::new(&se), true, false, false, false, smash::app::enSEType(0));
    state.voice_cooldown = cooldown;
}

// Custom random function
// NATIVE sv_math::rand IS NOT WIFI SAFE!!!
fn rand(state: &NaviState, salt: u32) -> f32 {
    ((state.frames.wrapping_add(salt).wrapping_mul(2654435761) >> 16) % 1000) as f32 / 1000.0
}

// Play warning lines
unsafe fn navi_warn(boma: *mut BattleObjectModuleAccessor, state: &mut NaviState) {
    // ret if damage is too low, in cooldown, or no fighters are nearby
    let damage = DamageModule::damage(boma, 0);
    if damage < WARN_DAMAGE_MIN
    || state.voice_cooldown > 0
    || !fighter_nearby(boma, WARN_RANGE) {
        return;
    }

    // Normalize range to increase warn prob to 0.0 - 1.0
    let danger = ((damage - WARN_DAMAGE_MIN) / (WARN_DAMAGE_MAX - WARN_DAMAGE_MIN)).min(1.0);

    // Pick voice line
    let hey = 2.0 * (1.0 - danger);
    let look = 0.5;
    let watchout = 0.5 + 3.0 * danger;
    let roll = rand(state, 0) * (hey + look + watchout);
    let line = if roll < hey {
        "hey"
    } else if roll < hey + look {
        "look"
    } else {
        "watchout"
    };

    // Roll chance to play line, reset cooldown
    let cooldown = WARN_COOLDOWN_LOW + (WARN_COOLDOWN_HIGH - WARN_COOLDOWN_LOW) * danger;
    let chance = WARN_CHANCE_LOW + (WARN_CHANCE_HIGH - WARN_CHANCE_LOW) * danger;
    if rand(state, 1998) < chance {
        play_navi_vc(boma, state, line, cooldown as i32);
    } else {
        state.voice_cooldown = cooldown as i32;
    }
}

// ret true if fighter can be hovered over
// false if: id is 0, is inactive, dead, or invisible
unsafe fn target_hoverable(id: u32) -> bool {
    // false if id is 0 or is inactive
    if id == NONE || !sv_battle_object::is_active(id) {
        return false;
    }

    // false if dead or invisible
    let f_boma = sv_battle_object::module_accessor(id);
    StatusModule::status_kind(f_boma) != *FIGHTER_STATUS_KIND_DEAD
    && VisibilityModule::is_visible(f_boma)
}

// ret id of the fighter nearest to Young Link
unsafe fn nearest_fighter(boma: *mut BattleObjectModuleAccessor) -> u32 {
    let self_id = (*boma).battle_object_id;
    let mut best = NONE;
    let mut best_dist = f32::MAX;
    for i in 0..lua_bind::FighterManager::total_fighter_num(singletons::FighterManager()) {
        // Ignore if fighter isn't hoverable
        let id = Fighter::get_id_from_entry_id(i);
        if id == self_id || !target_hoverable(id) {
            continue;
        }
        
        // Calc distance
        let f_boma = sv_battle_object::module_accessor(id);
        let dx = PostureModule::pos_x(f_boma) - PostureModule::pos_x(boma);
        let dy = PostureModule::pos_y(f_boma) - PostureModule::pos_y(boma);
        let dist = (dx * dx + dy * dy).sqrt();

        // Set best_distance if applicable
        if dist < best_dist {
            best = id;
            best_dist = dist;
        }
    }
    best
}

// Track idle_pos and idle_frames
unsafe fn idle_update(boma: *mut BattleObjectModuleAccessor, state: &mut NaviState) {
    // Distance
    let dx = PostureModule::pos_x(boma) - state.idle_pos[0];
    let dy = PostureModule::pos_y(boma) - state.idle_pos[1];

    // Statuses
    let status = StatusModule::status_kind(boma);
    let special_statuses = [
        *FIGHTER_STATUS_KIND_SPECIAL_N,
        *FIGHTER_STATUS_KIND_SPECIAL_S,
        *FIGHTER_STATUS_KIND_SPECIAL_HI,
        *FIGHTER_STATUS_KIND_SPECIAL_LW,
        *FIGHTER_LINK_STATUS_KIND_SPECIAL_S2,
        *FIGHTER_LINK_STATUS_KIND_SPECIAL_HI_HOLD,
        *FIGHTER_LINK_STATUS_KIND_SPECIAL_HI_END,
        *FIGHTER_LINK_STATUS_KIND_SPECIAL_LW_BLAST,
    ];

    // If doing special or moved enough, reset idle timer
    if special_statuses.contains(&status)
    || (dx * dx + dy * dy).sqrt() > IDLE_MOVE_DISTANCE {
        state.idle_pos = [PostureModule::pos_x(boma), PostureModule::pos_y(boma)];
        state.idle_frames = 0;
        state.target = NONE;
    } else {
        state.idle_frames += 1;
    }
}

// Handle hovering opponents during idle
unsafe fn navi_idle_target(boma: *mut BattleObjectModuleAccessor, state: &mut NaviState) -> Vector3f {
    // Set state.target with nearest fighter
    if state.target == NONE
    && state.idle_frames >= IDLE_FRAMES {
        state.target = nearest_fighter(boma);
        state.listen_pending = state.target != NONE
                            && VOICE_ENABLED
                            && rand(state, 0) < LISTEN_CHANCE;
    }

    // If target isn't hoverable, reset idle_frames & hover YL instead
    if state.target != NONE
    && !target_hoverable(state.target) {
        state.target = NONE;
        state.idle_frames = 0;
    }
    if state.target == NONE {
        return navi_hover_pos(boma);
    }
    let hip = joint_position(sv_battle_object::module_accessor(state.target), "hip");
    let point = Vector3f { x: hip.x, y: hip.y + TARGET_OFFSET_Y, z: hip.z };

    // Play "listen" line once she's close enough to the target
    if state.listen_pending {
        let dx = point.x - state.pos[0];
        let dy = point.y - state.pos[1];
        if (dx * dx + dy * dy).sqrt() <= LISTEN_ARRIVE_DISTANCE {
            state.listen_pending = false;
            let se = format!("se_younglink_navi_listen{:02}", state.frames % 5 + 1);
            SoundModule::play_se(boma, Hash40::new(&se), true, false, false, false, smash::app::enSEType(0));
            state.voice_cooldown = state.voice_cooldown.max(LISTEN_COOLDOWN_FRAMES);
        }
    }
    point
}

// ret true if a fighter is within range of Young Link
unsafe fn fighter_nearby(boma: *mut BattleObjectModuleAccessor, range: f32) -> bool {
    let self_id = (*boma).battle_object_id;
    for i in 0..lua_bind::FighterManager::total_fighter_num(singletons::FighterManager()) {
        // Ignore if fighter isn't hoverable
        let id = Fighter::get_id_from_entry_id(i);
        if id == self_id
        || !target_hoverable(id) {
            continue;
        }
        
        // ret true if in spcified range
        let f_boma = sv_battle_object::module_accessor(id);
        let dx = PostureModule::pos_x(f_boma) - PostureModule::pos_x(boma);
        let dy = PostureModule::pos_y(f_boma) - PostureModule::pos_y(boma);
        if (dx * dx + dy * dy).sqrt() <= range {
            return true;
        }
    }

    false
}

// ret true if in tumble
unsafe fn is_tumble(status: i32) -> bool {
    [
        *FIGHTER_STATUS_KIND_DAMAGE_FLY,
        *FIGHTER_STATUS_KIND_DAMAGE_FLY_ROLL,
        *FIGHTER_STATUS_KIND_DAMAGE_FLY_METEOR,
        *FIGHTER_STATUS_KIND_DAMAGE_FLY_REFLECT_LR,
        *FIGHTER_STATUS_KIND_DAMAGE_FLY_REFLECT_U,
        *FIGHTER_STATUS_KIND_DAMAGE_FLY_REFLECT_D,
        *FIGHTER_STATUS_KIND_DAMAGE_FALL,
    ].contains(&status)
}

// True while side taunt is using Navi
unsafe fn side_taunt_has_navi(boma: *mut BattleObjectModuleAccessor) -> bool {
    let motion = MotionModule::motion_kind(boma);
    (
        motion == hash40("appeal_s_l")
        || motion == hash40("appeal_s_r")
    ) && MotionModule::frame(boma) < SIDE_TAUNT_NAVI_END_FRAME
}

// ret true when Navi shouldn't be out
unsafe fn navi_hidden(boma: *mut BattleObjectModuleAccessor, status: i32) -> bool {
    // Dead
    if status == *FIGHTER_STATUS_KIND_DEAD {
        return true;
    }

    // Final Smash
    if status == *FIGHTER_STATUS_KIND_FINAL
    || status == *FIGHTER_LINK_STATUS_KIND_FINAL_DASH
    || status == *FIGHTER_LINK_STATUS_KIND_FINAL_COMBO {
        return true;
    }

    // Results screen
    if status == *FIGHTER_STATUS_KIND_WIN
    || status == *FIGHTER_STATUS_KIND_LOSE {
        return true;
    }

    // Before allowed entry frame
    if status == *FIGHTER_STATUS_KIND_ENTRY
    && MotionModule::frame(boma) < ENTRY_NAVI_FRAME {
        return true;
    }

    // Young Link isn't visible
    !VisibilityModule::is_visible(boma)
}

// opff
unsafe extern "C" fn younglink_frame(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let slot = navi_state(boma);
    if slot.is_null() {
        return;
    }
    let state = &mut *slot;
    let status = StatusModule::status_kind(boma);

    // Tick timers/cooldowns
    state.frames = state.frames.wrapping_add(1);
    if state.voice_cooldown > 0 {
        state.voice_cooldown -= 1;
    }
    idle_update(boma, state);

    // Deactivate Navi during side taunt
    if status == *FIGHTER_STATUS_KIND_APPEAL
    && side_taunt_has_navi(boma) {
        if state.active {
            navi_remove(boma, state);
        }
        state.tucked = false;
        state.idle_frames = 0;
        state.target = NONE;
        return;
    }

    // Remove Navi when she shouldn't be out
    if navi_hidden(boma, status) {
        if state.active {
            navi_remove(boma, state);
        }
        if status == *FIGHTER_STATUS_KIND_ENTRY {
            state.greet = true;
        }
        state.tucked = false;
        state.idle_frames = 0;
        state.target = NONE;
        return;
    }

    // Tuck in tumble
    let is_tumble = is_tumble(status);
    if is_tumble {
        state.idle_frames = 0;
        state.target = NONE;
    }
    if !is_tumble {
        if state.tucked {
            state.tucked = false;
            SoundModule::play_se(boma, Hash40::new("se_younglink_navi_out"), true, false, false, false, smash::app::enSEType(0));
        }
        state.tuck_frames = 0;
    } else if state.tucked {
        return;
    } else if !state.active {
        state.tucked = true;
        return;
    }

    // If Navi doesn't exist, spawn her
    if !state.active
    || navi_boma(boma).is_null() {
        navi_spawn(boma, state);
        if state.greet {
            state.greet = false;
            play_navi_vc(boma, state, "hello", HELLO_COOLDOWN_FRAMES);
        }
    }

    // If Navi's boma doesn't exist, ret
    let navi = navi_boma(boma);
    if navi.is_null() {
        return;
    }

    // Navi "opff"
    navi_glow_update(navi, state);
    if !is_tumble {
        // Follow either yl or opponent
        let hover = navi_idle_target(boma, state);
        let ease = if state.target != NONE { TARGET_EASE } else { FOLLOW_EASE };
        navi_follow(boma, navi, state, hover, ease);

        // Play warn line when opponent is close
        navi_warn(boma, state);
        return;
    }

    // Tuck
    let head = joint_position(boma, "head");
    navi_follow(boma, navi, state, Vector3f { x: head.x, y: head.y, z: head.z }, TUCK_EASE);
    state.tuck_frames += 1;
    let dist = ((head.x - state.pos[0]).powi(2) + (head.y - state.pos[1]).powi(2)).sqrt();
    if dist <= TUCK_DISTANCE
    || state.tuck_frames >= TUCK_MAX_FRAMES {
        navi_remove(boma, state);
        state.tucked = true;
        SoundModule::play_se(boma, Hash40::new("se_younglink_navi_in"), true, false, false, false, smash::app::enSEType(0));
    }
}

pub fn install() {
    Agent::new("younglink")
        .set_costume(marked_costumes())
        .on_line(Main, younglink_frame)
        .install();
}
