use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::Span;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Table, Paragraph, Row
};
use ratatui::Frame;

use super::state::AppState;
use crate::app::App;

pub fn draw<B>(rect: &mut Frame<B>, app: &App, symbol: &str, decimals: u32)
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
                Constraint::Min(10),
                Constraint::Length(12),
            ]
            .as_ref(),
        )
        .split(size);

    // Title
    let title = draw_title(symbol.to_string());
    rect.render_widget(title, chunks[0]);

    // Body
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [Constraint::Percentage(100)].as_ref(),
        )
        .split(chunks[1]);

    let summary = draw_summary(app.state(), decimals);
    rect.render_widget(summary, body_chunks[0]);

}

fn draw_title<'a>(symbol: String) -> Paragraph<'a> {
    Paragraph::new(format!("Orderbook Summary: {}", symbol))
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
    if rect.width < 52 {
        panic!("Require width >= 52, (got {})", rect.width);
    }
    if rect.height < 28 {
        panic!("Require height >= 28, (got {})", rect.height);
    }
}

fn draw_summary(state: &AppState, decimals: u32) -> Table {
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    rows.push(Row::new(vec![
        Cell::from(Span::styled("Ask/Bid".to_string(), help_style)),
        Cell::from(Span::styled(format!("{:>10}", "Quantity"), help_style)),
        Cell::from(Span::styled("Exchange".to_string(), help_style)),
    ]));
    if let Some(summary) = state.get_summary() {
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
    };

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