use super::*;

const ZERO: Vector3f = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
const NONE: u32 = 0;

#[derive(Clone, Copy)]
struct NaviState {
    owner: u32,
    active: bool,
    pos: [f32; 2],
    bob_frame: f32,
    facing: f32,        // 0° is towards camera, 90 right, -90 left
    glow_wait: i32,     // Timer for next glow eff spawn
    glow_handle: u32,
}

const NAVI_BLANK: NaviState = NaviState {
    owner: NONE,
    active: false,
    pos: [0.0; 2],
    bob_frame: 0.0,
    facing: 0.0,
    glow_wait: 0,
    glow_handle: 0,
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

// Ret hover location
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

    ArticleModule::change_motion(boma, *FIGHTER_YOUNGLINK_GENERATE_ARTICLE_NAVY, Hash40::new("hover"), false, -1.0);

    let hover = navi_hover_pos(boma);
    state.pos = [hover.x, hover.y];
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
unsafe fn navi_follow(boma: *mut BattleObjectModuleAccessor, navi: *mut BattleObjectModuleAccessor, state: &mut NaviState) {
    // Travel a percentage towards target position
    let hover = navi_hover_pos(boma);
    let speed_x = (hover.x - state.pos[0]) * FOLLOW_EASE;
    state.pos[0] += speed_x;
    state.pos[1] += (hover.y - state.pos[1]) * FOLLOW_EASE;

    // Bob up/down
    state.bob_frame = (state.bob_frame + 1.0) % BOB_PERIOD_FRAMES;
    let bob = (state.bob_frame / BOB_PERIOD_FRAMES * std::f32::consts::TAU).sin() * BOB_AMPLITUDE;

    // Face in direction of travel
    let side = if speed_x.abs() >= FACE_MIN_SPEED {
        speed_x.signum()
    } else {
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

// opff
unsafe extern "C" fn younglink_frame(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    let slot = navi_state(boma);
    if slot.is_null() {
        return;
    }
    let state = &mut *slot;
    let status = StatusModule::status_kind(boma);

    // Deactivate Navi turing taunts
    if status == *FIGHTER_STATUS_KIND_APPEAL {
        state.active = false;
        state.glow_handle = NONE;
        return;
    }

    // Remove Navi on death
    if status == *FIGHTER_STATUS_KIND_DEAD {
        if state.active {
            navi_remove(boma, state);
        }
        return;
    }

    // If Navi doesn't exist, spawn her
    if !state.active
    || navi_boma(boma).is_null() {
        navi_spawn(boma, state);
    }

    // If Navi's boma doesn't exist, ret
    let navi = navi_boma(boma);
    if navi.is_null() {
        return;
    }

    // Navi opff
    navi_glow_update(navi, state);
    navi_follow(boma, navi, state);
}

pub fn install() {
    Agent::new("younglink")
        .set_costume(marked_costumes())
        .on_line(Main, younglink_frame)
        .install();
}
