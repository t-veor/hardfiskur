# Elo History

# df02cdc (Initial)

-   Negamax search, fixed depth 4
-   Alpha-beta pruning
-   Basic move ordering ()
-   Material evaluation and simple piece square tables

# 5ad2154 (Quiescence search)

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

# 0f38185 (Iterative Deepening)

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
