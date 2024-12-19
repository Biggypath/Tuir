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

pub fn home_screen<B: Backend>(f: &mut Frame<B>, size: ratatui::layout::Rect) {
    let block = Block::default()
        .title("Home")
        .borders(Borders::ALL);

    let menu = List::new(vec![
        ListItem::new("1. CPU Metrics"),
        ListItem::new("2. Memory Metrics"),
        ListItem::new("3. Network Metrics"),
        ListItem::new("4. Process Metrics"),
        ListItem::new("5. Disk Metrics"),
        ListItem::new("6. Temperture Metrics"),
        ListItem::new("7. Battery Metrics"),
        ListItem::new("Ctrl+C. Exit"),
    ])
    .style(Style::default().fg(Color::Cyan))
    .block(block);

    f.render_widget(menu, size);
}

pub fn cpu_screen<B: Backend>(
    f: &mut Frame<B>,
    size: ratatui::layout::Rect,
    cpu_usage: Vec<Line<'_>>,
    chart: Vec<(&str, u64)>, 
) {
    let block = Block::default()
        .title("CPU Usage")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(cpu_usage)
        .alignment(Alignment::Left)
        .cyan()
        .block(block)
        .wrap(Wrap { trim: true });

    // Create a BarChart widget with the provided data
    let barchart = BarChart::default()
        .block(Block::default().title("BarChart").borders(Borders::ALL))
        .cyan()
        .bar_width(3)
        .bar_gap(1)
        .direction(Direction::Vertical)
        .group_gap(3)
        .bar_style(Style::new().light_green())
        .value_style(Style::new().blue().bold())
        .label_style(Style::new().white())
        .data(&chart);  // Pass the chart data here;

    // Split the available space to display both the paragraph and the barchart
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(size);

    // Render both the paragraph and the barchart in their respective areas
    f.render_widget(paragraph, chunks[0]);
    f.render_widget(barchart, chunks[1]);
}

    

pub fn memory_screen<B: Backend>(f: &mut Frame<B>, size: ratatui::layout::Rect, memory_con: String, memory_ratio: u16,) {
    let block = Block::default()
        .title("Memory Usage")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(memory_con)
        .alignment(Alignment::Left)
        .cyan()
        .block(block.clone())
        .wrap(Wrap { trim: true });

    let gauge = Gauge::default()
        .block(block.title("percent").borders(Borders::ALL))
        .cyan()
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::ITALIC))
        .percent(memory_ratio);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(size);
    
    f.render_widget(paragraph, size);
    f.render_widget(gauge, chunks[1]);
}

pub fn network_screen<B: Backend>(f: &mut Frame<B>, size: ratatui::layout::Rect, net_act:  Vec<Line<'_>>) {
    let block = Block::default()
        .title("Network activity")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(net_act)
        .alignment(Alignment::Left)
        .cyan()
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, size);
}

pub fn process_screen<B: Backend>(
    f: &mut Frame<B>,
    size: ratatui::layout::Rect,
    process: Vec<Line<'_>>,
    scrollbar_state: &mut ScrollbarState,
    vertical_scroll: u16,
) {
    let block = Block::default()
        .title("Process")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(process)
        .alignment(Alignment::Left)
        .cyan()
        .scroll((vertical_scroll, 0))
        .block(block)
        .wrap(Wrap { trim: true });
        
    f.render_widget(paragraph, size,);
    f.render_stateful_widget(Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(None)
            .track_symbol(None)
            .end_symbol(None),
            size.inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            scrollbar_state
            );

}


pub fn disk_screen<B: Backend>(f: &mut Frame<B>, size: ratatui::layout::Rect, disk: Vec<Line<'_>>) {
    let block = Block::default()
        .title("Disk Information")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(disk)
        .alignment(Alignment::Left)
        .cyan()
        .block(block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, size);
}

pub fn temp_screen<B: Backend>(
    f: &mut Frame<B>, 
    size: ratatui::layout::Rect, 
    temp: Vec<Line<'_>>,
    scrollbar_state: &mut ScrollbarState,
    vertical_scroll: u16,
    chart: Vec<(&str, u64)>,
) {
    let block = Block::default()
        .title("temperature")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(temp)
        .alignment(Alignment::Left)
        .cyan()
        .scroll((vertical_scroll, 0))
        .block(block)
        .wrap(Wrap { trim: true });

    let barchart = BarChart::default()
    .block(Block::default().title("BarChart").borders(Borders::ALL))
    .cyan()
    .bar_width(3)
    .bar_gap(1)
    .group_gap(3)
    .bar_style(Style::new().light_green())
    .value_style(Style::new().black().bold())
    .label_style(Style::new().white())
    .data(&chart); // Pass the chart data here;

    // Split the available space to display both the paragraph and the barchart
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(size);

    f.render_widget(paragraph, chunks[0]);
    f.render_stateful_widget(Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(None)
            .track_symbol(None)
            .end_symbol(None),
            chunks[0].inner(&Margin {
                vertical: 1,
                horizontal: 0,
            }),
            scrollbar_state
            );
    
    f.render_widget(barchart, chunks[1]);
}

pub fn battery_screen<B: Backend>(
    f: &mut Frame<B>,
    size: ratatui::prelude::Rect,
    battery: Vec<Line<'_>>,
    progress: u16,
) {
    let block = Block::default().title("Battery Info").borders(Borders::ALL);

    let paragraph = Paragraph::new(battery)
        .alignment(Alignment::Left)
        .cyan()
        .block(block.clone())
        .wrap(Wrap { trim: true });

    let gauge = Gauge::default()
        .block(block.title("Progress").borders(Borders::ALL))
        .cyan()
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::ITALIC))
        .percent(progress);

    // Split the available space to display both the paragraph and the gauge
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(size);

    // Render both the paragraph and the gauge in their respective areas
    f.render_widget(paragraph, chunks[0]);
    f.render_widget(gauge, chunks[1]);
}