use std::time::Duration;
use tokio::time;
use crate::network::MatrixConnection;
use crate::protocol::matrixserver::{
    MatrixServerMessage, MessageType, ScreenData, Status, ScreenInfo, screen_data::Encoding
};

#[derive(PartialEq, Debug)]
enum AppState {
    Starting,
    Running,
    Paused,
}

pub async fn run(mut conn: MatrixConnection) -> std::io::Result<()> {
    let mut app_id = 0; 
    let mut state = AppState::Starting;
    let mut screens: Vec<ScreenInfo> = Vec::new();

    println!("Sending RegisterApp...");
    let mut reg_msg = MatrixServerMessage::default();
    reg_msg.message_type = MessageType::RegisterApp as i32;
    conn.send_message(&reg_msg).await?;

    let mut tick_interval = time::interval(Duration::from_millis(33)); // ~30 FPS
    let mut shift: u8 = 0;

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                if state == AppState::Running && !screens.is_empty() {
                    let mut screen_data_list = Vec::new();
                    
                    for screen in &screens {
                        let num_pixels = (screen.width * screen.height) as usize;
                        let mut frame_data = vec![0u8; num_pixels * 3];
                        
                        for y in 0..screen.height {
                            for x in 0..screen.width {
                                let i = (y * screen.width + x) as usize;
                                // Add some spatial variation to the animation (diagonal sweep)
                                let color_idx = (x + y) as u16;
                                let r = ((color_idx + shift as u16) % 255) as u8;
                                
                                frame_data[i*3] = r;       
                                frame_data[i*3 + 1] = 0;   
                                frame_data[i*3 + 2] = 255u8.saturating_sub(r); 
                            }
                        }
                        
                        let mut sd = ScreenData::default();
                        sd.screen_id = screen.screen_id;
                        sd.frame_data = frame_data;
                        sd.encoding = Encoding::Rgb24bbp as i32;
                        screen_data_list.push(sd);
                    }

                    shift = shift.wrapping_add(5);

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
