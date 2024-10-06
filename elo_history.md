# Elo History

## df02cdc (Initial)

-   Negamax search, fixed depth 4
-   Alpha-beta pruning
-   Basic move ordering (ad-hoc implementation from previous engine)
-   Material evaluation and simple piece square tables

## 5ad2154 (Quiescence search)

-   Quiescence search added

```
--------------------------------------------------
Results of HF-new-5ad2154 vs HF-old-df02cdc (10+0.1, 1t, MB, opening_book.epd):
Elo: 353.65 +/- 27.86, nElo: 508.31 +/- 21.53
LOS: 100.00 %, DrawRatio: 9.80 %, PairsRatio: 63.43
Games: 1000, Wins: 818, Losses: 49, Draws: 133, Points: 884.5 (88.45 %)
Ptnml(0-2): [2, 5, 49, 110, 334]
--------------------------------------------------
```

## 0f38185 (Iterative Deepening)

-   Iterative deepening search added
-   Simple time management (remaining_time / 20 + increment / 2) added

```
--------------------------------------------------
Results of HF-new-0f38185 vs HF-old-213162b (10+0.1, 1t, MB, opening_book.epd):
Elo: 271.78 +/- 23.29, nElo: 368.33 +/- 21.53
LOS: 100.00 %, DrawRatio: 19.00 %, PairsRatio: 30.15
Games: 1000, Wins: 744, Losses: 90, Draws: 166, Points: 827.0 (82.70 %)
Ptnml(0-2): [1, 12, 95, 116, 276]
--------------------------------------------------
```

## cf308e5 (Transposition Table)

This one was awful because I kept running into bugs in the transposition table
implementation finding bugs elsewhere in the search code. It now seems to be
playing much better than main now, so I think I've got most of them.

-   Added transposition table
-   Fixed a repetition draw evaluation bug and the search not being properly
    fail-hard
-   Added some debug options onto the engine (`d`, `ttentry`, `makemove`,
    `undomove`)

The TT implementation is very rough and I still need to fix the bucketing
implementation, and this time unit test it so I don't waste as much time trying
to figure out what's wrong.

```
--------------------------------------------------
Results of HF-new-cf308e5 vs HF-old-a968384 (10+0.1, 1t, MB, opening_book.epd):
Elo: 222.41 +/- 21.33, nElo: 291.80 +/- 21.53
LOS: 100.00 %, DrawRatio: 19.20 %, PairsRatio: 11.62
Games: 1000, Wins: 671, Losses: 106, Draws: 223, Points: 782.5 (78.25 %)
Ptnml(0-2): [2, 30, 96, 145, 227]
--------------------------------------------------
```

## 68b3446 (Move Ordering pt. 1)

Replaced the weird ad-hoc move ordering with something a bit more sound.

-   Order hash move first
-   Order captures by MVV-LVA
-   Add killer move table and order quiets by killer move

```
--------------------------------------------------
Results of HF-new-68b3446 vs HF-old-11632cf (10+0.1, 1t, MB, opening_book.epd):
Elo: 45.78 +/- 16.30, nElo: 61.22 +/- 21.53
LOS: 100.00 %, DrawRatio: 36.60 %, PairsRatio: 1.78
Games: 1000, Wins: 381, Losses: 250, Draws: 369, Points: 565.5 (56.55 %)
Ptnml(0-2): [24, 90, 183, 137, 66]
--------------------------------------------------
```

## 6625208 (Move Ordering pt 2, Static Exchange Evaluation)

Added a static exchange evaluation method, which was a bit hard for me to get my
head around so I've commented quite thoroughly what I think is going on in the
algorithm.

It didn't actually improve the ELO that much but that's sort of expected I
think -- I expect it to help more when introducing features that are more
dependent on move ordering.

-   Added SEE (Static Exchange Evaluation)
-   Use SEE to split captures into winning and losing captures, and put losing
    captures behind quiet moves in move ordering

```
--------------------------------------------------
Results of HF-new-6625208 vs HF-old-68b3446 (10+0.1, 1t, MB, opening_book.epd):
Elo: 16.69 +/- 15.26, nElo: 23.60 +/- 21.53
LOS: 98.41 %, DrawRatio: 43.60 %, PairsRatio: 1.24
Games: 1000, Wins: 332, Losses: 284, Draws: 384, Points: 524.0 (52.40 %)
Ptnml(0-2): [28, 98, 218, 110, 46]
--------------------------------------------------
```

I was surprised how close to even this was so I ran a SPRT test with elo1=5, and
it definitely seems like it's helping the engine a bit:

```
SPRT test finished: (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
Results of HF-new-6625208 vs HF-old-68b3446 (10+0.1, 1t, MB, opening_book.epd):
Elo: 12.79 +/- 7.08, nElo: 18.22 +/- 10.08
LOS: 99.98 %, DrawRatio: 44.35 %, PairsRatio: 1.18
Games: 4564, Wins: 1527, Losses: 1359, Draws: 1678, Points: 2366.0 (51.84 %)
Ptnml(0-2): [134, 448, 1012, 492, 196]
LLR: 2.96 (-2.94, 2.94) [0.00, 5.00]
--------------------------------------------------
```
