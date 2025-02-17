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

## 3412995 (History Heuristic)

-   Added Butterfly History Heuristic, and used to order moves

Bench = 1718300

```
--------------------------------------------------
Results of HF-new-3412995 vs HF-old-698ac96 (10+0.1, 1t, 32MB, opening_book.epd):
Elo: 62.10 +/- 17.43, nElo: 76.69 +/- 21.06
LOS: 100.00 %, DrawRatio: 33.08 %, PairsRatio: 2.02
Games: 1046, Wins: 460, Losses: 275, Draws: 311, Points: 615.5 (58.84 %)
Ptnml(0-2): [31, 85, 173, 136, 98], WL/DD Ratio: 2.84
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 8f6fe0b (Static Exchange Evaluation)

-   Use Static Exchange Evaluation for ordering captures
-   Bench depth raised to 6

Bench = 4653857

```
--------------------------------------------------
Results of HF-new-8f6fe0b vs HF-old-3412995 (10+0.1, 1t, 32MB, opening_book.epd):
Elo: 41.24 +/- 13.55, nElo: 56.17 +/- 18.28
LOS: 100.00 %, DrawRatio: 41.21 %, PairsRatio: 1.79
Games: 1388, Wins: 540, Losses: 376, Draws: 472, Points: 776.0 (55.91 %)
Ptnml(0-2): [38, 108, 286, 176, 86], WL/DD Ratio: 2.04
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## b94707b (Optimisation Pass #2)

-   Added `#[inline]` to `BitIterator` (about a 25% search speedup)
-   Switch to using a move picker, which only calculates scores for moves if the
    TT move was not sufficient

Bench = 4392894

```
--------------------------------------------------
Results of HF-new-b94707b vs HF-old-8f6fe0b (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 21.32 +/- 9.86, nElo: 27.49 +/- 12.67
LOS: 100.00 %, DrawRatio: 39.34 %, PairsRatio: 1.34
Games: 2888, Wins: 1098, Losses: 921, Draws: 869, Points: 1532.5 (53.06 %)
Ptnml(0-2): [122, 252, 568, 331, 171], WL/DD Ratio: 2.97
LLR: 2.96 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 8c9625d (Late Move Reductions)

-   Added Late Move Reductions
-   Raised default bench depth to 9

Bench = 7905965

```
--------------------------------------------------
Results of HF-new-8c9625d vs HF-old-b94707b (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 126.03 +/- 22.60, nElo: 165.20 +/- 27.13
LOS: 100.00 %, DrawRatio: 29.52 %, PairsRatio: 5.00
Games: 630, Wins: 326, Losses: 107, Draws: 197, Points: 424.5 (67.38 %)
Ptnml(0-2): [9, 28, 93, 105, 80], WL/DD Ratio: 1.91
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 0a8f348 (Re-tune evaluation)

This was me implementing a static library that can be used by [Gediminas
Mesaitis's Texel Tuner](https://github.com/GediminasMasaitis/texel-tuner/) to
tune the evaluation more easily. The evaluation terms have been scaled down also
so a pawn is ~100 rather than ~1000.

No improvements engine-side were expected so the SPRT is a regression test.

-   Scaled down evaluation so a pawn is worth ~100 rather than ~1000
-   Retuned evaluation

Bench = 6282080

```
--------------------------------------------------
Results of HF-new-0a8f348 vs HF-old-8c9625d (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 5.62 +/- 6.02, nElo: 7.54 +/- 8.07
LOS: 96.64 %, DrawRatio: 35.51 %, PairsRatio: 1.08
Games: 7114, Wins: 2053, Losses: 1938, Draws: 3123, Points: 3614.5 (50.81 %)
Ptnml(0-2): [261, 844, 1263, 897, 292], WL/DD Ratio: 0.83
LLR: 2.95 (-2.94, 2.94) [-5.00, 0.00]
--------------------------------------------------
```

## 74fde33 (Mobility Evaluation)

-   Added mobility evaluation terms for knight, bishop, rooks, and queens
    (number of spaces that attacked by piece that are not attacked by pawns)

Bench = 6533808

```
--------------------------------------------------
Results of HF-new-74fde33 vs HF-old-0a8f348 (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 18.59 +/- 9.18, nElo: 23.60 +/- 11.64
LOS: 100.00 %, DrawRatio: 34.64 %, PairsRatio: 1.29
Games: 3424, Wins: 1184, Losses: 1001, Draws: 1239, Points: 1803.5 (52.67 %)
Ptnml(0-2): [149, 340, 593, 439, 191], WL/DD Ratio: 1.58
LLR: 2.97 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 3f154ee (POPCNT Optimisations)

Realised that rust wasn't emitting popcnt instructions for my bitboards, and
that I had to explicitly specify `targetcpu=native` in the compiler flags.

-   Use `popcnt` on platforms that support it
-   Found a bunch more functions that were not being inlined that really should
    be by looking at profiles

Bench = 6533808

```
--------------------------------------------------
Results of HF-new-3f154ee vs HF-old-74fde33 (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 28.58 +/- 11.19, nElo: 39.10 +/- 15.23
LOS: 100.00 %, DrawRatio: 33.53 %, PairsRatio: 1.44
Games: 1998, Wins: 643, Losses: 479, Draws: 876, Points: 1081.0 (54.10 %)
Ptnml(0-2): [49, 223, 335, 299, 93], WL/DD Ratio: 0.89
LLR: 2.96 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## e5fc493 (Passed Pawn Evaluation)

-   Added evaluation terms for passed pawns

Bench = 6719433

```
--------------------------------------------------
Results of HF-new-e5fc493 vs HF-old-3f154ee (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 31.25 +/- 12.00, nElo: 40.88 +/- 15.60
LOS: 100.00 %, DrawRatio: 34.94 %, PairsRatio: 1.44
Games: 1906, Wins: 673, Losses: 502, Draws: 731, Points: 1038.5 (54.49 %)
Ptnml(0-2): [57, 197, 333, 250, 116], WL/DD Ratio: 1.35
LLR: 2.96 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 6b9f020 (Doubled Pawn Evaluation)

-   Added evaluation term for doubled pawns

Bench = 7161910

```
--------------------------------------------------
Results of HF-new-6b9f020 vs HF-old-e5fc493 (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 23.20 +/- 10.18, nElo: 30.83 +/- 13.49
LOS: 100.00 %, DrawRatio: 34.98 %, PairsRatio: 1.31
Games: 2550, Wins: 842, Losses: 672, Draws: 1036, Points: 1360.0 (53.33 %)
Ptnml(0-2): [76, 283, 446, 335, 135], WL/DD Ratio: 1.13
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 6930496 (Isolated Pawn Evaluation)

-   Added evaluation term for isolated pawns

Bench = 5937946

```
--------------------------------------------------
Results of HF-new-6930496 vs HF-old-6b9f020 (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 7.66 +/- 5.33, nElo: 10.09 +/- 7.03
LOS: 99.76 %, DrawRatio: 34.65 %, PairsRatio: 1.10
Games: 9396, Wins: 2924, Losses: 2717, Draws: 3755, Points: 4801.5 (51.10 %)
Ptnml(0-2): [364, 1095, 1628, 1192, 419], WL/DD Ratio: 1.22
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 2faf458 (Pawn Shield Evaluation)

-   Added evaluation terms for pawn shield in front of king

Bench = 6624664

```
--------------------------------------------------
Results of HF-new-2faf458 vs HF-old-6930496 (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 19.29 +/- 9.23, nElo: 25.30 +/- 12.09
LOS: 100.00 %, DrawRatio: 36.67 %, PairsRatio: 1.27
Games: 3174, Wins: 1043, Losses: 867, Draws: 1264, Points: 1675.0 (52.77 %)
Ptnml(0-2): [114, 329, 582, 391, 171], WL/DD Ratio: 1.14
LLR: 2.97 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 9c42b54 (Late Move Pruning)

-   Added late move pruning

Bench = 2418178

```
--------------------------------------------------
Results of HF-new-9c42b54 vs HF-old-2faf458 (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 100.84 +/- 19.90, nElo: 140.06 +/- 26.11
LOS: 100.00 %, DrawRatio: 31.76 %, PairsRatio: 3.64
Games: 680, Wins: 292, Losses: 100, Draws: 288, Points: 436.0 (64.12 %)
Ptnml(0-2): [5, 45, 108, 117, 65], WL/DD Ratio: 0.71
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 79e2a14 (Futility Pruning)

-   Added futility pruning

Bench = 2153601

```
--------------------------------------------------
Results of HF-new-79e2a14 vs HF-old-9c42b54 (10+0.1, 1t, 32MB, 8moves_v3.pgn):
Elo: 8.63 +/- 5.61, nElo: 12.21 +/- 7.93
LOS: 99.87 %, DrawRatio: 38.06 %, PairsRatio: 1.15
Games: 7372, Wins: 2051, Losses: 1868, Draws: 3453, Points: 3777.5 (51.24 %)
Ptnml(0-2): [221, 842, 1403, 973, 247], WL/DD Ratio: 0.71
LLR: 2.96 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 2a77ebc (Internal Iterative Reduction)

-   Added internal iterative reduction

Bench = 2028673

```
--------------------------------------------------
Results of HF-new-2a77ebc vs HF-old-79e2a14 (10+0.1, 1t, 32MB, UHO_Lichess_4852_v1.epd):
Elo: 15.41 +/- 7.93, nElo: 21.59 +/- 11.09
LOS: 99.99 %, DrawRatio: 38.59 %, PairsRatio: 1.26
Games: 3768, Wins: 1181, Losses: 1014, Draws: 1573, Points: 1967.5 (52.22 %)
Ptnml(0-2): [111, 402, 727, 497, 147], WL/DD Ratio: 1.16
LLR: 2.96 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 869a8f7 (Hard and Soft Bounds)

-   Add simple hard and soft bounds for time management

Bench = 2028673 (unchanged)

```
--------------------------------------------------
Results of HF-new-869a8f7 vs HF-old-2a77ebc (10+0.1, 1t, 32MB, UHO_Lichess_4852_v1.epd):
Elo: 68.52 +/- 17.30, nElo: 92.11 +/- 22.65
LOS: 100.00 %, DrawRatio: 37.39 %, PairsRatio: 2.58
Games: 904, Wins: 371, Losses: 195, Draws: 338, Points: 540.0 (59.73 %)
Ptnml(0-2): [20, 59, 169, 133, 71], WL/DD Ratio: 1.32
LLR: 2.96 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 491d437 (Node Based Time Management)

-   Adjust soft bound based on effort spent on best move

Bench = 2028673 (unchanged)

```
--------------------------------------------------
Results of HF-new-491d437 vs HF-old-869a8f7 (10+0.1, 1t, 32MB, UHO_Lichess_4852_v1.epd):
Elo: 10.89 +/- 6.53, nElo: 15.08 +/- 9.03
LOS: 99.95 %, DrawRatio: 39.32 %, PairsRatio: 1.14
Games: 5682, Wins: 1797, Losses: 1619, Draws: 2266, Points: 2930.0 (51.57 %)
Ptnml(0-2): [175, 631, 1117, 677, 241], WL/DD Ratio: 1.33
LLR: 2.95 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```

## 89ac99a (Aspiration Windows)

-   Add aspiration window search

Bench = 1837505

Trying out OpenBench for running SPRTs.

```
Elo   | 32.99 +- 11.99 (95%)
SPRT  | 10.0+0.10s Threads=1 Hash=32MB
LLR   | 2.97 (-2.94, 2.94) [0.00, 5.00]
Games | N: 1690 W: 585 L: 425 D: 680
Penta | [46, 152, 320, 250, 77]
```

## 8c0ecd6 (Adaptive Aspiration Windows)

-   Use adaptive aspiration windows -- lower window and reduce depth on fail
    lows, increase window and reset depth on fail highs
-   Some parameter tweaking on initial aspiration window size

Bench = 1907036

```
Elo   | 19.47 +- 9.03 (95%)
SPRT  | 10.0+0.10s Threads=1 Hash=32MB
LLR   | 2.99 (-2.94, 2.94) [0.00, 5.00]
Games | N: 2948 W: 941 L: 776 D: 1231
Penta | [80, 319, 555, 396, 124]
```

## 2516f32 (Open Files)

-   Added semi- and fully open file bonuses (or maluses) for rooks, queens, and
    kings

Bench = 1519323

```
Elo   | 28.14 +- 11.17 (95%)
SPRT  | 10.0+0.10s Threads=1 Hash=32MB
LLR   | 2.95 (-2.94, 2.94) [0.00, 5.00]
Games | N: 1980 W: 690 L: 530 D: 760
Penta | [50, 201, 377, 263, 99]
```

## 7991a2a (Additional Pawn Evaluation)

-   Added protected and phalanx pawn evaluation terms

Bench = 1674830

```
Elo   | 18.26 +- 8.71 (95%)
SPRT  | 10.0+0.10s Threads=1 Hash=32MB
LLR   | 2.95 (-2.94, 2.94) [0.00, 5.00]
Games | N: 3066 W: 982 L: 821 D: 1263
Penta | [84, 320, 598, 413, 118]
```

## 328fcaa (Fix bug in mobility eval)

-   Found a bug for mobility eval, where when calculating black pawn attacks
    (used for calculating safe mobility for white), white pawn attacks were
    used...

Bench = 1519426

```
Elo   | 22.19 +- 9.77 (95%)
SPRT  | 10.0+0.10s Threads=1 Hash=32MB
LLR   | 2.95 (-2.94, 2.94) [0.00, 5.00]
Games | N: 2524 W: 814 L: 653 D: 1057
Penta | [67, 264, 484, 335, 112]
```

## 079a567 (Outpost Evaluation)

-   Apply bonus to knights and bishops in outposts

Bench = 1558639

```
Elo   | 8.11 +- 5.87 (95%)
SPRT  | 10.0+0.10s Threads=1 Hash=32MB
LLR   | 2.51 (-2.94, 2.94) [0.00, 5.00]
Games | N: 6816 W: 2086 L: 1927 D: 2803
Penta | [195, 806, 1308, 843, 256]
```

## 7ac3f8b (King Zone Attacks Evaluation)

-   Apply bonus to pieces attacking the opponent king zone

Bench = 1488160

```
Elo   | 33.45 +- 11.80 (95%)
SPRT  | 10.0+0.10s Threads=1 Hash=32MB
LLR   | 2.99 (-2.94, 2.94) [0.00, 5.00]
Games | N: 1594 W: 529 L: 376 D: 689
Penta | [31, 150, 318, 231, 67]
```

## 90b8f4a (Virtual Mobility Evaluation)

-   Add evaluation term for virtual mobility -- pretend that the king is a queen
    and apply a bonus/penalty based on how many squares the "virtual" queen can
    see.

Bench = 1693822

```
Elo   | 16.03 +- 8.04 (95%)
SPRT  | 10.0+0.10s Threads=1 Hash=32MB
LLR   | 2.97 (-2.94, 2.94) [0.00, 5.00]
Games | N: 3492 W: 1065 L: 904 D: 1523
Penta | [75, 408, 683, 441, 139]
```

## c42302b (Transposition Lookup in Quiesence)

-   Perform transposition table lookup in quiescence, to skip re-evaluating the
    same position, produce cutoffs, and order moves
-   Raised default bench depth to 12

Bench = 8047136

```
Elo   | 54.10 +- 15.31 (95%)
SPRT  | 10.0+0.10s Threads=1 Hash=32MB
LLR   | 2.95 (-2.94, 2.94) [0.00, 5.00]
Games | N: 1010 W: 368 L: 212 D: 430
Penta | [18, 83, 180, 173, 51]
```