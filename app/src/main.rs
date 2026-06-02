#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use install::perform_installation;
use state::{AppField, AppState, Step};
use wifi::{connect_wifi, discover_wifi};

fn get_disks() -> Vec<String> {
    let output = std::process::Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME", "-e", "7,11"])
        .output();

    if let Ok(out) = output {
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| format!("/dev/{}", l.trim()))
            .collect()
    } else {
        Vec::new()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let disks = get_disks();
    let state = Arc::new(Mutex::new(AppState::new(disks)));

    tokio::spawn(discover_wifi(state.clone()));

    let tick_rate = Duration::from_millis(100);

    loop {
        let current_step = { state.lock().unwrap().step };

        terminal.draw(|f| {
            let s = state.lock().unwrap();
            ui::draw_ui(f, &s);
        })?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                let mut s = state.lock().unwrap();

                if !s.wifi_input_mode 
                    && s.active_field != AppField::Hostname 
                    && s.active_field != AppField::Username 
                    && (key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q')) 
                {
                    break;
                }

                match s.step {
                    Step::Wifi => {
                        if s.wifi_input_mode {
                            match key.code {
                                KeyCode::Enter => {
                                    if s.wifi_idx == 0 {
                                        s.wifi_input_mode = false;
                                        s.step = Step::Configure;
                                    } else {
                                        let ssid = s.wifi_list[s.wifi_idx].clone();
                                        let pass = s.wifi_password.clone();
                                        tokio::spawn(connect_wifi(state.clone(), ssid, pass));
                                    }
                                }
                                KeyCode::Backspace => {
                                    s.wifi_password.pop();
                                }
                                KeyCode::Char(c) => {
                                    s.wifi_password.push(c);
                                }
                                KeyCode::Esc => {
                                    s.wifi_input_mode = false;
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Up if s.wifi_idx > 0 => s.wifi_idx -= 1,
                                KeyCode::Down if s.wifi_idx < s.wifi_list.len() - 1 => s.wifi_idx += 1,
                                KeyCode::Char('s') | KeyCode::Char('S') => {
                                    s.step = Step::Configure;
                                }
                                KeyCode::Enter => {
                                    if s.wifi_idx == 0 {
                                        s.step = Step::Configure;
                                    } else {
                                        s.wifi_password.clear();
                                        s.wifi_input_mode = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Step::Configure => match key.code {
                        KeyCode::Up => match s.active_field {
                            AppField::Disk => s.active_field = AppField::Gpu,
                            AppField::HorizontalLayout => {}
                            AppField::Hostname => s.active_field = AppField::Disk,
                            AppField::Username => s.active_field = AppField::Hostname,
                            AppField::Locale => s.active_field = AppField::Username,
                            AppField::Timezone => s.active_field = AppField::Locale,
                            AppField::Keymap => s.active_field = AppField::Timezone,
                            AppField::SwapSize => s.active_field = AppField::Keymap,
                            AppField::Dm => s.active_field = AppField::SwapSize,
                            AppField::Gpu => s.active_field = AppField::Dm,
                        },
                        KeyCode::Down => match s.active_field {
                            AppField::Disk => s.active_field = AppField::Hostname,
                            AppField::HorizontalLayout => {}
                            AppField::Hostname => s.active_field = AppField::Username,
                            AppField::Username => s.active_field = AppField::Locale,
                            AppField::Locale => s.active_field = AppField::Timezone,
                            AppField::Timezone => s.active_field = AppField::Keymap,
                            AppField::Keymap => s.active_field = AppField::SwapSize,
                            AppField::SwapSize => s.active_field = AppField::Dm,
                            AppField::Dm => s.active_field = AppField::Gpu,
                            AppField::Gpu => s.active_field = AppField::Disk,
                        },
                        KeyCode::Left => match s.active_field {
                            AppField::Disk if s.disk_idx > 0 => s.disk_idx -= 1,
                            AppField::Locale if s.locale_idx > 0 => s.locale_idx -= 1,
                            AppField::Timezone if s.timezone_idx > 0 => s.timezone_idx -= 1,
                            AppField::Keymap if s.keymap_idx > 0 => s.keymap_idx -= 1,
                            AppField::Dm if s.dm_idx > 0 => s.dm_idx -= 1,
                            AppField::Gpu if s.gpu_idx > 0 => s.gpu_idx -= 1,
                            _ => {}
                        },
                        KeyCode::Right => match s.active_field {
                            AppField::Disk if !s.disks.is_empty() && s.disk_idx < s.disks.len() - 1 => s.disk_idx += 1,
                            AppField::Locale if s.locale_idx < constants::LOCALES.len() - 1 => s.locale_idx += 1,
                            AppField::Timezone if s.timezone_idx < constants::TIMEZONES.len() - 1 => s.timezone_idx += 1,
                            AppField::Keymap if s.keymap_idx < constants::KEYMAPS.len() - 1 => s.keymap_idx += 1,
                            AppField::Dm if s.dm_idx < constants::DMS.len() - 1 => s.dm_idx += 1,
                            AppField::Gpu if s.gpu_idx < constants::GPUS.len() - 1 => s.gpu_idx += 1,
                            _ => {}
                        },
                        KeyCode::Char(c) => match s.active_field {
                            AppField::Hostname => s.hostname.push(c),
                            AppField::Username => {
                                if c.is_lowercase() || c.is_numeric() || c == '_' || c == '-' {
                                    s.username.push(c);
                                }
                            }
                            AppField::SwapSize => {
                                if let Some(digit) = c.to_digit(10) {
                                    let val = s.swap_size * 10 + digit;
                                    if val <= 64 { s.swap_size = val; }
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Backspace => match s.active_field {
                            AppField::Hostname => { s.hostname.pop(); }
                            AppField::Username => { s.username.pop(); }
                            AppField::SwapSize => { s.swap_size /= 10; }
                            _ => {}
                        },
                        KeyCode::Enter if s.can_continue() => {
                            s.step = Step::Review;
                        }
                        _ => {}
                    },
                    Step::Review => match key.code {
                        KeyCode::Esc => s.step = Step::Configure,
                        KeyCode::Enter => {
                            s.step = Step::Install;
                            let config = s.current_config();
                            let state_c = state.clone();
                            tokio::spawn(async move {
                                if let Err(e) = perform_installation(state_c.clone(), config).await {
                                    let mut lock = state_c.lock().unwrap();
                                    lock.error_message = Some(e.clone());
                                    lock.logs.push(format!("[FATAL ERROR] {}", e));
                                }
                            });
                        }
                        _ => {}
                    },
                    Step::Install => {}
                    Step::Success => {
                        if key.code == KeyCode::Enter {
                            let _ = std::process::Command::new("reboot").spawn();
                            break;
                        }
                    }
                }
            }
        }
        if current_step == Step::Success {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}