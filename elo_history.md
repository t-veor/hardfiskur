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

## c16c8db (Transposition Table Cutoffs)

-   Use transposition table entries to produce cutoffs

Bench = 4671980

```
--------------------------------------------------
Results of HF-new-c16c8db vs HF-old-9c3e6bc (10+0.1, NULL, NULL, 8moves_v3.pgn):
Elo: 56.86 +/- 15.93, nElo: 76.50 +/- 21.03
LOS: 100.00 %, DrawRatio: 40.46 %, PairsRatio: 2.28
Games: 1048, Wins: 458, Losses: 288, Draws: 302, Points: 609.0 (58.11 %)
Ptnml(0-2): [28, 67, 212, 141, 76], WL/DD Ratio: 3.51
LLR: 2.94 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 557a26e (Texel-tuned Eval)

-   Added texel-tuned evaluation function (eval is still simple material + PSTs)

Bench = 5975902

```
--------------------------------------------------
Results of HF-new-557a26e vs HF-old-c16c8db (10+0.1, NULL, NULL, 8moves_v3.pgn):
Elo: 184.54 +/- 27.93, nElo: 224.43 +/- 28.23
LOS: 100.00 %, DrawRatio: 25.09 %, PairsRatio: 7.38
Games: 582, Wins: 376, Losses: 93, Draws: 113, Points: 432.5 (74.31 %)
Ptnml(0-2): [6, 20, 73, 69, 123], WL/DD Ratio: 5.08
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 8659c6e (Principal Variation Search)

-   Added principal variation search

Bench = 4872335

```
--------------------------------------------------
Results of HF-new-8659c6e vs HF-old-557a26e (10+0.1, NULL, NULL, 8moves_v3.pgn):
Elo: 17.48 +/- 9.05, nElo: 21.22 +/- 10.96
LOS: 99.99 %, DrawRatio: 39.07 %, PairsRatio: 1.25
Games: 3860, Wins: 1558, Losses: 1364, Draws: 938, Points: 2027.0 (52.51 %)
Ptnml(0-2): [212, 310, 754, 380, 274], WL/DD Ratio: 5.08
LLR: 2.98 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## a3c9a98 (Optimisation Pass #1)

General optimization pass based on some profiling.

-   Changed an `unreachable!()` to an `unreachable_unchecked` in
    `Piece::piece_type` (this single change increased NPS from ~2.86 million/s
    to ~3.34 million/s)
-   Changed tapered eval to use packed middlegame and endgame scores
-   General optimisation of evaluation function
-   Changed move ordering to pick the top move each time (should save as in most
    nodes cutoffs occur in the first few moves, so we expect O(kn) time rather
    than O(n log n))

The search algorithm should be the same, but due to the move ordering changes
moves with the same score may be picked in a different order. This is the reason
for the slightly different bench.

Bench = 4349765

```
--------------------------------------------------
Results of HF-new-a3c9a98 vs HF-old-8659c6e (10+0.1, 1t - NULL, 32MB - NULL, opening_book.epd):
Elo: 65.57 +/- 18.06, nElo: 79.19 +/- 21.28
LOS: 100.00 %, DrawRatio: 37.70 %, PairsRatio: 2.13
Games: 1024, Wins: 486, Losses: 295, Draws: 243, Points: 607.5 (59.33 %)
Ptnml(0-2): [35, 67, 193, 106, 111], WL/DD Ratio: 4.51
LLR: 2.96 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## b3298ff (Reverse Futility Pruning)

-   Added Reverse Futility Pruning when off-PV and not in check

Bench = 3432514

(Accidentally ran the SPRT test against d3b6fca instead of a3c9a98 but the only
difference is the Elo history file.)

```
--------------------------------------------------
Results of HF-new-b3298ff vs HF-old-d3b6fca (10+0.1, 1t, 32MB, opening_book.epd):
Elo: 66.34 +/- 17.87, nElo: 82.53 +/- 21.69
LOS: 100.00 %, DrawRatio: 35.70 %, PairsRatio: 2.27
Games: 986, Wins: 459, Losses: 273, Draws: 254, Points: 586.0 (59.43 %)
Ptnml(0-2): [31, 66, 176, 126, 94], WL/DD Ratio: 4.68
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 77d056d (Null Move Pruning)

-   Added Null Move Pruning when off-PV and not in check

Bench = 3313576

```
--------------------------------------------------
Results of HF-new-77d056d vs HF-old-b3298ff (10+0.1, 1t, 32MB, opening_book.epd):
Elo: 43.96 +/- 14.79, nElo: 53.83 +/- 17.91
LOS: 100.00 %, DrawRatio: 37.21 %, PairsRatio: 1.65
Games: 1446, Wins: 618, Losses: 436, Draws: 392, Points: 814.0 (56.29 %)
Ptnml(0-2): [56, 115, 269, 157, 126], WL/DD Ratio: 3.48
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 698ac96 (Killer Move Heuristic)

-   Add killer move table and order quiet moves by killer move

Bench = 1912275

```
--------------------------------------------------
Results of HF-new-698ac96 vs HF-old-77d056d (10+0.1, 1t, 32MB, opening_book.epd):
Elo: 146.52 +/- 26.12, nElo: 172.71 +/- 27.35
LOS: 100.00 %, DrawRatio: 25.16 %, PairsRatio: 4.80
Games: 620, Wins: 360, Losses: 113, Draws: 147, Points: 433.5 (69.92 %)
Ptnml(0-2): [13, 27, 78, 84, 108], WL/DD Ratio: 3.33
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```
