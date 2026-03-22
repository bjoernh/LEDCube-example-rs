use crate::protocol::matrixserver::{ScreenInfo, AppParamDef, AppParamUpdate};

pub trait Animation: Send {
    /// Update animation state (called once per frame)
    fn update(&mut self);
    
    /// Render the current state to a screen (returns RGB byte array)
    fn render(&mut self, screen: &ScreenInfo, rotation: Rotation) -> Vec<u8>;

    /// Get the parameter schema for this animation
    fn get_schema(&self) -> Vec<AppParamDef> { Vec::new() }

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
    fn update(&mut self) {
        self.shift = self.shift.wrapping_add(5);
    }

    fn render(&mut self, screen: &ScreenInfo, _rotation: Rotation) -> Vec<u8> {
        let num_pixels = (screen.width * screen.height) as usize;
        let mut frame_data = vec![0u8; num_pixels * 3];
        
        for y in 0..screen.height {
            for x in 0..screen.width {
                let i = (y * screen.width + x) as usize;
                let color_idx = (x + y) as u16;
                let r = ((color_idx + self.shift as u16) % 255) as u8;
                
                frame_data[i*3] = r;       
                frame_data[i*3 + 1] = 0;   
                frame_data[i*3 + 2] = 255u8.saturating_sub(r); 
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
    fn update(&mut self) {
        self.shift = self.shift.wrapping_add(2);
    }

    fn render(&mut self, screen: &ScreenInfo, _rotation: Rotation) -> Vec<u8> {
        let num_pixels = (screen.width * screen.height) as usize;
        let mut frame_data = vec![0u8; num_pixels * 3];
        
        for i in 0..num_pixels {
            frame_data[i*3] = self.shift;
            frame_data[i*3 + 1] = 255u8.saturating_sub(self.shift);
            frame_data[i*3 + 2] = 127; 
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
        if max < min { return min; }
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
    fn update(&mut self) {
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
                    let base_cooling = (255.0 / self.height.max(1) as f32 * self.cooling_multiplier) as u32;
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

    fn render(&mut self, screen: &ScreenInfo, rotation: Rotation) -> Vec<u8> {
        let sw = screen.width as usize;
        let sh = screen.height as usize;

        if self.width != sw || self.height != sh {
            self.width = sw;
            self.height = sh;
            let num_pixels = self.width * self.height;
            self.heat_map = vec![0u8; num_pixels];
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
                let heat = self.heat_map[heat_idx];
                let (r, g, b) = Self::color_map(heat);
                
                let out_idx = (y_out * sw + x_out) * 3;
                frame_data[out_idx] = r;
                frame_data[out_idx + 1] = g;
                frame_data[out_idx + 2] = b;
            }
        }

        frame_data
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
