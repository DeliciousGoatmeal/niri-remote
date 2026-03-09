use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, MouseButton, MouseEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use serde::Deserialize;
use std::{collections::{BTreeMap, HashMap}, env, io::{self, stdout}, process::Command, time::Instant};

#[derive(Clone, Copy)]
enum Action {
    Niri(&'static str),
    SwapMonitors,
}

#[derive(Deserialize, Debug, Clone)]
struct Workspace { id: u64, output: Option<String> }

#[derive(Deserialize, Debug, Clone)]
struct WindowLayout { pos_in_scrolling_layout: Option<Vec<u64>>, window_size: Option<Vec<u64>> }

#[derive(Deserialize, Debug, Clone)]
struct Window { id: u64, title: Option<String>, is_focused: bool, workspace_id: Option<u64>, layout: Option<WindowLayout> }

fn niri_action(args: &[&str]) {
    let _ = Command::new("niri").arg("msg").arg("action").args(args).output();
}

fn get_snapped_pct(width: f64, mon_width: f64) -> u32 {
    let raw_pct = (width / mon_width.max(1.0)) * 100.0;
    if raw_pct > 85.0 { 100 }      
    else if raw_pct > 60.0 { 67 }  
    else if raw_pct > 40.0 { 50 }  
    else if raw_pct > 28.0 { 33 }  
    else { 25 }                    
}

fn swap_monitors() {
    let out_output = match Command::new("niri").args(["msg", "-j", "outputs"]).output() {
        Ok(out) => out,
        Err(e) => {
            eprintln!("ERR: Could not communicate with Niri to fetch outputs: {}", e);
            return;
        }
    };
    let outputs_json: serde_json::Value = serde_json::from_slice(&out_output.stdout).unwrap_or_default();
    
    let mut monitors = Vec::new();
    let extract_monitor = |name: &str, val: &serde_json::Value| -> Option<(String, f64, f64)> {
        let x = val.get("logical").and_then(|l| l.get("x")).and_then(|x| x.as_f64()).unwrap_or(0.0);
        let w = val.get("logical").and_then(|l| l.get("width")).and_then(|w| w.as_f64()).unwrap_or(1920.0);
        Some((name.to_string(), x, w))
    };

    if let Some(map) = outputs_json.as_object() {
        for val in map.values() {
            if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                if let Some(mon) = extract_monitor(name, val) { monitors.push(mon); }
            }
        }
    } else if let Some(arr) = outputs_json.as_array() {
        for val in arr {
            if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                if let Some(mon) = extract_monitor(name, val) { monitors.push(mon); }
            }
        }
    }

    if monitors.len() >= 2 {
        monitors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let left_mon = &monitors[0].0;
        let left_mon_w = monitors[0].2;
        let right_mon = &monitors[1].0;
        let right_mon_w = monitors[1].2;

        let ws_output = match Command::new("niri").args(["msg", "-j", "workspaces"]).output() {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("ERR: Could not communicate with Niri to fetch workspaces: {}", e);
                return;
            }
        };
        let workspaces_json: Vec<serde_json::Value> = serde_json::from_slice(&ws_output.stdout).unwrap_or_default();
        
        let mut left_ws_id = None;
        let mut right_ws_id = None;

        for ws in workspaces_json {
            if ws.get("is_active").and_then(|a| a.as_bool()).unwrap_or(false) {
                if let Some(out) = ws.get("output").and_then(|o| o.as_str()) {
                    if out == left_mon { left_ws_id = ws.get("id").and_then(|id| id.as_u64()); } 
                    else if out == right_mon { right_ws_id = ws.get("id").and_then(|id| id.as_u64()); }
                }
            }
        }

        let win_output = match Command::new("niri").args(["msg", "-j", "windows"]).output() {
            Ok(win) => win,
            Err(e) => {
                eprintln!("ERR: Could not communicate with Niri to fetch windows: {}", e);
                return;
            }
        };
        let windows_json: Vec<serde_json::Value> = serde_json::from_slice(&win_output.stdout).unwrap_or_default();

        let mut left_windows: Vec<(u64, u32, bool)> = Vec::new();
        let mut right_windows: Vec<(u64, u32, bool)> = Vec::new();

        for win in windows_json {
            if let Some(ws_id) = win.get("workspace_id").and_then(|id| id.as_u64()) {
                if let Some(id) = win.get("id").and_then(|id| id.as_u64()) {
                    let width = win.get("layout").and_then(|l| l.get("window_size")).and_then(|s| s.get(0)).and_then(|w| w.as_f64()).unwrap_or(1000.0);
                    let is_fs = win.get("is_fullscreen").and_then(|f| f.as_bool()).unwrap_or(false);

                    if Some(ws_id) == left_ws_id {
                        left_windows.push((id, get_snapped_pct(width, left_mon_w), is_fs));
                    } else if Some(ws_id) == right_ws_id {
                        right_windows.push((id, get_snapped_pct(width, right_mon_w), is_fs));
                    }
                }
            }
        }

        if let Some(&(l_win, _, _)) = left_windows.first() {
            let _ = Command::new("niri").args(["msg", "action", "focus-window", "--id", &l_win.to_string()]).output();
            let _ = Command::new("niri").args(["msg", "action", "move-workspace-to-monitor-right"]).output();
        }
        if let Some(&(r_win, _, _)) = right_windows.first() {
            let _ = Command::new("niri").args(["msg", "action", "focus-window", "--id", &r_win.to_string()]).output();
            let _ = Command::new("niri").args(["msg", "action", "move-workspace-to-monitor-left"]).output();
        }

        for (id, pct, is_fs) in &left_windows {
            if !is_fs {
                let _ = Command::new("niri").args(["msg", "action", "focus-window", "--id", &id.to_string()]).output();
                let _ = Command::new("niri").args(["msg", "action", "set-column-width", &format!("{}%", pct)]).output();
            }
        }
        if let Some(&(first_id, _, _)) = left_windows.first() {
            let _ = Command::new("niri").args(["msg", "action", "focus-window", "--id", &first_id.to_string()]).output();
        }

        for (id, pct, is_fs) in &right_windows {
            if !is_fs {
                let _ = Command::new("niri").args(["msg", "action", "focus-window", "--id", &id.to_string()]).output();
                let _ = Command::new("niri").args(["msg", "action", "set-column-width", &format!("{}%", pct)]).output();
            }
        }
        if let Some(&(first_id, _, _)) = right_windows.first() {
            let _ = Command::new("niri").args(["msg", "action", "focus-window", "--id", &first_id.to_string()]).output();
        }
    }
}

fn main() -> io::Result<()> {
    if let Err(e) = Command::new("niri").arg("--version").output() {
        if e.kind() == std::io::ErrorKind::NotFound {
            eprintln!("WARN: The 'niri' command was not found on this system.");
            eprintln!("ERR: niri-remote requires the Niri Wayland compositor to be installed and accessible in the system PATH.");
            eprintln!("Starting up in Empty Mode in 2 seconds...");
            
            // Give the user 2 seconds to read the warning before the UI clears the screen!
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "swap" => {
                println!("Swapping monitor workspaces and resizing proportionally...");
                swap_monitors();
                return Ok(());
            },
            "list" => {
                let win_output = match Command::new("niri").args(["msg", "-j", "windows"]).output() {
                    Ok(o) => o,
                    Err(_) => return Ok(()),
                };
                let windows: Vec<Window> = serde_json::from_slice(&win_output.stdout).unwrap_or_default();
                
                println!("\n{:<10} | {}", "WINDOW ID", "TITLE");
                println!("{:-<10}-+-{:-<50}", "", "");
                for win in windows {
                    let title = win.title.unwrap_or_else(|| "Unknown".to_string());
                    println!("{:<10} | {}", win.id, title);
                }
                println!();
                return Ok(());
            },
            "close" | "fullscreen" => {
                if args.len() < 3 {
                    println!("Usage: niri-remote {} <window_id_or_title>", args[1]);
                    return Ok(());
                }

                let action_cmd = args[1].as_str();
                let win_arg = &args[2];

                let win_output = match Command::new("niri").args(["msg", "-j", "windows"]).output() {
                    Ok(o) => o,
                    Err(_) => return Ok(()),
                };
                let windows: Vec<Window> = serde_json::from_slice(&win_output.stdout).unwrap_or_default();
                
                let target_win_id = if let Ok(id) = win_arg.parse::<u64>() { id } else {
                    let lower_arg = win_arg.to_lowercase();
                    if let Some(w) = windows.iter().find(|w| w.title.as_deref().unwrap_or("").to_lowercase().contains(&lower_arg)) { w.id } else {
                        eprintln!("ERR: Could not find open window matching '{}'", win_arg);
                        return Ok(());
                    }
                };

                let niri_cmd = if action_cmd == "close" { "close-window" } else { "fullscreen-window" };
                println!("Sending '{}' to window ID {}...", action_cmd, target_win_id);
                
                let id_str = target_win_id.to_string();
                niri_action(&["focus-window", "--id", &id_str]);
                niri_action(&[niri_cmd]);
                
                return Ok(());
            },
            "move" => {
                if args.len() < 5 || args[3] != "to" {
                    println!("Usage: niri-remote move <window_id_or_title> to <display_id_or_name>");
                    return Ok(());
                }
                
                let win_arg = &args[2];
                let mon_arg = &args[4];

                let win_output = match Command::new("niri").args(["msg", "-j", "windows"]).output() {
                    Ok(o) => o,
                    Err(_) => return Ok(()),
                };
                let windows: Vec<Window> = serde_json::from_slice(&win_output.stdout).unwrap_or_default();
                
                let target_win_id = if let Ok(id) = win_arg.parse::<u64>() { id } else {
                    let lower_arg = win_arg.to_lowercase();
                    if let Some(w) = windows.iter().find(|w| w.title.as_deref().unwrap_or("").to_lowercase().contains(&lower_arg)) { w.id } else {
                        eprintln!("ERR: Could not find open window matching '{}'", win_arg);
                        return Ok(());
                    }
                };

                let out_output = match Command::new("niri").args(["msg", "-j", "outputs"]).output() {
                    Ok(o) => o,
                    Err(_) => return Ok(()),
                };
                let outputs_json: serde_json::Value = serde_json::from_slice(&out_output.stdout).unwrap_or_default();
                let mut monitors = Vec::new();
                
                if let Some(map) = outputs_json.as_object() {
                    for val in map.values() {
                        if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                            let x = val.get("logical").and_then(|l| l.get("x")).and_then(|x| x.as_f64()).unwrap_or(0.0);
                            monitors.push((name.to_string(), x));
                        }
                    }
                } else if let Some(arr) = outputs_json.as_array() {
                    for val in arr {
                        if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                            let x = val.get("logical").and_then(|l| l.get("x")).and_then(|x| x.as_f64()).unwrap_or(0.0);
                            monitors.push((name.to_string(), x));
                        }
                    }
                }

                if monitors.is_empty() {
                    let ws_output = match Command::new("niri").args(["msg", "-j", "workspaces"]).output() {
                        Ok(o) => o,
                        Err(_) => return Ok(()),
                    };
                    let workspaces: Vec<Workspace> = serde_json::from_slice(&ws_output.stdout).unwrap_or_default();
                    for ws in workspaces {
                        if let Some(output) = ws.output {
                            if !monitors.iter().any(|(n, _)| n == &output) { monitors.push((output, 0.0)); }
                        }
                    }
                }

                monitors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                
                let target_mon_name = if let Ok(idx) = mon_arg.parse::<usize>() {
                    if idx > 0 && idx <= monitors.len() { monitors[idx - 1].0.clone() } else {
                        eprintln!("ERR: Display number {} is out of range.", idx);
                        return Ok(());
                    }
                } else { mon_arg.clone() };

                println!("Moving window ID {} to display '{}'...", target_win_id, target_mon_name);
                let id_str = target_win_id.to_string();
                niri_action(&["focus-window", "--id", &id_str]);
                niri_action(&["move-window-to-monitor", &target_mon_name]);
                
                return Ok(());
            },
            _ => {
                println!("Unknown command. Use 'list', 'move', 'close', 'fullscreen', 'swap', or run without arguments to start the UI.");
                return Ok(());
            }
        }
    }

    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut click_map_windows: Vec<(Rect, u64)> = Vec::new();
    let mut click_map_buttons: Vec<(Rect, Action)> = Vec::new();

    let mut needs_refresh = true;
    let mut last_refresh = Instant::now();
    let mut ordered_monitors: Vec<String> = Vec::new();
    let mut monitors_data: BTreeMap<String, BTreeMap<u64, Vec<Window>>> = BTreeMap::new();

    loop {
        if needs_refresh || last_refresh.elapsed().as_millis() > 250 {
            ordered_monitors.clear();
            monitors_data.clear();

            // NEW: Safe parsing so the UI doesn't crash if Niri is missing
            let outputs_json: serde_json::Value = Command::new("niri")
                .args(["msg", "-j", "outputs"])
                .output()
                .map(|o| serde_json::from_slice(&o.stdout).unwrap_or_default())
                .unwrap_or(serde_json::Value::Null);
            
            let mut monitors_sortable = Vec::new();
            if let Some(map) = outputs_json.as_object() {
                for val in map.values() {
                    if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                        let x = val.get("logical").and_then(|l| l.get("x")).and_then(|x| x.as_f64()).unwrap_or(0.0);
                        monitors_sortable.push((name.to_string(), x));
                    }
                }
            } else if let Some(arr) = outputs_json.as_array() {
                for val in arr {
                    if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                        let x = val.get("logical").and_then(|l| l.get("x")).and_then(|x| x.as_f64()).unwrap_or(0.0);
                        monitors_sortable.push((name.to_string(), x));
                    }
                }
            }

            monitors_sortable.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            ordered_monitors = monitors_sortable.into_iter().map(|m| m.0).collect();

            let workspaces: Vec<Workspace> = Command::new("niri")
                .args(["msg", "-j", "workspaces"])
                .output()
                .map(|o| serde_json::from_slice(&o.stdout).unwrap_or_default())
                .unwrap_or_default();
            
            let mut ws_to_monitor: HashMap<u64, String> = HashMap::new();
            for ws in &workspaces {
                if let Some(output) = &ws.output {
                    ws_to_monitor.insert(ws.id, output.clone());
                    if !ordered_monitors.contains(output) { ordered_monitors.push(output.clone()); }
                }
            }

            let windows: Vec<Window> = Command::new("niri")
                .args(["msg", "-j", "windows"])
                .output()
                .map(|o| serde_json::from_slice(&o.stdout).unwrap_or_default())
                .unwrap_or_default();

            for win in windows {
                if let Some(ws_id) = win.workspace_id {
                    if let Some(monitor_name) = ws_to_monitor.get(&ws_id) {
                        if let Some(layout) = &win.layout {
                            if let Some(pos) = &layout.pos_in_scrolling_layout {
                                if pos.len() == 2 {
                                    monitors_data.entry(monitor_name.clone()).or_default().entry(pos[0]).or_default().push(win.clone());
                                }
                            }
                        }
                    }
                }
            }

            for cols in monitors_data.values_mut() {
                for col_windows in cols.values_mut() {
                    col_windows.sort_by_key(|w| w.layout.as_ref().unwrap().pos_in_scrolling_layout.as_ref().unwrap()[1]);
                }
            }
            needs_refresh = false;
            last_refresh = Instant::now();
        }

        terminal.draw(|frame| {
            click_map_windows.clear();
            click_map_buttons.clear();
            let area = frame.area();

            let main_chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(5), Constraint::Length(9)]).split(area);

            let num_monitors = ordered_monitors.len() as u32;
            if num_monitors == 0 {
                // If Niri isn't running, they get exactly what they asked for: The app boots, and gives a blank UI!
                frame.render_widget(Paragraph::new("No windows found! (Is Niri running?)").alignment(Alignment::Center), main_chunks[0]);
            } else {
                let mon_constraints = vec![Constraint::Ratio(1, num_monitors); num_monitors as usize];
                let monitor_chunks = Layout::default().direction(Direction::Horizontal).constraints(mon_constraints).split(main_chunks[0]);

                for (mon_index, monitor_name) in ordered_monitors.iter().enumerate() {
                    let mon_block = Block::default().title(format!(" {} ", monitor_name)).style(Style::default().bg(Color::Reset)); 
                    let mon_inner_area = mon_block.inner(monitor_chunks[mon_index]);
                    frame.render_widget(mon_block, monitor_chunks[mon_index]);

                    if let Some(columns) = monitors_data.get(monitor_name) {
                        if !columns.is_empty() {
                            let mut total_width: u32 = 0;
                            let mut col_widths: Vec<u32> = Vec::new();
                            
                            for col_windows in columns.values() {
                                let w = col_windows.first().and_then(|win| win.layout.as_ref()).and_then(|l| l.window_size.as_ref()).and_then(|s| s.get(0)).copied().unwrap_or(100) as u32; 
                                col_widths.push(w);
                                total_width += w;
                            }

                            let mut col_constraints = Vec::new();
                            for w in col_widths { col_constraints.push(Constraint::Ratio(w, total_width.max(1))); }

                            let horizontal_chunks = Layout::default().direction(Direction::Horizontal).constraints(col_constraints).split(mon_inner_area);

                            for (col_index, (_col_id, col_windows)) in columns.iter().enumerate() {
                                let mut total_height: u32 = 0;
                                let mut win_heights: Vec<u32> = Vec::new();

                                for win in col_windows.iter() {
                                    let h = win.layout.as_ref().and_then(|l| l.window_size.as_ref()).and_then(|s| s.get(1)).copied().unwrap_or(100) as u32;
                                    win_heights.push(h);
                                    total_height += h;
                                }

                                let mut win_constraints = Vec::new();
                                for h in win_heights { win_constraints.push(Constraint::Ratio(h, total_height.max(1))); }

                                let vertical_chunks = Layout::default().direction(Direction::Vertical).constraints(win_constraints).split(horizontal_chunks[col_index]);

                                for (win_index, window) in col_windows.iter().enumerate() {
                                    let chunk = vertical_chunks[win_index];
                                    click_map_windows.push((chunk, window.id));

                                    let title = window.title.clone().unwrap_or_else(|| "Unknown".to_string());
                                    let (border_color, title_style) = if window.is_focused {
                                        (Color::Green, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                                    } else {
                                        (Color::DarkGray, Style::default().fg(Color::Gray))
                                    };

                                    let block = Block::default().title(title).title_style(title_style).borders(Borders::ALL).border_style(Style::default().fg(border_color));
                                    frame.render_widget(Paragraph::new("").block(block), chunk);
                                }
                            }
                        }
                    }
                }
            }

            let toolbar_rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
                .split(main_chunks[1]);
                
            let row1_chunks = Layout::default().direction(Direction::Horizontal).constraints(vec![Constraint::Ratio(1, 4); 4]).split(toolbar_rows[0]);
            let row2_chunks = Layout::default().direction(Direction::Horizontal).constraints(vec![Constraint::Ratio(1, 4); 4]).split(toolbar_rows[1]);
            let row3_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(100)]).split(toolbar_rows[2]);

            let btn_style = Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD);
            
            let btns_r1 = [
                ("<< TO SCREEN", Action::Niri("move-window-to-monitor-left")),
                ("< LEFT", Action::Niri("move-window-left")),
                ("RIGHT >", Action::Niri("move-window-right")),
                ("TO SCREEN >>", Action::Niri("move-window-to-monitor-right")),
            ];
            let btns_r2 = [
                ("v DOWN", Action::Niri("move-window-down")),
                ("^ UP", Action::Niri("move-window-up")),
                ("[ ] FULLSCREEN", Action::Niri("fullscreen-window")), 
                ("X CLOSE", Action::Niri("close-window")),             
            ];

            for i in 0..4 {
                let btn_block = Paragraph::new(btns_r1[i].0).alignment(Alignment::Center).style(btn_style).block(Block::default().borders(Borders::ALL));
                frame.render_widget(btn_block, row1_chunks[i]);
                click_map_buttons.push((row1_chunks[i], btns_r1[i].1));
            }
            for i in 0..4 {
                let btn_block = Paragraph::new(btns_r2[i].0).alignment(Alignment::Center).style(btn_style).block(Block::default().borders(Borders::ALL));
                frame.render_widget(btn_block, row2_chunks[i]);
                click_map_buttons.push((row2_chunks[i], btns_r2[i].1));
            }

            let swap_btn_style = Style::default().fg(Color::Cyan).bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            let swap_btn_block = Paragraph::new("🔄 SWAP SCREENS")
                .alignment(Alignment::Center)
                .style(swap_btn_style)
                .block(Block::default().borders(Borders::ALL));
            
            frame.render_widget(swap_btn_block, row3_chunks[0]);
            click_map_buttons.push((row3_chunks[0], Action::SwapMonitors));

        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                event::Event::Key(key) => {
                    let shift = key.modifiers.contains(event::KeyModifiers::SHIFT);
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                            if shift || key.code == KeyCode::Char('H') { niri_action(&["move-window-left"]); } else { niri_action(&["focus-column-left"]); }
                        }
                        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                            if shift || key.code == KeyCode::Char('L') { niri_action(&["move-window-right"]); } else { niri_action(&["focus-column-right"]); }
                        }
                        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                            if shift || key.code == KeyCode::Char('K') { niri_action(&["move-window-up"]); } else { niri_action(&["focus-window-up"]); }
                        }
                        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                            if shift || key.code == KeyCode::Char('J') { niri_action(&["move-window-down"]); } else { niri_action(&["focus-window-down"]); }
                        }
                        _ => {}
                    }
                    needs_refresh = true;
                },
                event::Event::Mouse(mouse_event) => {
                    if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                        let tap_x = mouse_event.column;
                        let tap_y = mouse_event.row;

                        for (rect, win_id) in &click_map_windows {
                            if tap_x >= rect.x && tap_x < rect.x + rect.width && tap_y >= rect.y && tap_y < rect.y + rect.height {
                                let id_str = win_id.to_string();
                                niri_action(&["focus-window", "--id", &id_str]);
                                needs_refresh = true; 
                                break;
                            }
                        }

                        for (rect, action) in &click_map_buttons {
                            if tap_x >= rect.x && tap_x < rect.x + rect.width && tap_y >= rect.y && tap_y < rect.y + rect.height {
                                match action {
                                    Action::Niri(cmd) => niri_action(&[*cmd]),
                                    Action::SwapMonitors => swap_monitors(),
                                }
                                needs_refresh = true;
                                break;
                            }
                        }
                    }
                },
                _ => {}
            }
        }
    }

    stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
