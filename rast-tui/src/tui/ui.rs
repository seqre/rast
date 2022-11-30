#[allow(unused_imports)]
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{
        canvas::{Canvas, Line, Map, MapResolution, Rectangle},
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};

use crate::tui::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    draw_ips(f, app, chunks[0]);
    draw_terminal(f, app, chunks[1]);
}

fn draw_ips<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let victims: Vec<ListItem> = app
        .victims
        .items
        .iter()
        .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
        .collect();
    let victims = List::new(victims)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Connected victims"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_stateful_widget(victims, area, &mut app.victims.state);
}

fn draw_terminal<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let text = vec![Spans::from("lala lal laldl dasl dsa lkjfdsa")];
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "Terminal",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}
