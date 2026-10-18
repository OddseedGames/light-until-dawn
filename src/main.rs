use macroquad::audio::{self, PlaySoundParams, Sound};
use macroquad::prelude::*;
use oddseed_until_dawn::game::{Controls, Difficulty, Game, Phase, FUEL_SITES, NIGHT_SECONDS};
use std::sync::atomic::{AtomicBool, Ordering};

const W: f32 = 1280.0;
const H: f32 = 900.0;
const FONT_SIZE: u16 = 48;
const BG: Color = Color::new(0.051, 0.078, 0.090, 1.0);
const PANEL: Color = Color::new(0.078, 0.114, 0.125, 1.0);
const LINE: Color = Color::new(0.157, 0.208, 0.216, 1.0);
const INK: Color = Color::new(0.875, 0.902, 0.867, 1.0);
const MUTED: Color = Color::new(0.475, 0.565, 0.549, 1.0);
const LIME: Color = Color::new(0.804, 0.867, 0.612, 1.0);
const AMBER: Color = Color::new(0.863, 0.647, 0.384, 1.0);
const RED: Color = Color::new(0.886, 0.459, 0.427, 1.0);
const BLUE: Color = Color::new(0.529, 0.702, 0.765, 1.0);
static PAUSE_REQUEST: AtomicBool = AtomicBool::new(false);

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn oddseed_ready();
    fn oddseed_status(
        phase: u32,
        elapsed: f32,
        fuel: f32,
        temperature: f32,
        integrity: f32,
        signal: f32,
        position: f32,
        outside_seconds: f32,
        carrying: u32,
        stamina: f32,
        difficulty: u32,
        searches_left: u32,
    );
}

#[no_mangle]
pub extern "C" fn oddseed_pause() {
    PAUSE_REQUEST.store(true, Ordering::Relaxed);
}

fn configuration() -> Conf {
    Conf {
        window_title: "Свет до рассвета · OddseedGames".to_owned(),
        window_width: 1280,
        window_height: 900,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

fn opacity(mut color: Color, alpha: f32) -> Color {
    color.a = alpha;
    color
}

fn prepare_font(font: &Font, dpi: f32) {
    let characters: Vec<char> = (' '..='~')
        .chain('А'..='я')
        .chain("Ёё°·×—−♪".chars())
        .collect();
    // Grow the atlas before drawing so queued text never references a replaced texture.
    font.populate_font_cache(&characters, (FONT_SIZE as f32 * dpi).ceil() as u16);
}

fn txt(font: &Font, content: &str, x: f32, y: f32, size: u16, color: Color) {
    draw_text_ex(
        content,
        x,
        y,
        TextParams {
            font: Some(font),
            font_size: FONT_SIZE,
            font_scale: size as f32 / FONT_SIZE as f32,
            color,
            ..Default::default()
        },
    );
}

fn text_width(font: &Font, content: &str, size: u16) -> f32 {
    measure_text(
        content,
        Some(font),
        FONT_SIZE,
        size as f32 / FONT_SIZE as f32,
    )
    .width
}

fn right(font: &Font, content: &str, x: f32, y: f32, size: u16, color: Color) {
    let width = text_width(font, content, size);
    txt(font, content, x - width, y, size, color);
}

fn centered(font: &Font, content: &str, x: f32, y: f32, size: u16, color: Color) {
    let width = text_width(font, content, size);
    txt(font, content, x - width / 2.0, y, size, color);
}

fn box_fill(rect: Rect, fill: Color, stroke: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, stroke);
}

fn bar(x: f32, y: f32, width: f32, height: f32, ratio: f32, color: Color) {
    draw_rectangle(x, y, width, height, rgba(39, 52, 55, 255));
    draw_rectangle(x, y, width * ratio.clamp(0.0, 1.0), height, color);
}

fn keycap(font: &Font, key: &str, x: f32, y: f32, color: Color) {
    box_fill(Rect::new(x, y, 25.0, 25.0), rgba(23, 34, 37, 255), LINE);
    centered(font, key, x + 12.5, y + 17.0, 13, color);
}

fn in_rect(point: Option<Vec2>, rect: Rect) -> bool {
    point.is_some_and(|point| rect.contains(point))
}

fn screen_to_game(x: f32, y: f32) -> Vec2 {
    vec2(x / screen_width() * W, y / screen_height() * H)
}

fn pointer_press() -> Option<Vec2> {
    if is_mouse_button_pressed(MouseButton::Left) {
        let (x, y) = mouse_position();
        Some(screen_to_game(x, y))
    } else {
        touches()
            .iter()
            .find(|touch| touch.phase == TouchPhase::Started)
            .map(|touch| screen_to_game(touch.position.x, touch.position.y))
    }
}

fn system_rect(index: usize) -> Rect {
    Rect::new(40.0 + index as f32 * 296.0, 632.0, 280.0, 142.0)
}

fn search_rect() -> Rect {
    Rect::new(928.0, 632.0, 312.0, 79.0)
}
fn repair_rect() -> Rect {
    Rect::new(928.0, 722.0, 312.0, 52.0)
}
fn main_button() -> Rect {
    Rect::new(438.0, 535.0, 404.0, 52.0)
}
fn pause_rect() -> Rect {
    Rect::new(1146.0, 31.0, 42.0, 35.0)
}
fn sound_rect() -> Rect {
    Rect::new(1198.0, 31.0, 42.0, 35.0)
}

fn difficulty_rect(index: usize) -> Rect {
    Rect::new(361.0 + index as f32 * 188.0, 491.0, 182.0, 30.0)
}

fn travel_rect(index: usize) -> Rect {
    Rect::new(928.0 + index as f32 * 108.0, 722.0, 96.0, 52.0)
}

fn world_x(position: f32) -> f32 {
    755.0 - position * 620.0
}

struct Sounds {
    switch: Option<Sound>,
    alarm: Option<Sound>,
    finish: Option<Sound>,
    wind: Option<Sound>,
    wind_started: bool,
}

impl Sounds {
    async fn load() -> Self {
        Self {
            switch: audio::load_sound_from_bytes(include_bytes!("../assets/switch.wav"))
                .await
                .ok(),
            alarm: audio::load_sound_from_bytes(include_bytes!("../assets/alarm.wav"))
                .await
                .ok(),
            finish: audio::load_sound_from_bytes(include_bytes!("../assets/finish.wav"))
                .await
                .ok(),
            wind: audio::load_sound_from_bytes(include_bytes!("../assets/wind.wav"))
                .await
                .ok(),
            wind_started: false,
        }
    }

    fn play(sound: &Option<Sound>, enabled: bool) {
        if enabled {
            if let Some(sound) = sound {
                audio::play_sound_once(sound);
            }
        }
    }

    fn ambient(&mut self, enabled: bool) {
        if let Some(wind) = &self.wind {
            if enabled && !self.wind_started {
                audio::play_sound(
                    wind,
                    PlaySoundParams {
                        looped: true,
                        volume: 0.35,
                    },
                );
                self.wind_started = true;
            }
            if self.wind_started {
                audio::set_sound_volume(wind, if enabled { 0.35 } else { 0.0 });
            }
        }
    }
}

fn tree(x: f32, ground: f32, height: f32, color: Color) {
    let width = height * 0.37;
    draw_rectangle(x - 2.0, ground - height * 0.20, 4.0, height * 0.20, color);
    for level in 0..4 {
        let level = level as f32;
        let top = ground - height + level * height * 0.17;
        let half = width * (0.55 + level * 0.16);
        draw_triangle(
            vec2(x, top),
            vec2(x - half, top + height * 0.46),
            vec2(x + half, top + height * 0.46),
            color,
        );
    }
}

fn grain(index: u32) -> f32 {
    let n = index.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    ((n ^ (n >> 15)) % 10_000) as f32 / 10_000.0
}

fn scene(game: &Game, t: f32, font: &Font) {
    let dawn = (game.elapsed / NIGHT_SECONDS - 0.76).max(0.0) / 0.24;
    for row in 0..74 {
        let k = row as f32 / 74.0;
        draw_rectangle(
            40.0,
            230.0 + row as f32 * 5.0,
            1200.0,
            5.0,
            Color::new(
                0.055 + k * 0.032 + dawn * 0.13,
                0.10 + k * 0.05 + dawn * 0.10,
                0.15 + k * 0.025 + dawn * 0.06,
                1.0,
            ),
        );
    }
    for index in 0..76 {
        let x = 55.0 + grain(index * 5 + 1) * 1170.0;
        let y = 239.0 + grain(index * 7 + 3) * 190.0;
        let alpha = (0.20 + grain(index * 9 + 4) * 0.4) * (1.0 - dawn);
        draw_rectangle(x, y, 1.5, 1.5, opacity(BLUE, alpha));
    }
    draw_circle(1046.0, 304.0, 44.0, rgba(132, 166, 165, 8));
    draw_circle(1046.0, 304.0, 30.0, rgba(132, 166, 165, 11));
    draw_circle(1046.0, 304.0, 21.0, rgba(170, 187, 170, 150));
    draw_circle(1053.0, 298.0, 21.0, rgba(18, 35, 46, 255));
    for i in 0..13 {
        let x = 40.0 + i as f32 * 102.0;
        let peak = 333.0 + grain(i + 99) * 68.0;
        draw_triangle(
            vec2(x - 100.0, 480.0),
            vec2(x + 35.0, peak),
            vec2(x + 180.0, 480.0),
            rgba(26, 49, 58, 255),
        );
        draw_triangle(
            vec2(x + 11.0, peak + 29.0),
            vec2(x + 35.0, peak),
            vec2(x + 63.0, peak + 39.0),
            rgba(43, 64, 70, 255),
        );
    }
    for i in 0..63 {
        let x = 45.0 + i as f32 * 19.0;
        tree(x, 507.0, 49.0 + grain(i + 51) * 77.0, rgba(19, 42, 46, 255));
    }
    for i in 0..31 {
        let x = 45.0 + i as f32 * 40.0;
        tree(x, 548.0, 95.0 + grain(i + 15) * 90.0, rgba(11, 32, 34, 255));
    }
    draw_rectangle(40.0, 540.0, 1200.0, 62.0, rgba(30, 47, 47, 255));
    for i in 0..16 {
        let x = 40.0 + i as f32 * 82.0;
        draw_ellipse(
            x,
            550.0 + grain(i + 400) * 20.0,
            94.0,
            16.0,
            0.0,
            rgba(41, 59, 57, 255),
        );
    }
    draw_triangle(
        vec2(713.0, 541.0),
        vec2(631.0, 602.0),
        vec2(950.0, 602.0),
        rgba(53, 65, 58, 130),
    );
    let beam = game.powered(0) && game.searching <= 0.0;
    if beam {
        for (x, y, alpha) in [(139.0, 583.0, 13), (199.0, 587.0, 11), (267.0, 589.0, 10)] {
            draw_triangle(
                vec2(567.0, 432.0),
                vec2(x, y),
                vec2(x + 126.0, y - 12.0),
                rgba(214, 217, 160, alpha),
            );
        }
        draw_triangle(
            vec2(567.0, 432.0),
            vec2(84.0, 579.0),
            vec2(420.0, 564.0),
            rgba(213, 219, 165, 17),
        );
        draw_ellipse(255.0, 567.0, 181.0, 15.0, 0.0, rgba(193, 202, 151, 12));
    }
    for i in 0..17 {
        let x = 100.0 + i as f32 * 66.0;
        draw_rectangle(x, 534.0, 3.0, 39.0, rgba(94, 110, 94, 160));
        draw_line(x, 541.0, x + 66.0, 544.0, 1.0, rgba(109, 125, 107, 110));
        draw_line(x, 555.0, x + 66.0, 557.0, 1.0, rgba(109, 125, 107, 110));
    }
    draw_rectangle(567.0, 421.0, 247.0, 140.0, rgba(36, 48, 45, 255));
    for i in 0..20 {
        draw_rectangle(
            569.0 + i as f32 * 12.0,
            423.0,
            2.0,
            136.0,
            rgba(22, 35, 33, 255),
        );
    }
    for i in 0..6 {
        draw_line(
            567.0,
            435.0 + i as f32 * 21.0,
            814.0,
            435.0 + i as f32 * 21.0,
            1.0,
            rgba(50, 62, 55, 255),
        );
    }
    draw_triangle(
        vec2(548.0, 426.0),
        vec2(689.0, 351.0),
        vec2(833.0, 426.0),
        rgba(25, 37, 36, 255),
    );
    draw_line(549.0, 424.0, 689.0, 350.0, 6.0, rgba(131, 145, 131, 255));
    draw_line(689.0, 350.0, 833.0, 424.0, 6.0, rgba(103, 124, 116, 255));
    draw_rectangle(545.0, 423.0, 290.0, 8.0, rgba(17, 29, 29, 255));
    draw_rectangle(770.0, 351.0, 17.0, 50.0, rgba(42, 51, 46, 255));
    draw_rectangle(766.0, 348.0, 24.0, 6.0, rgba(115, 128, 115, 255));
    if game.powered(1) {
        for i in 0..7 {
            let phase = (t * 0.28 + i as f32 * 0.16).fract();
            draw_ellipse(
                779.0 + phase * 39.0,
                341.0 - phase * 53.0,
                8.0 + phase * 16.0,
                4.0 + phase * 7.0,
                0.0,
                rgba(155, 171, 156, ((1.0 - phase) * 26.0) as u8),
            );
        }
    }
    let warm = game.powered(1);
    if warm {
        draw_circle(641.0, 484.0, 65.0, rgba(223, 169, 91, 5));
        draw_circle(641.0, 484.0, 46.0, rgba(223, 169, 91, 6));
    }
    for x in [599.0, 662.0] {
        draw_rectangle(x - 4.0, 452.0, 45.0, 48.0, rgba(17, 28, 29, 255));
        draw_rectangle(
            x,
            456.0,
            37.0,
            39.0,
            if warm {
                rgba(200, 149, 80, 210)
            } else {
                rgba(69, 103, 110, 160)
            },
        );
        draw_line(x + 19.0, 456.0, x + 19.0, 495.0, 3.0, rgba(41, 48, 39, 255));
        draw_line(x, 476.0, x + 37.0, 476.0, 3.0, rgba(41, 48, 39, 255));
    }
    if !game.outside() && game.phase != Phase::Lost {
        draw_circle(626.0, 480.0, 4.0, rgba(37, 40, 34, 255));
        draw_rectangle(622.0, 484.0, 8.0, 10.0, rgba(37, 40, 34, 255));
    }
    draw_rectangle(733.0, 452.0, 43.0, 109.0, rgba(16, 28, 28, 255));
    draw_rectangle(738.0, 457.0, 32.0, 96.0, rgba(46, 58, 49, 255));
    draw_rectangle(742.0, 464.0, 24.0, 32.0, rgba(139, 143, 97, 80));
    draw_circle(762.0, 514.0, 2.0, AMBER);
    draw_rectangle(725.0, 559.0, 62.0, 6.0, rgba(100, 116, 100, 255));
    draw_rectangle(720.0, 566.0, 72.0, 5.0, rgba(77, 96, 84, 255));
    txt(
        font,
        "STATION / 07",
        598.0,
        530.0,
        11,
        rgba(143, 154, 129, 255),
    );
    draw_rectangle(
        561.0,
        431.0,
        13.0,
        8.0,
        if beam {
            rgba(224, 228, 170, 255)
        } else {
            rgba(68, 83, 74, 255)
        },
    );
    draw_rectangle(467.0, 525.0, 61.0, 38.0, rgba(47, 59, 48, 255));
    draw_rectangle(472.0, 530.0, 51.0, 26.0, rgba(30, 45, 40, 255));
    for i in 0..6 {
        draw_rectangle(
            477.0 + i as f32 * 6.0,
            534.0,
            2.0,
            18.0,
            rgba(81, 95, 71, 255),
        );
    }
    draw_circle(518.0, 533.0, 2.0, if game.fuel > 0.0 { LIME } else { RED });
    draw_line(528.0, 552.0, 569.0, 551.0, 2.0, rgba(20, 31, 27, 255));
    for (index, position) in FUEL_SITES.iter().enumerate() {
        if index < 3 - game.searches_left as usize
            || (index == 3 - game.searches_left as usize && game.carrying)
        {
            continue;
        }
        let x = world_x(*position);
        let current = index == 3 - game.searches_left as usize;
        let fill = if current { AMBER } else { MUTED };
        draw_rectangle(x - 7.0, 551.0, 14.0, 18.0, fill);
        draw_rectangle_lines(x - 4.0, 547.0, 8.0, 7.0, 2.0, fill);
        if current && game.outside() {
            draw_triangle(
                vec2(x - 6.0, 535.0),
                vec2(x + 6.0, 535.0),
                vec2(x, 542.0),
                LIME,
            );
            centered(font, "ТОПЛИВО", x, 529.0, 10, LIME);
        }
    }
    draw_line(905.0, 546.0, 927.0, 345.0, 3.0, rgba(93, 114, 106, 255));
    draw_line(950.0, 546.0, 927.0, 345.0, 3.0, rgba(93, 114, 106, 255));
    for i in 0..6 {
        let y = 381.0 + i as f32 * 28.0;
        let half = (y - 345.0) / 9.0;
        draw_line(
            927.0 - half,
            y,
            927.0 + half,
            y,
            2.0,
            rgba(79, 101, 96, 255),
        );
    }
    draw_line(904.0, 369.0, 950.0, 369.0, 2.0, rgba(110, 131, 118, 255));
    draw_circle(
        927.0,
        343.0,
        2.5,
        if game.powered(2) && (t * 2.0).sin() > 0.0 {
            AMBER
        } else {
            rgba(118, 64, 51, 255)
        },
    );
    if game.powered(2) {
        for i in 0..3 {
            let pulse = (t * 0.45 + i as f32 / 3.0).fract();
            draw_circle_lines(
                927.0,
                344.0,
                12.0 + pulse * 30.0,
                1.0,
                opacity(LIME, (1.0 - pulse) * 0.14),
            );
        }
    }
    if game.outside() || game.phase == Phase::Lost {
        let x = world_x(game.position);
        let y = 567.0;
        draw_circle(x, y - 8.0, 23.0, rgba(219, 221, 156, 12));
        draw_circle(x, y - 16.0, 4.0, rgba(178, 158, 115, 255));
        draw_rectangle(x - 5.0, y - 12.0, 10.0, 14.0, rgba(133, 122, 84, 255));
        draw_line(
            x - 2.0,
            y + 2.0,
            x - 4.0 + (t * 9.0).sin() * 2.0,
            y + 7.0,
            3.0,
            rgba(39, 44, 38, 255),
        );
        draw_line(
            x + 2.0,
            y + 2.0,
            x + 4.0 - (t * 9.0).sin() * 2.0,
            y + 7.0,
            3.0,
            rgba(39, 44, 38, 255),
        );
        draw_circle(x - 12.0, y - 1.0, 13.0, rgba(236, 182, 99, 8));
        draw_rectangle(
            x - 15.0,
            y - 4.0,
            5.0,
            8.0,
            if game.carrying { AMBER } else { BLUE },
        );
        if game.carrying {
            draw_triangle(
                vec2(748.0, 537.0),
                vec2(762.0, 537.0),
                vec2(755.0, 545.0),
                LIME,
            );
            centered(font, "СТАНЦИЯ · E", 755.0, 529.0, 10, LIME);
        }
    }
    if game.threat > 22.0 {
        for i in 0..((game.threat / 22.0) as u32).min(4) {
            let x = 111.0 + i as f32 * 82.0 + (t * 0.3 + i as f32).sin() * 4.0;
            let y = 504.0 + i as f32 * 8.0;
            let a = if beam { 90 } else { 180 };
            draw_rectangle(x, y, 3.0, 2.0, rgba(221, 138, 96, a));
            draw_rectangle(x + 6.0, y, 3.0, 2.0, rgba(221, 138, 96, a));
        }
    }
    for i in 0..90 {
        let speed = if game.storm() { 39.0 } else { 10.0 };
        let x = 42.0 + (grain(i * 3 + 501) * 1196.0 + t * speed * 0.4) % 1196.0;
        let y = 233.0 + (grain(i * 5 + 888) * 366.0 + t * speed) % 366.0;
        draw_rectangle(
            x,
            y,
            if i % 7 == 0 { 2.0 } else { 1.0 },
            if game.storm() { 3.0 } else { 1.0 },
            rgba(164, 190, 184, if game.storm() { 72 } else { 40 }),
        );
    }
    for x in [63.0, 1198.0, 1224.0] {
        tree(
            x,
            602.0,
            if x < 100.0 { 166.0 } else { 215.0 },
            rgba(8, 24, 25, 255),
        );
    }
    draw_rectangle(40.0, 230.0, 1200.0, 36.0, rgba(11, 24, 29, 145));
    draw_circle(
        62.0,
        249.0,
        3.0,
        if game.threat >= 60.0 { RED } else { LIME },
    );
    txt(
        font,
        if game.searching > 0.0 {
            "ОПЕРАТОР СНАРУЖИ"
        } else if game.threat >= 60.0 {
            "ДВИЖЕНИЕ У ПЕРИМЕТРА"
        } else {
            "СЕВЕРНАЯ СТАНЦИЯ / 07"
        },
        75.0,
        253.0,
        11,
        if game.threat >= 60.0 {
            RED
        } else {
            rgba(177, 195, 183, 255)
        },
    );
    right(
        font,
        if game.storm() {
            "МЕТЕЛЬ · -32°C"
        } else {
            "ЯСНО · -22°C"
        },
        1218.0,
        253.0,
        11,
        rgba(159, 180, 176, 255),
    );
    draw_rectangle(40.0, 578.0, 1200.0, 24.0, rgba(10, 25, 26, 160));
    txt(font, "АКТИВНОСТЬ В ЛЕСУ", 58.0, 594.0, 10, MUTED);
    bar(
        214.0,
        587.0,
        122.0,
        3.0,
        game.threat / 100.0,
        if game.threat >= 60.0 { RED } else { AMBER },
    );
    right(
        font,
        if beam {
            "ПРОЖЕКТОР ВКЛЮЧЁН"
        } else {
            "ПРОЖЕКТОР ВЫКЛЮЧЕН"
        },
        1219.0,
        594.0,
        10,
        if beam { LIME } else { AMBER },
    );
    draw_rectangle(0.0, 230.0, 40.0, 372.0, BG);
    draw_rectangle(1240.0, 230.0, 40.0, 372.0, BG);
    draw_rectangle_lines(40.0, 230.0, 1200.0, 372.0, 1.0, LINE);
    if game.outside() {
        box_fill(
            Rect::new(60.0, 277.0, 550.0, 72.0),
            rgba(11, 24, 29, 235),
            LINE,
        );
        txt(
            font,
            &format!("СНАРУЖИ {:04.1} С", game.searching),
            76.0,
            300.0,
            15,
            if game.searching < 5.0 { RED } else { LIME },
        );
        txt(
            font,
            if game.carrying {
                "Канистра у тебя. Вернись к двери и нажми E."
            } else {
                "Иди к отмеченной канистре. Нажми E рядом с ней."
            },
            76.0,
            320.0,
            11,
            INK,
        );
        txt(
            font,
            "A / D или стрелки · Shift — бег · клик по сцене — идти",
            76.0,
            337.0,
            10,
            MUTED,
        );
        right(
            font,
            &format!("ВЫНОСЛИВОСТЬ {:.0}%", game.stamina),
            1218.0,
            297.0,
            10,
            MUTED,
        );
        bar(
            1058.0,
            307.0,
            160.0,
            4.0,
            game.stamina / 100.0,
            if game.sprinting { AMBER } else { LIME },
        );
    }
}

fn stat(font: &Font, x: f32, label: &str, value: &str, suffix: &str, ratio: f32, color: Color) {
    txt(font, label, x, 158.0, 11, MUTED);
    txt(font, value, x, 194.0, 28, color);
    let width = text_width(font, value, 28);
    txt(font, suffix, x + width + 8.0, 191.0, 11, MUTED);
    bar(x, 208.0, 260.0, 3.0, ratio, opacity(color, 0.65));
}

fn lamp_icon(x: f32, y: f32, color: Color) {
    draw_circle_lines(x, y, 8.0, 1.7, color);
    draw_line(x - 4.0, y + 9.0, x + 4.0, y + 9.0, 2.0, color);
    draw_line(x - 3.0, y + 13.0, x + 3.0, y + 13.0, 2.0, color);
    for angle in [0.0_f32, 0.8, 2.3, 3.14, 3.9, 5.5] {
        draw_line(
            x + angle.cos() * 13.0,
            y + angle.sin() * 13.0,
            x + angle.cos() * 17.0,
            y + angle.sin() * 17.0,
            1.4,
            color,
        );
    }
}

fn system_card(game: &Game, font: &Font, index: usize, hover: Vec2) {
    let rect = system_rect(index);
    let active = game.powered(index);
    let locked = game.outside();
    let color = if active {
        if index == 1 {
            AMBER
        } else {
            LIME
        }
    } else {
        MUTED
    };
    let over = rect.contains(hover) && game.phase == Phase::Playing && !locked;
    box_fill(
        rect,
        if over { rgba(27, 42, 43, 255) } else { PANEL },
        if over { opacity(LIME, 0.55) } else { LINE },
    );
    if active {
        draw_rectangle(rect.x, rect.y, rect.w, 2.0, opacity(color, 0.6));
    }
    keycap(
        font,
        &(index + 1).to_string(),
        rect.x + 18.0,
        rect.y + 18.0,
        color,
    );
    txt(
        font,
        ["Свет", "Отопление", "Радио"][index],
        rect.x + 55.0,
        rect.y + 37.0,
        19,
        INK,
    );
    if index == 0 {
        lamp_icon(rect.x + 244.0, rect.y + 31.0, color);
    }
    if index == 1 {
        for i in 0..3 {
            let x = rect.x + 236.0 + i as f32 * 7.0;
            draw_line(x, rect.y + 19.0, x - 3.0, rect.y + 27.0, 2.0, color);
            draw_line(x - 3.0, rect.y + 27.0, x, rect.y + 36.0, 2.0, color);
        }
    }
    if index == 2 {
        draw_circle(rect.x + 244.0, rect.y + 32.0, 3.0, color);
        draw_circle_lines(rect.x + 244.0, rect.y + 32.0, 9.0, 1.0, opacity(color, 0.7));
        draw_circle_lines(
            rect.x + 244.0,
            rect.y + 32.0,
            15.0,
            1.0,
            opacity(color, 0.4),
        );
    }
    txt(
        font,
        [
            "Сдерживает тех, кто в лесу",
            "Сохраняет тепло на станции",
            "Вызывает помощь раньше утра",
        ][index],
        rect.x + 18.0,
        rect.y + 69.0,
        11,
        MUTED,
    );
    draw_line(
        rect.x + 18.0,
        rect.y + 88.0,
        rect.x + rect.w - 18.0,
        rect.y + 88.0,
        1.0,
        LINE,
    );
    let status = if locked {
        "ОПЕРАТОР СНАРУЖИ"
    } else if game.fuel <= 0.0 {
        "НЕТ ТОПЛИВА"
    } else if active {
        "ВКЛЮЧЕНО"
    } else {
        "ВЫКЛЮЧЕНО"
    };
    draw_circle(rect.x + 21.0, rect.y + 113.0, 3.0, color);
    txt(font, status, rect.x + 32.0, rect.y + 117.0, 10, color);
    right(
        font,
        ["−0.17 л/с", "−0.21 л/с", "−0.24 л/с"][index],
        rect.x + rect.w - 18.0,
        rect.y + 117.0,
        10,
        MUTED,
    );
}

fn dashboard(game: &Game, font: &Font, hover: Vec2, sound: bool, auto_sprint: bool) {
    txt(
        font,
        "ОДНА СТАНЦИЯ. ШЕСТЬ ЧАСОВ ДО УТРА.",
        40.0,
        43.0,
        11,
        LIME,
    );
    txt(font, "СВЕТ ДО РАССВЕТА", 37.0, 96.0, 45, INK);
    txt(
        font,
        "Береги топливо. Сохраняй тепло. Не оставляй лес без присмотра.",
        40.0,
        124.0,
        13,
        MUTED,
    );
    right(
        font,
        &format!("РЕЖИМ: {}", game.difficulty.name().to_uppercase()),
        1240.0,
        124.0,
        10,
        LIME,
    );
    let hours = (game.elapsed / NIGHT_SECONDS * 360.0).min(360.0) as u32;
    right(
        font,
        &format!("{:02}:{:02}", hours / 60, hours % 60),
        1118.0,
        72.0,
        31,
        INK,
    );
    let remaining = (NIGHT_SECONDS - game.elapsed).max(0.0).ceil() as u32;
    right(
        font,
        &format!("ДО УТРА {:01}:{:02}", remaining / 60, remaining % 60),
        1118.0,
        96.0,
        10,
        MUTED,
    );
    box_fill(
        pause_rect(),
        if pause_rect().contains(hover) {
            LINE
        } else {
            PANEL
        },
        LINE,
    );
    centered(
        font,
        if game.phase == Phase::Paused {
            ">"
        } else {
            "II"
        },
        1167.0,
        54.0,
        15,
        INK,
    );
    box_fill(
        sound_rect(),
        if sound { rgba(37, 49, 38, 255) } else { PANEL },
        LINE,
    );
    centered(
        font,
        if sound { "♪" } else { "×" },
        1219.0,
        55.0,
        19,
        if sound { LIME } else { MUTED },
    );
    stat(
        font,
        40.0,
        "01 / ТОПЛИВО",
        &format!("{:02.0}", game.fuel),
        "ЛИТРОВ",
        game.fuel / 100.0,
        if game.fuel < 15.0 { RED } else { LIME },
    );
    stat(
        font,
        352.0,
        "02 / ВНУТРИ СТАНЦИИ",
        &format!("{:+.0}°", game.temperature),
        "КРИТИЧНО −10°",
        (game.temperature + 10.0) / 36.0,
        if game.temperature < 0.0 { RED } else { BLUE },
    );
    stat(
        font,
        664.0,
        "03 / ПРОЧНОСТЬ",
        &format!("{:.0}%", game.integrity),
        "ПЕРИМЕТР",
        game.integrity / 100.0,
        if game.integrity < 40.0 { RED } else { INK },
    );
    stat(
        font,
        976.0,
        "04 / СИГНАЛ ПОМОЩИ",
        &format!("{:.0}%", game.signal),
        "ЦЕЛЬ 100%",
        game.signal / 100.0,
        LIME,
    );
    for i in 0..3 {
        system_card(game, font, i, hover);
    }
    let rect = search_rect();
    let enabled = game.can_interact();
    let fill = if enabled {
        if rect.contains(hover) {
            rgba(216, 231, 174, 255)
        } else {
            LIME
        }
    } else {
        PANEL
    };
    box_fill(rect, fill, if enabled { LIME } else { LINE });
    let color = if enabled { BG } else { MUTED };
    let label = if game.outside() {
        if game.near_station() {
            if game.carrying {
                "Занести топливо"
            } else {
                "Вернуться на станцию"
            }
        } else if game.near_fuel() && !game.carrying {
            "Забрать канистру"
        } else if game.carrying {
            "Вернись к двери"
        } else {
            "Дойди до канистры"
        }
        .to_owned()
    } else if game.searches_left == 0 {
        "Запасы закончились".to_owned()
    } else if game.search_cooldown > 0.0 {
        format!("Отдых · {:.0} с", game.search_cooldown.ceil())
    } else {
        "Найти топливо".to_owned()
    };
    txt(font, &label, rect.x + 19.0, rect.y + 31.0, 17, color);
    txt(
        font,
        &if game.outside() {
            format!(
                "Снаружи {:.1} с / груз: {}",
                game.searching,
                if game.carrying {
                    "канистра"
                } else {
                    "пусто"
                }
            )
        } else {
            format!(
                "+{:.0} л / {:.0} с на путь / осталось {}",
                game.difficulty.fuel_reward(),
                game.difficulty.outside_seconds(),
                game.searches_left
            )
        },
        rect.x + 19.0,
        rect.y + 56.0,
        10,
        opacity(color, 0.70),
    );
    if enabled {
        right(font, "E", rect.x + rect.w - 18.0, rect.y + 32.0, 14, BG);
    }
    if game.outside() {
        bar(
            rect.x + 1.0,
            rect.y + rect.h - 4.0,
            rect.w - 2.0,
            3.0,
            game.searching / game.difficulty.outside_seconds(),
            AMBER,
        );
    }
    if game.outside() {
        for (index, label) in ["В ЛЕС", "К ДОМУ", "БЕГ"].iter().enumerate() {
            let rect = travel_rect(index);
            let active = index == 2 && auto_sprint;
            box_fill(
                rect,
                if active {
                    LIME
                } else if rect.contains(hover) {
                    LINE
                } else {
                    PANEL
                },
                LINE,
            );
            centered(
                font,
                label,
                rect.x + rect.w / 2.0,
                rect.y + 23.0,
                11,
                if active { BG } else { INK },
            );
            centered(
                font,
                ["A / <-", "D / ->", "SHIFT"][index],
                rect.x + rect.w / 2.0,
                rect.y + 40.0,
                9,
                if active { BG } else { MUTED },
            );
        }
    } else {
        let repair = repair_rect();
        let can_repair = game.can_repair();
        box_fill(
            repair,
            if can_repair && repair.contains(hover) {
                LINE
            } else {
                PANEL
            },
            LINE,
        );
        txt(
            font,
            "Укрепить периметр",
            repair.x + 17.0,
            repair.y + 22.0,
            12,
            if can_repair { INK } else { MUTED },
        );
        txt(
            font,
            "−9 л / +28% прочности",
            repair.x + 17.0,
            repair.y + 40.0,
            10,
            MUTED,
        );
        right(
            font,
            "F",
            repair.x + repair.w - 18.0,
            repair.y + 30.0,
            14,
            if can_repair { INK } else { MUTED },
        );
    }
    txt(font, "ЖУРНАЛ СТАНЦИИ", 40.0, 811.0, 10, MUTED);
    let events: Vec<_> = game.events.iter().rev().take(2).collect();
    for (index, event) in events.iter().enumerate() {
        let at = (event.at / NIGHT_SECONDS * 360.0) as u32;
        txt(
            font,
            &format!("{:02}:{:02}", at / 60, at % 60),
            40.0,
            835.0 + index as f32 * 21.0,
            11,
            opacity(LIME, if index == 0 { 0.75 } else { 0.4 }),
        );
        txt(
            font,
            event.text,
            103.0,
            835.0 + index as f32 * 21.0,
            11,
            if index == 0 {
                rgba(165, 184, 174, 255)
            } else {
                MUTED
            },
        );
    }
    right(
        font,
        "A / D — ХОДЬБА   SHIFT — БЕГ   E — ДЕЙСТВИЕ   1 2 3 — СИСТЕМЫ   F — РЕМОНТ   P — ПАУЗА   M — ЗВУК",
        1240.0,
        886.0,
        10,
        MUTED,
    );
}

fn overlay(game: &Game, font: &Font, hover: Vec2) {
    if game.phase == Phase::Playing {
        return;
    }
    draw_rectangle(40.0, 230.0, 1200.0, 544.0, rgba(7, 14, 17, 190));
    let panel = Rect::new(336.0, 287.0, 608.0, 325.0);
    box_fill(panel, rgba(17, 29, 31, 250), rgba(72, 91, 78, 255));
    draw_rectangle(panel.x, panel.y, panel.w, 2.0, LIME);
    let (eyebrow, title, line1, line2, line3, label) = match game.phase {
        Phase::Ready => (
            "ТВОЯ ПЕРВАЯ НОЧЬ",
            "Смена начинается",
            "Переживи 3 минуты или передай сигнал помощи на 100%.",
            "Снаружи: A / D — идти, Shift — бежать, E — взять / занести.",
            "Дойди до канистры и вернись, пока не закончился таймер.",
            "Начать смену",
        ),
        Phase::Paused => (
            "СТАНЦИЯ НА ПАУЗЕ",
            "Переведи дыхание",
            "Время остановлено. Топливо и тепло не расходуются.",
            "A / D или стрелки — ходьба. Shift — бег. E — действие.",
            "1 / 2 / 3 — системы. Нажми R, чтобы начать заново.",
            "Вернуться на станцию",
        ),
        Phase::Dawn => (
            "НОЧЬ ПОЗАДИ",
            "Ты встретил рассвет",
            "Свет вернулся в лес. Станция всё ещё стоит.",
            "Ты пережил эту смену. Но следующая ночь уже близко.",
            "Попробуй вызвать спасателей до рассвета через радио.",
            "Ещё одна смена",
        ),
        Phase::Rescued => (
            "СИГНАЛ ПРИНЯТ",
            "Помощь пришла",
            "Твои координаты получили. Вертолёт уже над станцией.",
            "Ты выбрался из леса до рассвета.",
            "В следующий раз попробуй сохранить больше топлива.",
            "Ещё одна смена",
        ),
        Phase::Frozen => (
            "СВЯЗЬ СО СТАНЦИЕЙ ПОТЕРЯНА",
            "Слишком холодно",
            "Температура опустилась до −10°. Ты не пережил ночь.",
            "Отопление можно выключать, пока в комнате тепло.",
            "Но не откладывай его включение до последней минуты.",
            "Попробовать снова",
        ),
        Phase::Overrun => (
            "СВЯЗЬ СО СТАНЦИЕЙ ПОТЕРЯНА",
            "Периметр прорван",
            "Ограждение не выдержало. Лес добрался до станции.",
            "Следи за активностью и не оставляй свет выключенным.",
            "Потрёпанный периметр можно укрепить клавишей F.",
            "Попробовать снова",
        ),
        Phase::Lost => (
            "СТАНЦИЯ ОСТАЛАСЬ В ТЕМНОТЕ",
            "Ты не успел вернуться",
            "Время снаружи вышло. Нужно вернуться к двери и нажать E.",
            "Shift ускоряет ходьбу, но расходует силы и привлекает лес.",
            "Можно вернуться без канистры, если времени мало.",
            "Попробовать снова",
        ),
        Phase::Playing => unreachable!(),
    };
    centered(
        font,
        eyebrow,
        640.0,
        329.0,
        11,
        if matches!(game.phase, Phase::Frozen | Phase::Overrun | Phase::Lost) {
            RED
        } else {
            LIME
        },
    );
    centered(font, title, 640.0, 378.0, 32, INK);
    centered(font, line1, 640.0, 426.0, 12, rgba(159, 180, 169, 255));
    centered(font, line2, 640.0, 451.0, 12, rgba(159, 180, 169, 255));
    centered(font, line3, 640.0, 476.0, 12, rgba(159, 180, 169, 255));
    if game.phase == Phase::Ready || game.is_finished() {
        for (index, difficulty) in Difficulty::ALL.iter().enumerate() {
            let rect = difficulty_rect(index);
            let active = *difficulty == game.difficulty;
            box_fill(
                rect,
                if active {
                    LIME
                } else if rect.contains(hover) {
                    LINE
                } else {
                    PANEL
                },
                LINE,
            );
            centered(
                font,
                difficulty.name(),
                rect.x + rect.w / 2.0,
                rect.y + 20.0,
                12,
                if active { BG } else { MUTED },
            );
        }
    }
    let button = main_button();
    box_fill(
        button,
        if button.contains(hover) {
            rgba(221, 233, 181, 255)
        } else {
            LIME
        },
        LIME,
    );
    centered(font, label, 640.0, 567.0, 17, BG);
    centered(
        font,
        if game.phase == Phase::Ready || game.is_finished() {
            "Q — СЛОЖНОСТЬ · ENTER — НАЧАТЬ"
        } else {
            "ENTER — ВЕРНУТЬСЯ"
        },
        640.0,
        604.0,
        9,
        MUTED,
    );
}

#[macroquad::main(configuration)]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/IBMPlexMono-Regular.ttf"))
        .expect("Bundled font must be valid");
    let mut sounds = Sounds::load().await;
    let mut game = Game::default();
    let mut sound_on = false;
    let mut font_dpi = 0.0;
    let mut destination: Option<f32> = None;
    let mut auto_sprint = false;
    #[cfg(target_arch = "wasm32")]
    let mut status_marker = (u32::MAX, -1_i32, u32::MAX);
    let camera = Camera2D {
        target: vec2(W / 2.0, H / 2.0),
        zoom: vec2(2.0 / W, 2.0 / H),
        ..Default::default()
    };
    #[cfg(target_arch = "wasm32")]
    unsafe {
        oddseed_ready();
    }
    loop {
        let dpi = macroquad::miniquad::window::dpi_scale();
        if dpi != font_dpi {
            prepare_font(&font, dpi);
            font_dpi = dpi;
        }
        if PAUSE_REQUEST.swap(false, Ordering::Relaxed) && game.phase == Phase::Playing {
            game.pause();
        }
        let pointer = pointer_press();
        let (mx, my) = mouse_position();
        let hover = screen_to_game(mx, my);
        let phase_before = game.phase;
        if game.phase == Phase::Ready || game.is_finished() {
            if is_key_pressed(KeyCode::Q) {
                let index = (game.difficulty as usize + 1) % Difficulty::ALL.len();
                game.select_difficulty(Difficulty::ALL[index]);
            }
            for (index, difficulty) in Difficulty::ALL.iter().enumerate() {
                if in_rect(pointer, difficulty_rect(index)) {
                    game.select_difficulty(*difficulty);
                }
            }
        }
        if is_key_pressed(KeyCode::M) || in_rect(pointer, sound_rect()) {
            sound_on = !sound_on;
            Sounds::play(&sounds.switch, sound_on);
        }
        if is_key_pressed(KeyCode::P)
            || is_key_pressed(KeyCode::Escape)
            || in_rect(pointer, pause_rect())
        {
            game.pause();
        }
        if game.phase != Phase::Playing
            && (is_key_pressed(KeyCode::Enter) || in_rect(pointer, main_button()))
        {
            if game.phase == Phase::Paused {
                game.pause();
            } else {
                game.start();
            }
            Sounds::play(&sounds.switch, sound_on);
        } else if game.phase == Phase::Playing {
            for (index, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3]
                .iter()
                .enumerate()
            {
                if (is_key_pressed(*key) || in_rect(pointer, system_rect(index)))
                    && game.toggle(index)
                {
                    Sounds::play(&sounds.switch, sound_on);
                }
            }
            if (is_key_pressed(KeyCode::E) || in_rect(pointer, search_rect())) && game.interact() {
                destination = None;
                Sounds::play(&sounds.switch, sound_on);
            }
            if (is_key_pressed(KeyCode::F) || (!game.outside() && in_rect(pointer, repair_rect())))
                && game.repair()
            {
                Sounds::play(&sounds.switch, sound_on);
            }
            if game.outside() {
                if let Some(point) =
                    pointer.filter(|point| Rect::new(40.0, 350.0, 1200.0, 228.0).contains(*point))
                {
                    destination = Some(((755.0 - point.x) / 620.0).clamp(0.0, 1.0));
                }
                if in_rect(pointer, travel_rect(0)) {
                    destination = Some(game.fuel_site());
                }
                if in_rect(pointer, travel_rect(1)) {
                    destination = Some(0.0);
                }
                if in_rect(pointer, travel_rect(2)) {
                    auto_sprint = !auto_sprint;
                }
            }
        }
        if is_key_pressed(KeyCode::R) && (game.phase == Phase::Paused || game.is_finished()) {
            game.start();
            destination = None;
            auto_sprint = false;
        }
        let left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
        let right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
        let mut direction = i8::from(left) - i8::from(right);
        if left || right {
            destination = None;
        } else if let Some(target) = destination {
            let delta = target - game.position;
            if delta.abs() <= 0.02 {
                destination = None;
            } else {
                direction = if delta > 0.0 { 1 } else { -1 };
            }
        }
        if !game.outside() {
            destination = None;
            auto_sprint = false;
        }
        let integrity = game.integrity;
        game.advance_with_controls(
            get_frame_time().min(0.10),
            Controls {
                direction,
                sprint: auto_sprint
                    || is_key_down(KeyCode::LeftShift)
                    || is_key_down(KeyCode::RightShift),
            },
        );
        if game.integrity < integrity {
            Sounds::play(&sounds.alarm, sound_on);
        }
        if game.is_finished()
            && !matches!(
                phase_before,
                Phase::Dawn | Phase::Rescued | Phase::Frozen | Phase::Overrun | Phase::Lost
            )
        {
            Sounds::play(&sounds.finish, sound_on);
        }
        sounds.ambient(sound_on && game.phase == Phase::Playing);
        #[cfg(target_arch = "wasm32")]
        {
            let marker = (
                game.phase as u32,
                (game.elapsed * 4.0) as i32,
                game.difficulty as u32,
            );
            if marker != status_marker {
                unsafe {
                    oddseed_status(
                        game.phase as u32,
                        game.elapsed,
                        game.fuel,
                        game.temperature,
                        game.integrity,
                        game.signal,
                        game.position,
                        game.searching,
                        u32::from(game.carrying),
                        game.stamina,
                        game.difficulty as u32,
                        game.searches_left as u32,
                    );
                }
                status_marker = marker;
            }
        }
        clear_background(BG);
        set_camera(&camera);
        dashboard(&game, &font, hover, sound_on, auto_sprint);
        scene(&game, get_time() as f32, &font);
        overlay(&game, &font, hover);
        set_default_camera();
        next_frame().await;
    }
}
