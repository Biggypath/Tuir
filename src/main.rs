#[allow(unused)]
use std::{io, thread, time::Duration};
#[allow(unused)]
use ratatui::{
    backend::{CrosstermBackend, Backend},
    widgets::{Widget, Block, Borders, Paragraph, List, ListItem, Wrap, Scrollbar, ScrollbarState, ScrollbarOrientation, BarChart, Bar, BarGroup, Gauge, Chart, Dataset},
    layout::{Layout, Constraint, Direction,Alignment,},
    Terminal,
    Frame,
    style::{Style, Color, Stylize, Modifier},
    text::{Line, Span}, 
    symbols::scrollbar,
    prelude::Margin
};
use battery::units::ratio::percent;

#[allow(unused)]
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind, KeyEventState,},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, ClearType, Clear},
    cursor::{Hide, Show},
    ExecutableCommand,

};
#[allow(unused)]
use sysinfo::{System, SystemExt, CpuExt,RefreshKind, CpuRefreshKind, NetworkExt, NetworksExt, NetworksIter, Process, ProcessExt, DiskExt, ComponentExt};
#[allow(unused)]
use std::sync::mpsc;

mod lib;
use lib::{home_screen, cpu_screen, memory_screen, network_screen, process_screen, disk_screen, temp_screen, battery_screen};
enum Screen {
    Home,
    CPU,
    Memory,
    Network,
    Process,
    Disk,
    Temp,
    Battery,
    Exit,
}

#[derive(Default)]
pub struct App {
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
}

#[allow(unused)]
fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut scroll_position = 0;
    let mut app = App::default();
    terminal::enable_raw_mode()?;
    io::stdout().execute(Hide)?; // Hide the cursor

    
    let mut system = System::new_all();

    let mut current_screen = Screen::Home;

    loop {
        match current_screen {
            Screen::Home => {
                // Display the home screen
                terminal.draw(|f| {
                    let size = f.size();
                    home_screen(f, size);
                })?;
            }
            Screen::CPU => {
                // Display the CPU usage
                system.refresh_all();
                let mut cpu_usage = vec![];
                let mut chart_value = vec![];
                
                for (i, cpu) in system.cpus().iter().enumerate() {
                    let cpu_stat = format!("CPU {}: {:.2}%", i, cpu.cpu_usage());
                    let cpu_name = format!("{}", i); // cpu_name is an owned String now
                    cpu_usage.push(Line::from(vec![
                        Span::raw(cpu_stat),
                    ]));
                    chart_value.push((cpu_name, cpu.cpu_usage() as u64));
                }
                
                let chart_data: Vec<(&str, u64)> = chart_value.iter().map(|(name, value)| (name.as_str(), *value)).collect();
                

                terminal.draw(|f| {
                    let size = f.size();
                    cpu_screen(f, size, cpu_usage, chart_data);
                })?;
            }
            
            Screen::Memory => {
                // Display the Memory screen
                system.refresh_all();
                let used_memory = system.used_memory() as f64;
                let total_memory = system.total_memory() as f64;
                let mem_usage = format!("Memory: {:.2} / {:.2} GB", used_memory/ 1_073_741_824.0 , total_memory / 1_073_741_824.0); // 1 GB = 1_073_741_824.0 bytes
                let percent_mem = ((used_memory/ total_memory) * 100.0 )as u16;

                terminal.draw(|f| {
                    let size = f.size();
                    memory_screen(f, size, mem_usage, percent_mem);
                })?;
            }

            Screen::Network => {
                // Display the Memory screen
                system.refresh_all();
                let networks = system.networks();
                let mut network_act = vec![];
                for (interface_name, net_data) in networks.iter() {
                    let net = format!(
                        "[{}] in: {}, out: {}",
                        interface_name,
                        net_data.received(),
                        net_data.transmitted(),
                    );
                    network_act.push(Line::from(vec![Span::raw(net)]))
                }
                
                terminal.draw(|f| {
                    let size = f.size();
                    network_screen(f, size, network_act);
                })?;
            }
            
            Screen::Process => {
                // Display the Process screen
                system.refresh_all();
                let pro = system.processes();
                let mut all_processes = vec![];
                
                for (pid, process) in pro.iter() {
                    let formatted_pid = format!("Process ID: {:7}", pid);
                    let result = format!("[{:17}] {:40} {:.2} MB", formatted_pid, process.name(), process.memory() as f64 / 1_048_576.0);
                    all_processes.push(Line::from(vec![Span::raw(result)]))
            
                }
                app.vertical_scroll_state = app.vertical_scroll_state.content_length(all_processes.len() as u16);
                let vertical_scroll = app.vertical_scroll as u16;

                // Render the visible lines
                terminal.draw(|f| {
                    let size = f.size();
                    let num_visible_lines = size.height as usize;
                
                    // Calculate the range of lines to display based on scroll position
                    let start_line = scroll_position;
                    let end_line = scroll_position + num_visible_lines;
                    
                    // Extract the subset of lines to display
                    let visible_lines = all_processes[start_line..end_line].to_vec();
                    let scroll = &mut ScrollbarState::default();
                    process_screen(f, size, visible_lines, &mut app.vertical_scroll_state, vertical_scroll);
        })?;
            }

            Screen::Disk => {
                // Display the Disk screen
                system.refresh_all();
                let disk_info = system.disks();
                let mut all_disk = vec![];

                for disk in disk_info {
                    let disk_name = format!("Disk({:?})\n", disk.name());
                    let file_s = format!("  [FS: {:?}]\n", disk.file_system().iter().map(|c| *c as char).collect::<Vec<_>>());
                    let types = format!("  [Type: {:?}]\n", disk.kind());
                    let removable = format!("  [removeable: {}\n", if disk.is_removable() { "yes" } else { "no" });
            
                    let mounted_on_gb = disk.available_space() / 1_073_741_824;
                    let total_gb = disk.total_space() / 1_073_741_824;
                    
                    let gb = (format!("  mounted on {:?}: {:.2}/{:.2} GB]\n", disk.mount_point(), mounted_on_gb, total_gb));
                    all_disk.push(Line::from(vec![Span::raw(disk_name)]));
                    all_disk.push(Line::from(vec![Span::raw(file_s)]));
                    all_disk.push(Line::from(vec![Span::raw(types)]));
                    all_disk.push(Line::from(vec![Span::raw(removable)]));
                    all_disk.push(Line::from(vec![Span::raw(gb)]));
                }

                terminal.draw(|f| {
                    let size = f.size();
                    disk_screen(f, size, all_disk);
                })?;
            }

            Screen::Temp => {
                // Display the Temperture screen
                system.refresh_all();
                let temp_comp = system.components();
                let mut all_temp = vec![];
                let mut chart = vec![];
                for component in temp_comp {
                    let result = format!("{:?}", component);
                    all_temp.push(Line::from(vec![Span::raw(result)]));
                    let temp_name = format!("{}", component.label());
                    chart.push((temp_name, component.temperature() as u64));
                }

                let chart_data: Vec<(&str, u64)> = chart.iter().map(|(name, value)| (name.as_str(), *value)).collect();

                app.vertical_scroll_state = app.vertical_scroll_state.content_length(all_temp.len() as u16);
                let vertical_scroll = app.vertical_scroll as u16;

                terminal.draw(|f| {
                    let size = f.size();
                    temp_screen(f, size, all_temp, &mut app.vertical_scroll_state, vertical_scroll, chart_data);
                })?;
            }

            Screen::Battery => {
                // Display the Battery screen
                
                let manager = battery::Manager::new().unwrap();
                let mut battery_data = vec![];
                for (idx, maybe_battery) in manager.batteries().unwrap().enumerate() {
                    let battery = maybe_battery.unwrap();
                    let model_info = match battery.model() {
                        Some(model) => model,
                        None => "Unknown Model",
                    };
                    battery_data.push(Line::from(vec![Span::raw(format!("Battery #{}", idx))]));
                    battery_data.push(Line::from(vec![Span::raw(format!("Vendor: {:?}", battery.vendor()))]));
                    battery_data.push(Line::from(vec![Span::raw(format!("Model: {:?}", model_info))]));
                    battery_data.push(Line::from(vec![Span::raw(format!("State: {:?}", battery.state()))]));
                    battery_data.push(Line::from(vec![Span::raw(format!("Time to full charge: {:?}", battery.time_to_full()))]));
                    battery_data.push(Line::from(vec![Span::raw(format!("State of charge: {:.0} %", battery.state_of_charge().get::<percent>()))]));
                }
                let mut battery_percentages: Vec<u16> = manager
                    .batteries()
                    .unwrap()
                    .into_iter()
                    .map(|bat| (bat.unwrap().state_of_charge().get::<percent>()) as u16)
                    .collect();

                let overall_battery_percentage = battery_percentages.iter().sum::<u16>() / battery_percentages.len() as u16;
                
                
                terminal.draw(|f| {
                    let size = f.size();
                    battery_screen(f, size, battery_data, overall_battery_percentage);
                })?;
            }

            Screen::Exit => break, // Exit the application
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let event::Event::Key(KeyEvent { code, modifiers, state, kind }) = event::read()? {
                match current_screen {
                    Screen::Home => {
                        // Handle navigation from the home screen
                        match code {
                            KeyCode::Char('1') => current_screen = Screen::CPU,
                            KeyCode::Char('2') => current_screen = Screen::Memory,
                            KeyCode::Char('3') => current_screen = Screen::Network,
                            KeyCode::Char('4') => current_screen = Screen::Process,
                            KeyCode::Char('5') => current_screen = Screen::Disk,
                            KeyCode::Char('6') => current_screen = Screen::Temp,
                            KeyCode::Char('7') => current_screen = Screen::Battery,
                            KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => current_screen = Screen::Exit,
                            _ => (),
                        }
                    }
                    Screen::Process => {
                        // Handle scrolling within the Process screen
                        match code {
                            KeyCode::Down => { //down
                                app.vertical_scroll = app.vertical_scroll.saturating_add(1);
                                app.vertical_scroll_state = app
                                    .vertical_scroll_state
                                    .position(app.vertical_scroll as u16);
                            }
                            KeyCode::Up => { //up
                                app.vertical_scroll = app.vertical_scroll.saturating_sub(1);
                                app.vertical_scroll_state = app
                                    .vertical_scroll_state
                                    .position(app.vertical_scroll as u16);
                            }
                            KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => current_screen = Screen::Home,
                            _ => ()
                        }
                    }
                    Screen::Temp => {
                        // Handle scrolling within the Process screen
                        match code {
                            KeyCode::Down => { //down
                                app.vertical_scroll = app.vertical_scroll.saturating_add(1);
                                app.vertical_scroll_state = app
                                    .vertical_scroll_state
                                    .position(app.vertical_scroll as u16);
                            }
                            KeyCode::Up => { //up
                                app.vertical_scroll = app.vertical_scroll.saturating_sub(1);
                                app.vertical_scroll_state = app
                                    .vertical_scroll_state
                                    .position(app.vertical_scroll as u16);
                            }
                            KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => current_screen = Screen::Home,
                            _ => ()
                        }
                    }
                    _ => {
                        // Handle returning to the home screen from other screens
                        if code == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL {
                            current_screen = Screen::Home;
                        }
                    }
                }
            }
        }
        
    }

    io::stdout().execute(Show)?;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
