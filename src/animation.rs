use crate::protocol::matrixserver::{AppParamDef, AppParamUpdate, ScreenInfo};
use std::collections::{HashMap, HashSet};

/// Identifies animation types for type-safe registration and screen configuration
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[allow(dead_code)] // Variants may not be used initially but are for future extension
pub enum AnimationType {
    Fire,
    DiagonalSweep,
    SolidColorSweep,
    NightSky,
}

pub trait Animation: Send {
    /// Update animation state (called once per frame)
    /// Takes ScreenInfo for animations that need screen dimensions
    fn update(&mut self, screen: Option<&ScreenInfo>);

    /// Render the current state to a screen (returns RGB byte array)
    /// Note: Takes &self since rendering doesn't mutate state
    fn render(&self, screen: &ScreenInfo, rotation: Rotation) -> Vec<u8>;

    /// Get the parameter schema for this animation
    fn get_schema(&self) -> Vec<AppParamDef> {
        Vec::new()
    }

    /// Handle a parameter update from the server
    fn handle_param(&mut self, _update: &AppParamUpdate) {}
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Rotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

#[allow(dead_code)]
pub struct DiagonalSweep {
    shift: u8,
}

impl DiagonalSweep {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { shift: 0 }
    }
}

impl Animation for DiagonalSweep {
    fn update(&mut self, _screen: Option<&ScreenInfo>) {
        self.shift = self.shift.wrapping_add(5);
    }

    fn render(&self, screen: &ScreenInfo, _rotation: Rotation) -> Vec<u8> {
        let num_pixels = (screen.width * screen.height) as usize;
        let mut frame_data = vec![0u8; num_pixels * 3];

        for y in 0..screen.height {
            for x in 0..screen.width {
                let i = (y * screen.width + x) as usize;
                let color_idx = (x + y) as u16;
                let r = ((color_idx + self.shift as u16) % 255) as u8;

                frame_data[i * 3] = r;
                frame_data[i * 3 + 1] = 0;
                frame_data[i * 3 + 2] = 255u8.saturating_sub(r);
            }
        }

        frame_data
    }
}

#[allow(dead_code)]
pub struct SolidColorSweep {
    shift: u8,
}

impl SolidColorSweep {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { shift: 0 }
    }
}

impl Animation for SolidColorSweep {
    fn update(&mut self, _screen: Option<&ScreenInfo>) {
        self.shift = self.shift.wrapping_add(2);
    }

    fn render(&self, screen: &ScreenInfo, _rotation: Rotation) -> Vec<u8> {
        let num_pixels = (screen.width * screen.height) as usize;
        let mut frame_data = vec![0u8; num_pixels * 3];

        for i in 0..num_pixels {
            frame_data[i * 3] = self.shift;
            frame_data[i * 3 + 1] = 255u8.saturating_sub(self.shift);
            frame_data[i * 3 + 2] = 127;
        }

        frame_data
    }
}

pub struct Lcg {
    state: u32,
}

impl Lcg {
    pub fn new() -> Self {
        Self { state: 123456789 }
    }
    pub fn gen_range(&mut self, min: u32, max: u32) -> u32 {
        if max < min {
            return min;
        }
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        let range = max - min + 1;
        min + (self.state % range)
    }
}

pub struct FireAnimation {
    heat_map: Vec<u8>,
    width: usize,
    height: usize,
    rng: Lcg,
    cooling_multiplier: f32,
    spark_intensity: u8,
}

impl FireAnimation {
    pub fn new() -> Self {
        Self {
            heat_map: Vec::new(),
            width: 0,
            height: 0,
            rng: Lcg::new(),
            cooling_multiplier: 1.0,
            spark_intensity: 255,
        }
    }

    fn color_map(heat: u8) -> (u8, u8, u8) {
        match heat {
            0..=31 => ((heat as u16 * 8) as u8, 0, 0),
            32..=127 => (255, ((heat as u16 - 32) * 2) as u8, 0),
            128..=199 => (255, 255, ((heat as u16 - 128) * 3) as u8),
            200..=255 => (255, 255, 255),
        }
    }
}

impl Animation for FireAnimation {
    fn render(&self, screen: &ScreenInfo, rotation: Rotation) -> Vec<u8> {
        let sw = screen.width as usize;
        let sh = screen.height as usize;

        // Initialize heat_map on first render or size change (via update)
        if self.width != sw || self.height != sh {
            // This will be handled by the next update() call
        }

        let mut frame_data = vec![0u8; sw * sh * 3];

        for y_out in 0..sh {
            for x_out in 0..sw {
                let (x_in, y_in) = match rotation {
                    Rotation::Rotate0 => (x_out, y_out),
                    Rotation::Rotate90 => (y_out, (sw - 1) - x_out),
                    Rotation::Rotate180 => ((sw - 1) - x_out, (sh - 1) - y_out),
                    Rotation::Rotate270 => ((sh - 1) - y_out, x_out),
                };

                let heat_idx = y_in * self.width + x_in;
                let heat = if heat_idx < self.heat_map.len() {
                    self.heat_map[heat_idx]
                } else {
                    0
                };
                let (r, g, b) = Self::color_map(heat);

                let out_idx = (y_out * sw + x_out) * 3;
                frame_data[out_idx] = r;
                frame_data[out_idx + 1] = g;
                frame_data[out_idx + 2] = b;
            }
        }

        frame_data
    }

    fn update(&mut self, screen: Option<&ScreenInfo>) {
        // Get screen dimensions from parameter or use stored values
        let (sw, sh) = match screen {
            Some(s) => (s.width as usize, s.height as usize),
            None => (self.width, self.height),
        };

        // Initialize heat_map on first update or size change
        if self.width != sw || self.height != sh {
            self.width = sw;
            self.height = sh;
            let num_pixels = self.width * self.height;
            self.heat_map = vec![0u8; num_pixels];
        }

        if self.width == 0 || self.height == 0 {
            return;
        }

        // Fill bottom row with random bright heat (sparks)
        let bottom_row_start = (self.height - 1) * self.width;
        let min_spark = (self.spark_intensity as u16 * 160 / 255) as u32;
        let max_spark = self.spark_intensity as u32;

        for x in 0..self.width {
            self.heat_map[bottom_row_start + x] = self.rng.gen_range(min_spark, max_spark) as u8;
        }

        // Propagate heat upwards
        for y in 0..(self.height - 1) {
            for x in 0..self.width {
                // Get the pixel directly below
                let src_idx = (y + 1) * self.width + x;
                let heat = self.heat_map[src_idx];

                if heat == 0 {
                    let dst_idx = y * self.width + x;
                    self.heat_map[dst_idx] = 0;
                } else {
                    let base_cooling =
                        (255.0 / self.height.max(1) as f32 * self.cooling_multiplier) as u32;
                    let min_cooling = base_cooling / 2;
                    let max_cooling = base_cooling + 2;
                    let cooling = self.rng.gen_range(min_cooling, max_cooling) as u8;

                    let new_heat = heat.saturating_sub(cooling);

                    let spread = self.rng.gen_range(0, 2);
                    let dst_x = match spread {
                        0 => x.saturating_sub(1),
                        1 => x,
                        2 => (x + 1).min(self.width - 1),
                        _ => x,
                    };

                    let dst_idx = y * self.width + dst_x;
                    self.heat_map[dst_idx] = new_heat;
                }
            }
        }
    }

    fn get_schema(&self) -> Vec<AppParamDef> {
        vec![
            AppParamDef {
                key: "cooling".to_string(),
                label: "Cooling Multiplier".to_string(),
                r#type: "float".to_string(),
                min_val: 0.1,
                max_val: 3.0,
                step: 0.1,
                default_val: 1.0,
                ..Default::default()
            },
            AppParamDef {
                key: "intensity".to_string(),
                label: "Spark Intensity".to_string(),
                r#type: "int".to_string(),
                min_val: 50.0,
                max_val: 255.0,
                step: 1.0,
                default_val: 255.0,
                ..Default::default()
            },
        ]
    }

    fn handle_param(&mut self, update: &AppParamUpdate) {
        match update.key.as_str() {
            "cooling" => {
                self.cooling_multiplier = update.float_val;
                println!("Updated cooling multiplier to: {}", self.cooling_multiplier);
            }
            "intensity" => {
                self.spark_intensity = update.int_val as u8;
                println!("Updated spark intensity to: {}", self.spark_intensity);
            }
            _ => {}
        }
    }
}

pub struct NightSkyAnimation {
    star_x: Vec<u16>,         // X positions (scaled 0-65535)
    star_y: Vec<u16>,         // Y positions
    base_brightness: Vec<u8>, // Base brightness per star
    twinkle_phase: Vec<f32>,  // Phase offset for twinkle sine wave
    sparkle_timer: Vec<i32>,  // Timer until next sparkle check (>0 = sparkling)
    fade_timer: Vec<i32>,     // Fade-out timer after sparkle

    width: usize,
    height: usize,
    frame_count: u32, // For twinkle animation

    // Parameters
    density: u16,        // Star density multiplier (default 50 -> ~30 stars on 64x64)
    twinkle_speed: u16,  // Twinkle animation speed
    sparkle_chance: u16, // Chance per frame for sparkle burst
    background_darkness: u8, // Background brightness (0=black)
}

impl NightSkyAnimation {
    pub fn new() -> Self {
        Self {
            star_x: Vec::new(),
            star_y: Vec::new(),
            base_brightness: Vec::new(),
            twinkle_phase: Vec::new(),
            sparkle_timer: Vec::new(),
            fade_timer: Vec::new(),
            width: 0,
            height: 0,
            frame_count: 0,
            density: 50, // Default gives ~30 stars on 64x64 screen
            twinkle_speed: 50,
            sparkle_chance: 30,
            background_darkness: 5,
        }
    }

    fn regenerate_stars(&mut self) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        // Calculate number of stars based on density and screen size
        // density=50 on 64x64 gives ~30 stars (as requested)
        let num_stars =
            ((self.width as u32 * self.height as u32 * self.density as u32) / 1000).max(1) as usize;

        // Use a simple LCG for star generation
        let mut rng_state: u32 = 42; // Fixed seed for consistent stars across frames
        let next_rand = |state: &mut u32| -> u32 {
            *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            *state
        };

        self.star_x.clear();
        self.star_y.clear();
        self.base_brightness.clear();
        self.twinkle_phase.clear();
        self.sparkle_timer.clear();
        self.fade_timer.clear();

        for _ in 0..num_stars {
            let x = next_rand(&mut rng_state) as u16;
            let y = next_rand(&mut rng_state) as u16;
            let brightness = (next_rand(&mut rng_state) % 200 + 55) as u8; // 55-254
            let phase = (next_rand(&mut rng_state) % 360) as f32 * std::f32::consts::PI / 180.0;

            self.star_x.push(x);
            self.star_y.push(y);
            self.base_brightness.push(brightness);
            self.twinkle_phase.push(phase);
            self.sparkle_timer.push(0);
            self.fade_timer.push(0);
        }
    }
}

impl Animation for NightSkyAnimation {
    fn update(&mut self, screen: Option<&ScreenInfo>) {
        // Get screen dimensions from parameter or use stored values
        let (sw, sh) = match screen {
            Some(s) => (s.width as usize, s.height as usize),
            None => (self.width, self.height),
        };

        // Initialize on first update or size change
        if self.width != sw || self.height != sh {
            self.width = sw;
            self.height = sh;
            self.regenerate_stars();
        }

        if self.width == 0 || self.height == 0 {
            return;
        }

        // Increment frame counter for twinkle animation
        self.frame_count = self.frame_count.wrapping_add(1);
    }

    fn render(&self, screen: &ScreenInfo, rotation: Rotation) -> Vec<u8> {
        let sw = screen.width as usize;
        let sh = screen.height as usize;
        let num_pixels = sw * sh;

        // Initialize frame with background darkness
        let mut frame_data = vec![self.background_darkness; num_pixels * 3];

        if self.star_x.is_empty() {
            return frame_data;
        }

        // Render each star
        for i in 0..self.star_x.len() {
            // Calculate twinkle effect using sine wave
            let twinkle_angle = (self.frame_count as f32 * self.twinkle_speed as f32 / 100.0)
                + self.twinkle_phase[i];
            let twinkle_factor = twinkle_angle.sin() * 15.0; // +/- 15 brightness

            // Calculate sparkle bonus (sustained period with fade-out)
            let sparkle_bonus: u8 = if self.sparkle_timer[i] > 0 {
                // During sparkle period, add extra brightness
                let max_sparkle = 100u8;
                max_sparkle
                    .saturating_mul(self.sparkle_timer[i] as u8)
                    .saturating_div(60)
            } else if self.fade_timer[i] > 0 {
                // Fade-out after sparkle period
                50u8.saturating_mul(self.fade_timer[i] as u8)
                    .saturating_div(10)
            } else {
                0
            };

            // Calculate final brightness
            let base_brightness = self.base_brightness[i] as f32;
            let final_brightness =
                (base_brightness + twinkle_factor + sparkle_bonus as f32).clamp(0.0, 255.0) as u8;

            if final_brightness <= self.background_darkness {
                continue; // Skip if not visible above background
            }

            // Convert scaled position to screen coordinates
            let star_x = (self.star_x[i] as f32 * sw as f32 / 65535.0) as usize;
            let star_y = (self.star_y[i] as f32 * sh as f32 / 65535.0) as usize;

            // Draw star with glow effect (radius based on brightness)
            let radius = if final_brightness > 200 {
                2
            } else if final_brightness > 150 {
                1
            } else {
                0
            };

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let px = (star_x as i32 + dx).clamp(0, sw as i32 - 1) as usize;
                    let py = (star_y as i32 + dy).clamp(0, sh as i32 - 1) as usize;

                    // Distance falloff for soft glow edges
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq > radius * radius {
                        continue;
                    }

                    // Calculate pixel brightness with distance falloff
                    let falloff = match dist_sq {
                        0 => 255,
                        1 => 200,
                        _ => 128,
                    };

                    let pixel_brightness = (final_brightness as u16 * falloff as u16 / 255) as u8;

                    // Apply rotation to find output position
                    let (x_out, y_out) = match rotation {
                        Rotation::Rotate0 => (px, py),
                        Rotation::Rotate90 => ((sh - 1) - py, px),
                        Rotation::Rotate180 => ((sw - 1) - px, (sh - 1) - py),
                        Rotation::Rotate270 => (py, (sw - 1) - px),
                    };

                    let idx = (y_out * sw + x_out) * 3;

                    // Star color: white with slight blue tint for cool night sky feel
                    frame_data[idx] = pixel_brightness; // R
                    frame_data[idx + 1] = pixel_brightness; // G
                    frame_data[idx + 2] = (pixel_brightness as u16 + 30).min(255) as u8;
                    // B (slightly blue)
                }
            }
        }

        frame_data
    }

    fn get_schema(&self) -> Vec<AppParamDef> {
        vec![
            AppParamDef {
                key: "density".to_string(),
                label: "Star Density".to_string(),
                r#type: "int".to_string(),
                min_val: 10.0,
                max_val: 200.0,
                step: 1.0,
                default_val: 50.0,
                ..Default::default()
            },
            AppParamDef {
                key: "twinkle_speed".to_string(),
                label: "Twinkle Speed".to_string(),
                r#type: "int".to_string(),
                min_val: 5.0,
                max_val: 200.0,
                step: 1.0,
                default_val: 50.0,
                ..Default::default()
            },
            AppParamDef {
                key: "sparkle_chance".to_string(),
                label: "Sparkle Chance".to_string(),
                r#type: "int".to_string(),
                min_val: 0.0,
                max_val: 100.0,
                step: 1.0,
                default_val: 30.0,
                ..Default::default()
            },
            AppParamDef {
                key: "background_darkness".to_string(),
                label: "Background Darkness".to_string(),
                r#type: "int".to_string(),
                min_val: 0.0,
                max_val: 20.0,
                step: 1.0,
                default_val: 5.0,
                ..Default::default()
            },
        ]
    }

    fn handle_param(&mut self, update: &AppParamUpdate) {
        let old_density = self.density;

        match update.key.as_str() {
            "density" => {
                self.density = update.int_val as u16;
                println!("Updated star density to: {}", self.density);
            }
            "twinkle_speed" => {
                self.twinkle_speed = update.int_val as u16;
                println!("Updated twinkle speed to: {}", self.twinkle_speed);
            }
            "sparkle_chance" => {
                self.sparkle_chance = update.int_val as u16;
                println!("Updated sparkle chance to: {}", self.sparkle_chance);
            }
            "background_darkness" => {
                self.background_darkness = update.int_val as u8;
                println!(
                    "Updated background darkness to: {}",
                    self.background_darkness
                );
            }
            _ => return,
        }

        // Regenerate stars if density changed
        if old_density != self.density {
            self.regenerate_stars();
        }
    }
}

/// Registry that manages shared animation instances per type
/// Ensures synchronized animations across multiple screens
pub struct AnimationRegistry {
    instances: HashMap<AnimationType, Box<dyn Animation>>,
    active_types: HashSet<AnimationType>,
}

impl AnimationRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            active_types: HashSet::new(),
        }
    }

    /// Register FireAnimation instance (shared across all fire screens)
    pub fn register_fire(&mut self) {
        self.instances
            .insert(AnimationType::Fire, Box::new(FireAnimation::new()));
    }

    /// Register NightSkyAnimation instance
    pub fn register_night_sky(&mut self) {
        self.instances
            .insert(AnimationType::NightSky, Box::new(NightSkyAnimation::new()));
    }

    /// Register DiagonalSweep animation instance
    #[allow(dead_code)] // Available for future use
    pub fn register_diagonal_sweep(&mut self) {
        self.instances
            .insert(AnimationType::DiagonalSweep, Box::new(DiagonalSweep::new()));
    }

    /// Register SolidColorSweep animation instance
    #[allow(dead_code)] // Available for future use
    pub fn register_solid_color_sweep(&mut self) {
        self.instances.insert(
            AnimationType::SolidColorSweep,
            Box::new(SolidColorSweep::new()),
        );
    }

    /// Set which animation types are actively used by configured screens
    /// Called once at startup to enable active-only parameter filtering
    pub fn set_active_types(&mut self, types: HashSet<AnimationType>) {
        self.active_types = types;
    }

    /// Update a specific animation type with screen info (idempotent)
    pub fn update_with_screen(&mut self, anim_type: AnimationType, screen: Option<&ScreenInfo>) {
        if let Some(anim) = self.instances.get_mut(&anim_type) {
            anim.update(screen);
        }
    }

    /// Render an animation for a specific screen with rotation
    pub fn render(
        &self,
        anim_type: AnimationType,
        screen: &ScreenInfo,
        rotation: Rotation,
    ) -> Vec<u8> {
        self.instances
            .get(&anim_type)
            .map(|anim| anim.render(screen, rotation))
            .unwrap_or_default()
    }

    /// Get parameter schemas from ONLY active animation types
    pub fn get_active_schemas(&self) -> Vec<AppParamDef> {
        let mut all_params = Vec::new();
        for anim_type in self.active_types.iter() {
            if let Some(anim) = self.instances.get(anim_type) {
                let params = anim.get_schema();
                all_params.extend(params);
            }
        }
        all_params
    }

    /// Handle parameter update (applies globally to the shared animation instance)
    pub fn handle_param(&mut self, update: &AppParamUpdate) {
        // Find which registered animation has this parameter and update it
        for (_anim_type, anim) in self.instances.iter_mut() {
            anim.handle_param(update);
        }
    }
}
