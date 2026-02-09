use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        BarChart, BarGroup, Block, Borders, BorderType, Paragraph,
    },
};

pub fn render_barchart_view(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // CPU chart
            Constraint::Min(10),    // Memory chart
            Constraint::Length(2),  // Help
        ])
        .split(frame.area());

    render_barchart_title(frame, app, chunks[0]);
    render_cpu_barchart(frame, app, chunks[1]);
    render_memory_barchart(frame, app, chunks[2]);
    render_barchart_help(frame, app, chunks[3]);
}

fn render_barchart_title(frame: &mut Frame, app: &App, area: Rect) {
    let title_line = Line::from(vec![
        Span::styled("📊 ", Style::default().fg(app.theme.primary)),
        Span::styled("Resource Distribution Across Sessions", 
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(app.theme.primary));

    let para = Paragraph::new(title_line)
        .block(block)
        .style(Style::default().bg(app.theme.bg_primary));
    
    frame.render_widget(para, area);
}

fn render_cpu_barchart(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.success))
        .title(" CPU Usage by Session (%) ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Collect CPU data from sessions
    let mut data: Vec<(&str, u64)> = app
        .sessions
        .iter()
        .filter_map(|s| {
            s.stats.as_ref().map(|stats| {
                let name = if s.name.len() > 8 {
                    &s.name[..8]
                } else {
                    &s.name
                };
                (name, stats.cpu_percent as u64)
            })
        })
        .collect();

    // Sort by CPU descending and take top 10
    data.sort_by(|a, b| b.1.cmp(&a.1));
    data.truncate(10);

    if data.is_empty() {
        let empty = Paragraph::new("No session data available")
            .style(Style::default().fg(app.theme.text_dim));
        frame.render_widget(empty, inner);
        return;
    }

    // Create bar data with colors based on CPU usage
    let bars: Vec<_> = data
        .iter()
        .map(|(name, cpu)| {
            let color = if *cpu > 200 {
                app.theme.error
            } else if *cpu > 100 {
                app.theme.warning
            } else {
                app.theme.success
            };
            (name, *cpu, color)
        })
        .collect();

    // Build BarGroup with colored bars
    let bar_data: Vec<_> = bars
        .iter()
        .map(|(label, value, color)| {
            ratatui::widgets::Bar::default()
                .value(*value)
                .label(Line::from(**label))
                .style(Style::default().fg(*color))
        })
        .collect();

    let group = BarGroup::default().bars(&bar_data);

    let barchart = BarChart::default()
        .data(group)
        .bar_width(5)
        .bar_gap(1)
        .bar_style(Style::default().fg(app.theme.success))
        .value_style(Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD));

    frame.render_widget(barchart, inner);
}

fn render_memory_barchart(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.info))
        .title(" Memory Usage by Session (MB) ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Collect memory data from sessions
    let mut data: Vec<(&str, u64)> = app
        .sessions
        .iter()
        .filter_map(|s| {
            s.stats.as_ref().map(|stats| {
                let name = if s.name.len() > 8 {
                    &s.name[..8]
                } else {
                    &s.name
                };
                (name, stats.mem_mb)
            })
        })
        .collect();

    // Sort by memory descending and take top 10
    data.sort_by(|a, b| b.1.cmp(&a.1));
    data.truncate(10);

    if data.is_empty() {
        let empty = Paragraph::new("No session data available")
            .style(Style::default().fg(app.theme.text_dim));
        frame.render_widget(empty, inner);
        return;
    }

    // Create bar data with colors based on memory usage
    let bars: Vec<_> = data
        .iter()
        .map(|(name, mem)| {
            let color = if *mem > 4096 {
                app.theme.error
            } else if *mem > 2048 {
                app.theme.warning
            } else {
                app.theme.info
            };
            (name, *mem, color)
        })
        .collect();

    // Build BarGroup with colored bars
    let bar_data: Vec<_> = bars
        .iter()
        .map(|(label, value, color)| {
            ratatui::widgets::Bar::default()
                .value(*value)
                .label(Line::from(**label))
                .style(Style::default().fg(*color))
        })
        .collect();

    let group = BarGroup::default().bars(&bar_data);

    let barchart = BarChart::default()
        .data(group)
        .bar_width(5)
        .bar_gap(1)
        .bar_style(Style::default().fg(app.theme.info))
        .value_style(Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD));

    frame.render_widget(barchart, inner);
}

fn render_barchart_help(frame: &mut Frame, app: &App, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(app.theme.text_dim)),
        Span::styled("B", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" or ", Style::default().fg(app.theme.text_dim)),
        Span::styled("ESC", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" to return to normal view", Style::default().fg(app.theme.text_dim)),
    ]));
    
    frame.render_widget(help, area);
}
