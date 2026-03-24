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
