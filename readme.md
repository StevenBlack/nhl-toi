# 🏒🥅 NHL Time On Ice

🏒🥅 Feed me an NHL game id and I'll serve the running Time On Ice tallies in a Ratatui window 🦀

Looks like this:

```
┌ NHL Time On Ice ──────────────────────────────────────────────────────────────────────────────────────────────────┐
│MONTRÉAL CANADIENS @ WASHINGTON CAPITALS   |   Game 020106   |   End of Period 1   |   updated 2s ago              │
└───────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ MONTRÉAL CANADIENS ─────────────────────────────────────┐┌ WASHINGTON CAPITALS ───────────────────────────────────┐
│#   Player                        TOI   EV    PP    SH   ││#   Player                       TOI   EV    PP    SH   │
│13  CAUFIELD, COLE                05:49 05:49 00:00 00:00││4   HOLL, JUSTIN                 05:15 05:15 00:00 00:00│
│14  SUZUKI, NICK                  05:17 05:17 00:00 00:00││6   CHYCHRUN, JAKOB              08:45 06:40 02:05 00:00│
│15  NEWHOOK, ALEX                 05:30 03:49 00:00 01:41││8   OVECHKIN, ALEX               05:52 03:23 02:29 00:00│
│17  ANDERSON, JOSH                05:29 03:50 00:00 01:39││9   LEONARD, RYAN                06:02 03:55 02:07 00:00│
│20  SLAFKOVSKÝ, JURAJ             05:30 05:30 00:00 00:00││17  STROME, DYLAN                06:56 04:46 02:10 00:00│
│21  GUHLE, KAIDEN                 07:38 05:09 00:00 02:29││21  PROTAS, ALIAKSEI             04:36 04:36 00:00 00:00│
│22  KREIDER, CHRIS                04:16 04:16 00:00 00:00││22  JENNER, BOONE                05:09 05:09 00:00 00:00│
│24  DANAULT, PHILLIP              05:10 03:01 00:00 02:09││25  KYROU, JORDAN                04:49 03:09 01:40 00:00│
│42  ENGSTROM, ADAM                04:22 04:22 00:00 00:00││27  LILJEGREN, TIMOTHY           05:32 05:32 00:00 00:00│
│45  CARRIER, ALEXANDRE            08:05 05:39 00:00 02:26││34  SOURDIF, JUSTIN              04:02 04:02 00:00 00:00│
│48  HUTSON, LANE                  07:07 07:07 00:00 00:00││42  FEHERVARY, MARTIN            03:35 03:32 00:03 00:00│
│53  DOBSON, NOAH                  08:07 06:44 00:00 01:23││43  WILSON, TOM                  04:43 02:38 02:05 00:00│
│62  BECK, OWEN                    02:14 02:14 00:00 00:00││44  HUTSON, COLE                 07:39 05:52 01:47 00:00│
│71  EVANS, JAKE                   04:33 02:28 00:00 02:05││62  PROTAS, ILYA                 05:11 03:30 01:41 00:00│
│72  XHEKAJ, ARBER                 04:52 03:32 00:00 01:20││63  MIROSHNICHENKO, IVAN         04:46 03:30 01:16 00:00│
│75  DOBES, JAKUB                  20:00 16:12 00:00 03:48││72  BEAUVILLIER, ANTHONY         03:57 03:57 00:00 00:00│
│76  BOLDUC, ZACHARY               03:40 03:40 00:00 00:00││73  DESHARNAIS, VINCENT          05:39 05:39 00:00 00:00│
│77  DACH, KIRBY                   03:07 03:07 00:00 00:00││79  LINDGREN, CHARLIE            20:00 16:12 03:48 00:00│
│85  TEXIER, ALEXANDRE             03:26 03:26 00:00 00:00││89  TUCH, ALEX                   05:32 03:55 01:37 00:00│
│                                                         ││                                                        │
└─────────────────────────────────────────────────────────┘└────────────────────────────────────────────────────────┘
Tab/←/→: switch table   1-6: sort by # / Player / TOI / EV / PP / SH (again to reverse)   q: quit
```
## Usage

```
cargo run                           # default game (020106, season 20262027)
cargo run -- <game id> [season]     # e.g. cargo run -- 020107
cargo run -- 020106 20262027        # explicit game and season
```

- **Game id** is the six-digit number at the end of the NHL report URL, e.g. `020106` in
  `https://www.nhl.com/scores/htmlreports/20262027/TV020106.HTM`.
- **Season** is the two years joined together, e.g. `20262027`.
- The visitor and home Time On Ice reports (`TV…` and `TH…`) are re-fetched every 10 seconds.

### Keys

| Key           | Action                                                              |
|---------------|---------------------------------------------------------------------|
| `Tab`, `←`/`→` | Switch between the visitor and home table (focused one is cyan)     |
| `1`–`6`       | Sort the focused table by `#`, `Player`, `TOI`, `EV`, `PP`, `SH`    |
| same key again | Reverse the sort order                                              |
| `q`, `Esc`    | Quit                                                                |

Each table sorts independently, and the sort is kept across refreshes.
