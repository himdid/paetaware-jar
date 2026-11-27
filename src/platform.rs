#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Left,
    Right,
    Confirm,
    Cancel,
    Step,
}

#[derive(Copy, Clone)]
pub struct EyeRenderState {
    pub w: i16,
    pub h: i16,
    pub radius: i16,
    pub top_lid: i16,
    pub tilt: i16,
    pub bottom_arch: i16,
    pub look_x: i16,
    pub look_y: i16,
    pub swirl_phase: Option<u8>,
    pub show_cheeks: bool,
    pub show_tear: bool,
}

#[derive(Copy, Clone)]
pub struct BubbleRender {
    pub x: i16,
    pub y: i16,
    pub speed: i16,
    pub large: bool,
}

#[derive(Copy, Clone)]
pub struct EchoHud {
    pub round: u8,
    pub pot: u16,
    pub brain_turn: bool,
}

pub struct RenderScene<'a> {
    pub show_stats: bool,
    pub stasis: bool,
    pub hud_visible: bool,
    pub fluid: u8,
    pub lucidity: u8,
    pub static_load: u8,
    pub steps: u32,
    pub credits: u16,
    pub selected_menu: u8,
    pub status_banner: &'a str,
    pub grade_label: &'a str,
    pub echo_hud: Option<EchoHud>,
    pub eye: EyeRenderState,
    pub bubbles: &'a [BubbleRender; 5],
}

pub trait PetPlatform {
    fn render(&mut self, scene: &RenderScene);
    fn poll_input(&mut self) -> Option<InputEvent>;
    fn is_sleep_switch_on(&self) -> bool;
    fn read_uart_byte(&mut self) -> Option<u8>;
    fn write_uart_bytes(&mut self, bytes: &[u8]);
    fn toggle_status_led(&mut self);
    fn delay_tick(&mut self);
}