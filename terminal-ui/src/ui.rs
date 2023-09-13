use ratatui::backend::Backend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::Color;
use ratatui::text::Span;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Table, Paragraph, Row
};
use ratatui::Frame;

use orderbook_merger::orderbook_summary::Summary;

pub fn draw<B>(rect: &mut Frame<B>, summary: &Summary, decimals: u32)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(12),
            ]
            .as_ref(),
        )
        .split(rect.size());

    // Title
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    // Summary
    let summary_widget = draw_summary(&summary, decimals);
    rect.render_widget(summary_widget, chunks[1]);
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("Orderbook Summary")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL),
        )
}

fn draw_summary(summary: &Summary, decimals: u32) -> Table {

    let mut rows = vec![];

    rows.push(Row::new(vec![
        Cell::from("Ask/Bid"),
        Cell::from(format!("{:>10}", "Quantity")),
        Cell::from("Exchange"),
    ]));

    rows.push(Row::new(vec![""; 3]));

    for level in summary.asks.iter().rev() {
        let row = Row::new(vec![
            Cell::from(Span::styled(
                format!("{:>8.1$}", level.price, decimals as usize),
                Style::default().fg(Color::LightRed),
            )),
            Cell::from(format!("{:>10.5}", level.quantity),),
            Cell::from(&*level.exchange),
        ]);
        rows.push(row);
    }

    rows.push(Row::new(vec![""; 3]));

    rows.push(Row::new(vec![
        Cell::from(Span::styled(
            format!("{:>8.1$}", summary.spread, decimals as usize),
            Style::default().fg(Color::LightYellow),
        )),
        Cell::from(""),
        Cell::from(""),
    ]));

    rows.push(Row::new(vec![""; 3]));

    for level in summary.bids.iter() {
        let row = Row::new(vec![
            Cell::from(Span::styled(
                format!("{:>8.1$}", level.price, decimals as usize),
                Style::default().fg(Color::LightGreen),
            )),
            Cell::from(format!("{:>10.5}", level.quantity)),
            Cell::from(&*level.exchange),
        ]);
        rows.push(row);
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .column_spacing(1)
}