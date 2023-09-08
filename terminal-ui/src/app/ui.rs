use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::Span;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Table, Paragraph, Row
};
use ratatui::Frame;

use orderbook_merger::orderbook_summary::Summary;
use crate::app::App;

pub fn draw<B>(rect: &mut Frame<B>, app: &App, decimals: u32)
where
    B: Backend,
{
    let size = rect.size();
    check_size(&size);

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(12),
            ]
            .as_ref(),
        )
        .split(size);

    // Title
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    // Body
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [Constraint::Percentage(100)].as_ref(),
        )
        .split(chunks[1]);

    let summary = draw_summary(&app.summary, decimals);
    rect.render_widget(summary, body[0]);

}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new(format!("Orderbook Summary"))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}

fn check_size(rect: &Rect) {
    if rect.width < 20 {
        panic!("Require width >= 20, (got {})", rect.width);
    }
    if rect.height < 15 {
        panic!("Require height >= 15, (got {})", rect.height);
    }
}

fn draw_summary(summary: &Summary, decimals: u32) -> Table {
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    rows.push(Row::new(vec![
        Cell::from(Span::styled("Ask/Bid".to_string(), help_style)),
        Cell::from(Span::styled(format!("{:>10}", "Quantity"), help_style)),
        Cell::from(Span::styled("Exchange".to_string(), help_style)),
    ]));

    for level in summary.asks.iter().rev() {
        let row = Row::new(vec![
            Cell::from(Span::styled(
                format!("{:>8.1$}", level.price, decimals as usize),
                Style::default().fg(Color::LightRed),
            )),
            Cell::from(Span::styled(
                format!("{:>10.5}", level.quantity),
                help_style,
            )),
            Cell::from(Span::styled(&level.exchange, help_style)),
        ]);
        rows.push(row);
    }
    rows.push(Row::new(vec![Span::styled("".to_string(), help_style); 3]));
    rows.push(Row::new(vec![
        Cell::from(Span::styled(
            format!("{:>8.1$}", summary.spread, decimals as usize),
            Style::default().fg(Color::LightYellow),
        )),
        Cell::from(Span::styled("".to_string(), help_style)),
        Cell::from(Span::styled("".to_string(), help_style)),
    ]));
    rows.push(Row::new(vec![Span::styled("".to_string(), help_style); 3]));
    for level in summary.bids.iter() {
        let row = Row::new(vec![
            Cell::from(Span::styled(
                format!("{:>8.1$}", level.price, decimals as usize),
                Style::default().fg(Color::LightGreen),
            )),
            Cell::from(Span::styled(
                format!("{:>10.5}", level.quantity),
                help_style,
            )),
            Cell::from(Span::styled(&level.exchange, help_style)),
        ]);
        rows.push(row);
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Orderbook Summary"),
        )
        .widths(&[
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .column_spacing(1)
}