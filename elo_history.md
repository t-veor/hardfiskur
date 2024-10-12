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

## 2957c8f (Transposition Table pt. 1)

-   Add transposition table
-   Use transposition table only for move ordering
-   Bench depth raised to 5

Bench = 5110741

```
--------------------------------------------------
Results of HF-new-2957c8f vs HF-old-5ada119 (10+0.1, NULL, NULL, 8moves_v3.pgn):
Elo: 149.96 +/- 24.60, nElo: 191.81 +/- 27.80
LOS: 100.00 %, DrawRatio: 30.33 %, PairsRatio: 6.21
Games: 600, Wins: 352, Losses: 108, Draws: 140, Points: 422.0 (70.33 %)
Ptnml(0-2): [6, 23, 91, 81, 99], WL/DD Ratio: 4.06
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 9c3e6bc (Partial Search Results)

-   Accept partial search results if the at least one move has been searched, as
    the TT and move ordering guarantee the move from the previous generation is
    first to be searched

Bench = 5110741 (no change to fixed-depth searches)

```
--------------------------------------------------
Results of HF-new-9c3e6bc vs HF-old-2957c8f (10+0.1, NULL, NULL, 8moves_v3.pgn):
Elo: 42.21 +/- 14.79, nElo: 49.29 +/- 17.09
LOS: 100.00 %, DrawRatio: 36.90 %, PairsRatio: 1.58
Games: 1588, Wins: 710, Losses: 518, Draws: 360, Points: 890.0 (56.05 %)
Ptnml(0-2): [77, 117, 293, 151, 156], WL/DD Ratio: 5.37
LLR: 2.97 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```
