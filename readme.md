# 🏒🥅 NHL Time On Ice

🏒🥅 Feed me an NHL game id and I'll serve the running Time On Ice tallies in a Ratatui window 🦀

Looks like this:

```
┌ NHL Time On Ice ──────────────────────────────────────────────────────────────────────────────────────────────────┐
│MONTRÉAL CANADIENS @ WASHINGTON CAPITALS   |   Game 020106   |   Final   |   Official                              │
└───────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
┌ MONTRÉAL CANADIENS ─────────────────────────────────────┐┌ WASHINGTON CAPITALS ───────────────────────────────────┐
│#   Player                  TOI   EV    PP    SH    STT  ││#   Player                 TOI   EV    PP    SH    STT  │
│13  CAUFIELD, COLE          19:53 17:47 02:06 00:00 02:06││4   HOLL, JUSTIN           13:08 13:08 00:00 00:00 00:00│
│14  SUZUKI, NICK            20:15 18:09 02:06 00:00 02:06││6   CHYCHRUN, JAKOB        23:03 18:27 04:18 00:18 04:36│
│15  NEWHOOK, ALEX           15:52 13:07 00:09 02:36 02:45││8   OVECHKIN, ALEX         13:14 08:35 04:39 00:00 04:39│
│17  ANDERSON, JOSH          13:49 11:45 00:00 02:04 02:04││9   LEONARD, RYAN          14:55 10:49 04:06 00:00 04:06│
│20  SLAFKOVSKÝ, JURAJ       19:40 17:34 02:06 00:00 02:06││17  STROME, DYLAN          18:56 14:54 04:02 00:00 04:02│
│21  GUHLE, KAIDEN           22:22 18:40 00:18 03:24 03:42││21  PROTAS, ALIAKSEI       17:37 16:16 00:02 01:19 01:21│
│22  KREIDER, CHRIS          15:31 13:25 02:06 00:00 02:06││22  JENNER, BOONE          15:08 13:58 00:04 01:06 01:10│
│24  DANAULT, PHILLIP        17:39 11:52 00:33 05:14 05:47││25  KYROU, JORDAN          15:15 11:36 03:39 00:00 03:39│
│42  ENGSTROM, ADAM          11:57 11:49 00:00 00:08 00:08││27  LILJEGREN, TIMOTHY     16:57 16:36 00:00 00:21 00:21│
│45  CARRIER, ALEXANDRE      20:42 17:22 00:00 03:20 03:20││34  SOURDIF, JUSTIN        16:05 15:56 00:09 00:00 00:09│
│48  HUTSON, LANE            24:14 22:03 02:06 00:05 02:11││42  FEHERVARY, MARTIN      21:54 19:22 00:29 02:03 02:32│
│53  DOBSON, NOAH            22:21 17:40 00:18 04:23 04:41││43  WILSON, TOM            17:13 12:21 03:47 01:05 04:52│
│62  BECK, OWEN              07:46 07:46 00:00 00:00 00:00││44  HUTSON, COLE           17:07 13:30 03:37 00:00 03:37│
│71  EVANS, JAKE             12:33 07:25 00:00 05:08 05:08││62  PROTAS, ILYA           12:22 08:37 03:45 00:00 03:45│
│72  XHEKAJ, ARBER           14:16 09:58 00:00 04:18 04:18││63  MIROSHNICHENKO, IVAN   10:01 08:28 01:33 00:00 01:33│
│75  DOBES, JAKUB            57:14 47:26 02:00 07:48 09:48││72  BEAUVILLIER, ANTHONY   12:54 11:50 01:04 00:00 01:04│
└─────────────────────────────────────────────────────────┘└────────────────────────────────────────────────────────┘
Tab/←/→: switch table   1-7: sort by # / Player / TOI / EV / PP / SH / STTot (again to reverse)   q: quit
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
| `1`–`7`       | Sort the focused table by `#`, `Player`, `TOI`, `EV`, `PP`, `SH`,SPT |
| same key again | Reverse the sort order                                              |
| `q`, `Esc`    | Quit                                                                |

Each table sorts independently, and the sort is kept across refreshes.
