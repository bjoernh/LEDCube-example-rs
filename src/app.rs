#![allow(clippy::field_reassign_with_default)]

use std::time::Duration;
use tokio::time;
use crate::network::MatrixConnection;
use crate::protocol::matrixserver::{
    MatrixServerMessage, MessageType, ScreenData, Status, ScreenInfo, screen_data::Encoding,
    AppParamSchema
};
use crate::animation::{AnimationType, AnimationRegistry, Rotation};
use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Debug)]
enum AppState {
    Starting,
    Running,
    Paused,
}

/// Configuration for a single screen's animation and rotation
#[derive(Clone)]
pub struct ScreenConfig {
    pub animation_type: AnimationType,
    pub rotation: Rotation,
}

pub async fn run(
    mut conn: MatrixConnection,
    mut registry: AnimationRegistry,
    screen_configs: HashMap<i32, ScreenConfig>,
) -> std::io::Result<()> {
    let mut app_id = 0; 
    let mut state = AppState::Starting;
    let mut screens: Vec<ScreenInfo> = Vec::new();

    // Compute active animation types from screen configuration (once at startup)
    let active_types: HashSet<AnimationType> = screen_configs.values()
        .map(|config| config.animation_type)
        .collect();
    registry.set_active_types(active_types);

    println!("Sending RegisterApp...");
    let mut reg_msg = MatrixServerMessage::default();
    reg_msg.message_type = MessageType::RegisterApp as i32;
    conn.send_message(&reg_msg).await?;

    let mut tick_interval = time::interval(Duration::from_millis(33)); // ~30 FPS

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                if state == AppState::Running && !screens.is_empty() {
                    let mut screen_data_list = Vec::new();

                    // Update and render for each screen
                    for screen in &screens {
                        let mut sd = ScreenData::default();
                        sd.screen_id = screen.screen_id;
                        sd.encoding = Encoding::Rgb24bbp as i32;

                        match screen_configs.get(&screen.screen_id) {
                            Some(config) => {
                                // Update this animation type with screen info (for initialization)
                                registry.update_with_screen(config.animation_type, Some(screen));
                                
                                // Render with configured rotation
                                sd.frame_data = registry.render(
                                    config.animation_type,
                                    screen,
                                    config.rotation,
                                );
                            }
                            None => {
                                // Unmapped screens stay black (explicit opt-in)
                                sd.frame_data = vec![0u8; (screen.width * screen.height * 3) as usize];
                            }
                        }

                        screen_data_list.push(sd);
                    }

                    let mut frame_msg = MatrixServerMessage::default();
                    frame_msg.message_type = MessageType::SetScreenFrame as i32;
                    frame_msg.app_id = app_id;
                    frame_msg.screen_data = screen_data_list;

                    if let Err(e) = conn.send_message(&frame_msg).await {
                        eprintln!("Failed to send frame: {}", e);
                        break;
                    }
                }
            }
            result = conn.read_message() => {
                match result {
                    Ok(Some(msg)) => {
                        let msg_type = MessageType::try_from(msg.message_type).unwrap_or(MessageType::DefaultMessageType);
                        
                        match msg_type {
                            MessageType::RegisterApp => {
                                if msg.status == Status::Success as i32 {
                                    app_id = msg.app_id;
                                    println!("Registered successfully! Assigned App ID: {}", app_id);
                                    
                                    let mut info_req = MatrixServerMessage::default();
                                    info_req.message_type = MessageType::GetServerInfo as i32;
                                    info_req.app_id = app_id;
                                    conn.send_message(&info_req).await?;

                                    // Send parameter schema (only active animations)
                                    let mut schema_msg = MatrixServerMessage::default();
                                    schema_msg.message_type = MessageType::AppParamSchema as i32;
                                    schema_msg.app_id = app_id;
                                    schema_msg.app_param_schema = Some(AppParamSchema {
                                        app_name: "LEDCube-Rust-Multi".to_string(),
                                        params: registry.get_active_schemas(),
                                    });
                                    conn.send_message(&schema_msg).await?;
                                } else {
                                    eprintln!("Registration failed with status: {:?}", msg.status);
                                    break;
                                }
                            }
                            MessageType::GetServerInfo => {
                                if let Some(config) = msg.server_config {
                                    println!("Received ServerConfig with {} screens.", config.screen_info.len());
                                    screens = config.screen_info;
                                    for s in &screens {
                                        println!("  Screen {}: {}x{}", s.screen_id, s.width, s.height);
                                    }
                                    state = AppState::Running;
                                }
                            }
                            MessageType::AppPause => {
                                println!("Server requested Pause.");
                                state = AppState::Paused;
                            }
                            MessageType::AppResume => {
                                println!("Server requested Resume.");
                                state = AppState::Running;
                            }
                            MessageType::AppKill => {
                                println!("Server requested Kill. Shutting down.");
                                break;
                            }
                            MessageType::SetAppParam => {
                                if let Some(update) = msg.app_param_update {
                                    registry.handle_param(&update);
                                }
                            }
                            MessageType::GetAppParams => {
                                // Normally we'd send current values back, but for now we'll just log
                                println!("Server requested current parameter values.");
                            }
                            _ => {}
                        }
                    }
                    Ok(None) => {
                        println!("Connection closed by server.");
                        break;
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
