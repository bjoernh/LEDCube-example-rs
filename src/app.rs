use std::time::Duration;
use tokio::time;
use crate::network::MatrixConnection;
use crate::protocol::matrixserver::{
    MatrixServerMessage, MessageType, ScreenData, Status, ScreenInfo, screen_data::Encoding,
    AppParamSchema
};
use crate::animation::{Animation, Rotation};
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
enum AppState {
    Starting,
    Running,
    Paused,
}

pub async fn run(mut conn: MatrixConnection, mut animation: Box<dyn Animation>, screen_rotations: HashMap<i32, Rotation>) -> std::io::Result<()> {
    let mut app_id = 0; 
    let mut state = AppState::Starting;
    let mut screens: Vec<ScreenInfo> = Vec::new();

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
                    
                    for screen in &screens {
                        let mut sd = ScreenData::default();
                        sd.screen_id = screen.screen_id;
                        sd.encoding = Encoding::Rgb24bbp as i32;

                        if let Some(&rotation) = screen_rotations.get(&screen.screen_id) {
                            sd.frame_data = animation.render(screen, rotation);
                        } else {
                            // Send black frame for inactive screens
                            sd.frame_data = vec![0u8; (screen.width * screen.height * 3) as usize];
                        }
                        
                        screen_data_list.push(sd);
                    }

                    animation.update();

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

                                    // Send parameter schema
                                    let mut schema_msg = MatrixServerMessage::default();
                                    schema_msg.message_type = MessageType::AppParamSchema as i32;
                                    schema_msg.app_id = app_id;
                                    schema_msg.app_param_schema = Some(AppParamSchema {
                                        app_name: "LEDCube-Rust-Fire".to_string(),
                                        params: animation.get_schema(),
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
                                    animation.handle_param(&update);
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
