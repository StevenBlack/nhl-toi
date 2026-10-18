//! Terminal UI showing live NHL time-on-ice for both teams of a game.
//!
//! A background thread polls the NHL HTML reports (see [`report`]) and the UI
//! shows one sortable table per team. Keys: `Tab`/`←`/`→` switch table,
//! `1`-`7` sort by column (again to reverse), `q` quits.

mod report;

use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table};
use ratatui::{DefaultTerminal, Frame};

use report::{Player, Side, TeamReport};

/// Season used when none is given on the command line.
const DEFAULT_SEASON: &str = "20262027";
/// Game id used when none is given on the command line.
const DEFAULT_GAME: &str = "020106";
/// Delay between fetches.
const REFRESH: Duration = Duration::from_secs(10);

/// One fetch result for both teams, sent from the fetcher thread to the UI.
struct Update {
    visitor: Result<TeamReport, String>,
    home: Result<TeamReport, String>,
}

/// A table column; also the sort key.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Column {
    Number,
    Name,
    Toi,
    Ev,
    Pp,
    Sh,
    Spt,
}

impl Column {
    /// Columns in display order; index + 1 is the sort key.
    const ALL: [Column; 7] = [
        Column::Number,
        Column::Name,
        Column::Toi,
        Column::Ev,
        Column::Pp,
        Column::Sh,
        Column::Spt,
    ];

    /// Header text.
    fn label(self) -> &'static str {
        match self {
            Column::Number => "#",
            Column::Name => "Player",
            Column::Toi => "TOI",
            Column::Ev => "EV",
            Column::Pp => "PP",
            Column::Sh => "SH",
            Column::Spt => "STT",
        }
    }

    /// Text columns read best ascending on first press, time columns descending.
    fn default_desc(self) -> bool {
        !matches!(self, Column::Number | Column::Name)
    }

    /// Ascending order of two players by this column (PP/SH tie-break on each other).
    fn compare(self, a: &Player, b: &Player) -> std::cmp::Ordering {
        match self {
            Column::Number => a
                .number
                .parse::<u32>()
                .unwrap_or(u32::MAX)
                .cmp(&b.number.parse::<u32>().unwrap_or(u32::MAX)),
            Column::Name => a.name.cmp(&b.name),
            Column::Toi => seconds(&a.toi).cmp(&seconds(&b.toi)),
            Column::Ev => seconds(&a.ev).cmp(&seconds(&b.ev)),
            // PP and SH subsort by each other.
            Column::Pp => seconds(&a.pp)
                .cmp(&seconds(&b.pp))
                .then_with(|| seconds(&a.sh).cmp(&seconds(&b.sh))),
            Column::Sh => seconds(&a.sh)
                .cmp(&seconds(&b.sh))
                .then_with(|| seconds(&a.pp).cmp(&seconds(&b.pp))),
            Column::Spt => seconds(&a.spt).cmp(&seconds(&b.spt)),
        }
    }
}

/// "MM:SS" to seconds; unparsable parts count as 0.
fn seconds(t: &str) -> u32 {
    t.split(':')
        .fold(0, |acc, part| acc * 60 + part.parse::<u32>().unwrap_or(0))
}

/// Sort column and direction for one table.
#[derive(Clone, Copy, Default)]
struct SortState {
    /// `None` keeps report order.
    column: Option<Column>,
    desc: bool,
}

impl SortState {
    /// Selects `col`, or reverses direction if it's already selected.
    fn press(&mut self, col: Column) {
        if self.column == Some(col) {
            self.desc = !self.desc;
        } else {
            *self = SortState { column: Some(col), desc: col.default_desc() };
        }
    }

    /// Players ordered per this state.
    fn sorted<'a>(&self, players: &'a [Player]) -> Vec<&'a Player> {
        let mut v: Vec<&Player> = players.iter().collect();
        if let Some(col) = self.column {
            // Stable sort keeps report order for ties.
            v.sort_by(|a, b| {
                let o = col.compare(a, b);
                if self.desc { o.reverse() } else { o }
            });
        }
        v
    }
}

/// UI state.
#[derive(Default)]
struct App {
    visitor: TeamReport,
    home: TeamReport,
    /// Latest fetch errors, joined.
    error: Option<String>,
    /// Time of the last fully successful fetch.
    updated: Option<Instant>,
    /// Index 0 = visitor (left), 1 = home (right). Each table sorts independently.
    sort: [SortState; 2],
    /// Table (0 or 1) that receives sort keys.
    focus: usize,
}

impl App {
    /// Merges a fetch result, keeping old data for any side that failed.
    fn apply(&mut self, u: Update) {
        let mut errors = Vec::new();
        match u.visitor {
            Ok(r) => self.visitor = r,
            Err(e) => errors.push(e),
        }
        match u.home {
            Ok(r) => self.home = r,
            Err(e) => errors.push(e),
        }
        if errors.is_empty() {
            self.updated = Some(Instant::now());
        }
        self.error = (!errors.is_empty()).then(|| errors.join(" | "));
    }
}

/// Polls both reports every `REFRESH` on a thread; stops once the game is final or the UI exits.
fn spawn_fetcher(season: String, game: String) -> Receiver<Update> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            let update = Update {
                visitor: report::fetch(&season, &game, Side::Visitor),
                home: report::fetch(&season, &game, Side::Home),
            };
            let game_over = matches!((&update.visitor, &update.home), (Ok(v), Ok(h)) if v.is_final() && h.is_final());
            if tx.send(update).is_err() || game_over {
                break; // UI has exited, or the game is final so nothing more will change
            }
            thread::sleep(REFRESH);
        }
    });
    rx
}

/// Usage: `nhl-toi [game] [season]`.
fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let game = args.next().unwrap_or_else(|| DEFAULT_GAME.into());
    let season = args.next().unwrap_or_else(|| DEFAULT_SEASON.into());

    let rx = spawn_fetcher(season, game.clone());
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &rx, &game);
    ratatui::restore();
    result
}

/// Event loop: applies updates, draws, handles keys.
fn run(terminal: &mut DefaultTerminal, rx: &Receiver<Update>, game: &str) -> std::io::Result<()> {
    let mut app = App::default();
    loop {
        while let Ok(u) = rx.try_recv() {
            app.apply(u);
        }
        terminal.draw(|f| draw(f, &app, game))?;

        if event::poll(Duration::from_millis(250))?
            && let Event::Key(k) = event::read()?
            && k.kind == KeyEventKind::Press
        {
            match k.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                KeyCode::Tab | KeyCode::BackTab => app.focus = 1 - app.focus,
                KeyCode::Left => app.focus = 0,
                KeyCode::Right => app.focus = 1,
                KeyCode::Char(c @ '1'..='7') => {
                    let col = Column::ALL[c as usize - '1' as usize];
                    app.sort[app.focus].press(col);
                }
                _ => {}
            }
        }
    }
}

/// Renders header, both tables and the key-help footer.
fn draw(f: &mut Frame, app: &App, game: &str) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(f.area());

    let status = if app.visitor.status.is_empty() { "loading…" } else { &app.visitor.status };
    let age = if app.error.is_none() && app.visitor.is_final() && app.home.is_final() {
        "Official".to_string()
    } else {
        app.updated
            .map(|t| format!("updated {}s ago", t.elapsed().as_secs()))
            .unwrap_or_else(|| "waiting for data".into())
    };
    let mut lines = vec![Line::from(format!(
        "{} @ {}   |   Game {game}   |   {status}   |   {age}",
        app.visitor.team, app.home.team
    ))];
    if let Some(e) = &app.error {
        lines.push(Line::styled(format!("Error: {e}"), Style::new().fg(Color::Red)));
    }
    f.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" NHL Time On Ice ")),
        header,
    );

    let [left, right] = Layout::horizontal([Constraint::Percentage(50); 2]).areas(body);
    f.render_widget(team_table(&app.visitor, app.sort[0], app.focus == 0), left);
    f.render_widget(team_table(&app.home, app.sort[1], app.focus == 1), right);

    f.render_widget(
        Paragraph::new("Tab/←/→: switch table   1-7: sort by # / Player / TOI / EV / PP / SH / STTot (again to reverse)   q: quit")
            .style(Style::new().fg(Color::DarkGray)),
        footer,
    );
}

/// One team's table; the focused one gets a cyan border.
fn team_table(r: &TeamReport, sort: SortState, focused: bool) -> Table<'_> {
    let head = Row::new(Column::ALL.map(|c| {
        if sort.column == Some(c) {
            format!("{}{}", c.label(), if sort.desc { "▼" } else { "▲" })
        } else {
            c.label().to_string()
        }
    }))
    .style(Style::new().add_modifier(Modifier::BOLD).fg(Color::Yellow));

    let rows = sort.sorted(&r.players).into_iter().map(|p| {
        Row::new([
            Cell::from(p.number.as_str()),
            Cell::from(p.name.as_str()),
            Cell::from(p.toi.as_str()),
            Cell::from(p.ev.as_str()),
            Cell::from(p.pp.as_str()),
            Cell::from(p.sh.as_str()),
            Cell::from(p.spt.as_str()),
        ])
    });

    let border = if focused { Style::new().fg(Color::Cyan) } else { Style::new() };
    Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
        ],
    )
    .header(head)
    .block(
        Block::bordered()
            .border_style(border)
            .title(format!(" {} ", r.team)),
    )
}
