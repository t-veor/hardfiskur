# Elo History

## 5c9988b (Initial)

-   Negamax search
-   Alpha-beta pruning
-   Material evaluation and simple piece square tables
-   Iterative Deepening (but no move ordering)
-   Simple time management (remaining_time / 20 + increment / 2)

Bench = 7972010

## ebdc236 (Quiescence Search)

-   Quiescence search

Bench = 505609605 (Whoops! No move ordering sucks huh!)

```
--------------------------------------------------
Results of HF-new-ebdc236 vs HF-old-5c9988b (10+0.1, 1t, MB, 8moves_v3.pgn):
Elo: 294.93 +/- 33.71, nElo: 371.80 +/- 27.13
LOS: 100.00 %, DrawRatio: 18.41 %, PairsRatio: 31.12
Games: 630, Wins: 505, Losses: 70, Draws: 55, Points: 532.5 (84.52 %)
Ptnml(0-2): [4, 4, 58, 51, 198]
LLR: 2.96 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

This version appears to have a bug that causes fastchess to hang sometimes where
if the time is too short, it doesn't report a best move. This is fixed in later
versions.

## 5ada119 (MVV-LVA)

-   Sort captures first
-   Use MVV-LVA for capture ordering

Bench = 3197660

```
--------------------------------------------------
Results of HF-new-5ada119 vs HF-old-ebdc236 (10+0.1, NULL, NULL, 8moves_v3.pgn):
Elo: 355.96 +/- 35.36, nElo: 482.39 +/- 25.48
LOS: 100.00 %, DrawRatio: 12.61 %, PairsRatio: 51.00
Games: 714, Wins: 598, Losses: 47, Draws: 69, Points: 632.5 (88.59 %)
Ptnml(0-2): [1, 5, 45, 54, 252], WL/DD Ratio: 8.00
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```
