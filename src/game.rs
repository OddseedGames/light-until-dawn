pub const NIGHT_SECONDS: f32 = 180.0;
pub const FUEL_SITES: [f32; 3] = [0.55, 0.77, 1.0];
pub const INTERACTION_DISTANCE: f32 = 0.035;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u32)]
pub enum Difficulty {
    Calm,
    #[default]
    Night,
    Nightmare,
}

impl Difficulty {
    pub const ALL: [Self; 3] = [Self::Calm, Self::Night, Self::Nightmare];

    pub fn name(self) -> &'static str {
        match self {
            Self::Calm => "Знакомство",
            Self::Night => "Ночь",
            Self::Nightmare => "Кошмар",
        }
    }

    pub fn initial_fuel(self) -> f32 {
        match self {
            Self::Calm => 54.0,
            Self::Night => 42.0,
            Self::Nightmare => 38.0,
        }
    }

    pub fn outside_seconds(self) -> f32 {
        match self {
            Self::Calm => 24.0,
            Self::Night => 18.0,
            Self::Nightmare => 15.0,
        }
    }

    pub fn fuel_reward(self) -> f32 {
        match self {
            Self::Calm => 22.0,
            Self::Night => 18.0,
            Self::Nightmare => 18.0,
        }
    }

    fn resource_rate(self) -> f32 {
        match self {
            Self::Calm => 0.85,
            Self::Night => 1.0,
            Self::Nightmare => 1.15,
        }
    }

    fn danger_rate(self) -> f32 {
        match self {
            Self::Calm => 0.8,
            Self::Night => 1.0,
            Self::Nightmare => 1.3,
        }
    }

    fn attack_damage(self) -> f32 {
        match self {
            Self::Calm => 9.0,
            Self::Night => 12.0,
            Self::Nightmare => 16.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum Phase {
    Ready,
    Playing,
    Paused,
    Dawn,
    Rescued,
    Frozen,
    Overrun,
    Lost,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Controls {
    pub direction: i8,
    pub sprint: bool,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub at: f32,
    pub text: &'static str,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub difficulty: Difficulty,
    pub phase: Phase,
    pub elapsed: f32,
    pub fuel: f32,
    pub temperature: f32,
    pub integrity: f32,
    pub signal: f32,
    pub threat: f32,
    pub systems: [bool; 3],
    pub searches_left: u8,
    pub searching: f32,
    pub position: f32,
    pub carrying: bool,
    pub stamina: f32,
    pub sprinting: bool,
    pub search_cooldown: f32,
    pub repair_cooldown: f32,
    pub events: Vec<Event>,
    attack_cooldown: f32,
    previous_storm: bool,
    winded: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Difficulty::default())
    }
}

impl Game {
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            difficulty,
            phase: Phase::Ready,
            elapsed: 0.0,
            fuel: difficulty.initial_fuel(),
            temperature: 18.0,
            integrity: 100.0,
            signal: 0.0,
            threat: 16.0,
            systems: [true, false, false],
            searches_left: 3,
            searching: 0.0,
            position: 0.0,
            carrying: false,
            stamina: 100.0,
            sprinting: false,
            search_cooldown: 0.0,
            repair_cooldown: 0.0,
            events: vec![Event {
                at: 0.0,
                text: "Смена началась. В лесу замечено движение.",
            }],
            attack_cooldown: 0.0,
            previous_storm: false,
            winded: false,
        }
    }

    pub fn select_difficulty(&mut self, difficulty: Difficulty) -> bool {
        if self.phase != Phase::Ready && !self.is_finished() {
            return false;
        }
        *self = Self::new(difficulty);
        true
    }

    pub fn start(&mut self) {
        *self = Self::new(self.difficulty);
        self.phase = Phase::Playing;
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.phase,
            Phase::Dawn | Phase::Rescued | Phase::Frozen | Phase::Overrun | Phase::Lost
        )
    }

    pub fn outside(&self) -> bool {
        self.searching > 0.0
    }

    pub fn fuel_site(&self) -> f32 {
        FUEL_SITES[(3 - self.searches_left).min(2) as usize]
    }

    pub fn near_station(&self) -> bool {
        self.position <= INTERACTION_DISTANCE
    }

    pub fn near_fuel(&self) -> bool {
        self.searches_left > 0 && (self.position - self.fuel_site()).abs() <= INTERACTION_DISTANCE
    }

    pub fn pause(&mut self) {
        self.phase = match self.phase {
            Phase::Playing => Phase::Paused,
            Phase::Paused => Phase::Playing,
            phase => phase,
        };
    }

    pub fn powered(&self, index: usize) -> bool {
        self.fuel > 0.0 && self.systems.get(index).copied().unwrap_or(false)
    }

    pub fn storm(&self) -> bool {
        (38.0..58.0).contains(&self.elapsed)
            || (98.0..128.0).contains(&self.elapsed)
            || (157.0..174.0).contains(&self.elapsed)
    }

    pub fn consumption(&self) -> f32 {
        if self.fuel <= 0.0 {
            return 0.0;
        }
        (0.03
            + self
                .systems
                .iter()
                .zip([0.17, 0.21, 0.24])
                .filter(|(enabled, _)| **enabled)
                .map(|(_, rate)| rate)
                .sum::<f32>())
            * self.difficulty.resource_rate()
    }

    pub fn toggle(&mut self, index: usize) -> bool {
        if self.phase != Phase::Playing || index >= 3 || self.outside() {
            return false;
        }
        self.systems[index] = !self.systems[index];
        true
    }

    pub fn can_search(&self) -> bool {
        self.phase == Phase::Playing
            && self.searches_left > 0
            && !self.outside()
            && self.search_cooldown <= 0.0
    }

    pub fn search(&mut self) -> bool {
        if !self.can_search() {
            return false;
        }
        self.searching = self.difficulty.outside_seconds();
        self.position = 0.0;
        self.carrying = false;
        self.systems[0] = false;
        self.log("Ты снаружи. Дойди до канистры и вернись до конца таймера.");
        true
    }

    pub fn can_interact(&self) -> bool {
        self.phase == Phase::Playing
            && if self.outside() {
                self.near_station() || (!self.carrying && self.near_fuel())
            } else {
                self.can_search()
            }
    }

    pub fn interact(&mut self) -> bool {
        if !self.can_interact() {
            return false;
        }
        if !self.outside() {
            return self.search();
        }
        if self.near_station() {
            if self.carrying {
                self.searches_left -= 1;
                self.fuel = (self.fuel + self.difficulty.fuel_reward()).min(100.0);
                self.log("Канистра на станции. Топливо заправлено, прожектор включён.");
            } else {
                self.log("Ты вернулся без канистры. Прожектор включён.");
            }
            self.search_cooldown = if self.carrying { 18.0 } else { 6.0 };
            self.carrying = false;
            self.searching = 0.0;
            self.position = 0.0;
            self.sprinting = false;
            self.systems[0] = true;
        } else {
            self.carrying = true;
            self.log("Канистра у тебя. Вернись к двери и нажми E.");
        }
        true
    }

    pub fn can_repair(&self) -> bool {
        self.phase == Phase::Playing
            && !self.outside()
            && self.fuel >= 9.0
            && self.integrity <= 90.0
            && self.repair_cooldown <= 0.0
    }

    pub fn repair(&mut self) -> bool {
        if !self.can_repair() {
            return false;
        }
        self.fuel -= 9.0;
        self.integrity = (self.integrity + 28.0).min(100.0);
        self.repair_cooldown = 16.0;
        self.log("Периметр укреплён. Потрачено 9 л топлива, прочность +28%.");
        true
    }

    pub fn advance(&mut self, seconds: f32) {
        self.advance_with_controls(seconds, Controls::default());
    }

    pub fn advance_with_controls(&mut self, seconds: f32, controls: Controls) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        let mut remaining = seconds;
        while remaining > 0.000_01 && self.phase == Phase::Playing {
            let dt = remaining.min(0.05);
            self.step(dt, controls);
            remaining -= dt;
        }
    }

    fn step(&mut self, dt: f32, controls: Controls) {
        self.elapsed = (self.elapsed + dt).min(NIGHT_SECONDS);
        self.sprinting = false;
        if self.stamina >= 30.0 {
            self.winded = false;
        }
        if self.outside() {
            self.searching = (self.searching - dt).max(0.0);
            let direction = controls.direction.clamp(-1, 1) as f32;
            if controls.sprint && self.stamina < 26.0 * dt {
                self.winded = true;
            }
            self.sprinting = controls.sprint && direction != 0.0 && !self.winded;
            let speed = if self.sprinting { 0.31 } else { 0.18 };
            let previous_position = self.position;
            self.position = (self.position + direction * speed * dt).clamp(0.0, 1.0);
            self.sprinting &= self.position != previous_position;
            if self.searching == 0.0 {
                self.phase = Phase::Lost;
                self.log("Время снаружи вышло. Ты потерял станцию в темноте.");
                return;
            }
        }
        self.stamina =
            (self.stamina + if self.sprinting { -26.0 } else { 14.0 } * dt).clamp(0.0, 100.0);
        let storm = self.storm();
        if storm != self.previous_storm {
            self.log(if storm {
                "Поднялась метель. Станция остывает, радиосвязь слабеет."
            } else {
                "Метель стихла. Сигнал снова проходит."
            });
            self.previous_storm = storm;
        }
        let was_powered = self.fuel > 0.0;
        self.fuel = (self.fuel - self.consumption() * dt).max(0.0);
        if was_powered && self.fuel == 0.0 {
            self.log("Топливо закончилось. Системы обесточены.");
        }
        let heating = if self.powered(1) {
            0.30
        } else {
            -0.23 * self.difficulty.resource_rate()
        };
        self.temperature = (self.temperature
            + (heating
                - if storm { 0.14 } else { 0.0 } * self.difficulty.resource_rate()
                - if self.outside() { 0.10 } else { 0.0 })
                * dt)
            .clamp(-18.0, 26.0);
        let threat_rate = if self.powered(0) { -0.90 } else { 1.10 }
            + (if self.powered(2) { 0.38 } else { 0.0 }
                + if storm { 0.48 } else { 0.0 }
                + if self.outside() { 0.52 } else { 0.0 }
                + if self.sprinting { 0.75 } else { 0.0 })
                * self.difficulty.danger_rate();
        self.threat = (self.threat + threat_rate * dt).clamp(0.0, 100.0);
        self.attack_cooldown = (self.attack_cooldown - dt).max(0.0);
        if self.threat >= 70.0 && self.attack_cooldown <= 0.0 {
            self.integrity = (self.integrity - self.difficulty.attack_damage()).max(0.0);
            self.threat = (self.threat - 9.0).max(0.0);
            self.attack_cooldown = match self.difficulty {
                Difficulty::Calm => 10.0,
                Difficulty::Night => 8.0,
                Difficulty::Nightmare => 6.0,
            };
            self.log("Кто-то ударил в ограждение. Нужно проверить периметр.");
        }
        if self.powered(2) {
            self.signal = (self.signal + if storm { 0.37 } else { 0.82 } * dt).min(100.0);
        }
        self.repair_cooldown = (self.repair_cooldown - dt).max(0.0);
        self.search_cooldown = (self.search_cooldown - dt).max(0.0);
        self.phase = if self.temperature <= -10.0 {
            Phase::Frozen
        } else if self.integrity <= 0.0 {
            Phase::Overrun
        } else if self.signal >= 100.0 {
            Phase::Rescued
        } else if self.elapsed >= NIGHT_SECONDS {
            Phase::Dawn
        } else {
            Phase::Playing
        };
    }

    fn log(&mut self, text: &'static str) {
        self.events.push(Event {
            at: self.elapsed,
            text,
        });
        if self.events.len() > 4 {
            self.events.remove(0);
        }
    }
}
