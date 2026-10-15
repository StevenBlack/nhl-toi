use scraper::{Html, Selector};

const BASE_URL: &str = "https://www.nhl.com/scores/htmlreports";

/// Chrome major version to impersonate. Chrome freezes the minor parts of its UA string at 0.0.0.
const CHROME_MAJOR: u32 = 155;
const USER_AGENT: &str = const_format::formatcp!(
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{CHROME_MAJOR}.0.0.0 Safari/537.36"
);
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";

#[derive(Debug, Clone, Default)]
pub struct Player {
    pub number: String,
    pub name: String,
    pub toi: String,
    pub ev: String,
    pub pp: String,
    pub sh: String,
}

#[derive(Debug, Clone, Default)]
pub struct TeamReport {
    pub team: String,
    pub status: String,
    pub players: Vec<Player>,
}

impl TeamReport {
    /// The game clock line reads "Final" (possibly with an OT/SO suffix) once the game is over.
    pub fn is_final(&self) -> bool {
        self.status.to_ascii_lowercase().starts_with("final")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Visitor,
    Home,
}

pub fn url(season: &str, game: &str, side: Side) -> String {
    let prefix = match side {
        Side::Visitor => "TV",
        Side::Home => "TH",
    };
    format!("{BASE_URL}/{season}/{prefix}{game}.HTM")
}

pub fn fetch(season: &str, game: &str, side: Side) -> Result<TeamReport, String> {
    let url = url(season, game, side);
    let mut resp = ureq::get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", ACCEPT)
        .header("Accept-Language", ACCEPT_LANGUAGE)
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    let html = resp
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("{url}: {e}"))?;
    Ok(parse(&html))
}

fn text(el: scraper::ElementRef) -> String {
    el.text()
        .collect::<String>()
        .replace('\u{a0}', " ")
        .trim()
        .to_string()
}

/// Pulls one row per player out of an NHL "TOI Shift Report" page.
///
/// Each player is a `td.playerHeading` ("13 CAUFIELD, COLE") followed by a shift
/// table and a summary table whose final row is bold and starts with "TOT":
/// TOT, SHF, AVG, TOI, EV TOT, PP TOT, SH TOT.
pub fn parse(html: &str) -> TeamReport {
    let doc = Html::parse_document(html);
    let cells = Selector::parse("td.playerHeading, td.bold").unwrap();
    let team_sel = Selector::parse("td.teamHeading").unwrap();
    let info_sel = Selector::parse("table#GameInfo td").unwrap();

    let team = doc.select(&team_sel).next().map(text).unwrap_or_default();
    // The last non-empty line of the game info block is the clock, e.g. "Period 1 (03:16 Remaining)".
    let status = doc
        .select(&info_sel)
        .map(text)
        .rfind(|t| !t.is_empty())
        .unwrap_or_default();

    let mut players: Vec<Player> = Vec::new();
    let mut totals: Vec<String> = Vec::new();

    fn flush(players: &mut [Player], totals: &mut Vec<String>) {
        if let (Some(p), true) = (players.last_mut(), totals.len() >= 7 && totals[0] == "TOT") {
            p.toi = totals[3].clone();
            p.ev = totals[4].clone();
            p.pp = totals[5].clone();
            p.sh = totals[6].clone();
        }
        totals.clear();
    }

    for cell in doc.select(&cells) {
        let t = text(cell);
        if cell.value().classes().any(|c| c == "playerHeading") {
            flush(&mut players, &mut totals);
            let (number, name) = t.split_once(' ').unwrap_or(("", &t));
            players.push(Player {
                number: number.to_string(),
                name: name.trim().to_string(),
                toi: "00:00".into(),
                ev: "00:00".into(),
                pp: "00:00".into(),
                sh: "00:00".into(),
            });
        } else {
            totals.push(t);
        }
    }
    flush(&mut players, &mut totals);

    TeamReport {
        team,
        status,
        players,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
<table id="GameInfo"><tr><td>TOI Shift Report</td></tr><tr><td>Period 1 (03:16 Remaining)</td></tr></table>
<table><tr><td class="teamHeading + border">MONTRÉAL CANADIENS</td></tr>
<tr><td class="playerHeading + border" colspan="8">13 CAUFIELD, COLE</td></tr></table>
<table><tr><td class="bborder + lborder + bold">TOT</td><td class="bold">5</td><td class="bold">01:04</td>
<td class="bold">05:23</td><td class="bold">05:00</td><td class="bold">00:20</td><td class="bold">00:03</td></tr></table>
<table><tr><td class="playerHeading + border" colspan="8">30 MONTEMBEAULT, SAMUEL</td></tr></table>
"#;

    #[test]
    fn parses_players_and_totals() {
        let r = parse(SAMPLE);
        assert_eq!(r.team, "MONTRÉAL CANADIENS");
        assert_eq!(r.status, "Period 1 (03:16 Remaining)");
        assert_eq!(r.players.len(), 2);
        let p = &r.players[0];
        assert_eq!(
            (p.number.as_str(), p.name.as_str()),
            ("13", "CAUFIELD, COLE")
        );
        assert_eq!(
            (p.toi.as_str(), p.ev.as_str(), p.pp.as_str(), p.sh.as_str()),
            ("05:23", "05:00", "00:20", "00:03")
        );
        assert_eq!(r.players[1].toi, "00:00");
    }
}

#[cfg(test)]
mod tmp_real {
    #[test]
    fn real() {
        for f in ["/tmp/toi/TV.htm", "/tmp/toi/TH.htm"] {
            let r = super::parse(&std::fs::read_to_string(f).unwrap());
            println!("{} | {} | {}", r.team, r.status, r.players.len());
            for p in &r.players {
                println!(
                    "{:>3} {:<28} {} {} {} {}",
                    p.number, p.name, p.toi, p.ev, p.pp, p.sh
                );
            }
        }
    }
}
