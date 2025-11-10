A chess engine I made from scratch, with a lot of help from
- The [Chess Programming Wiki](https://www.chessprogramming.org)
- jordanbray's [movegen crate](https://github.com/jordanbray/chess)
- analog-hors's [blog post](https://analog-hors.github.io/site/magic-bitboards/) on magic tables 
- Wikipedia

Most of the code is devoted to board representation, move generation, and UCI protocol because that's what I found the most interesting.
The board is represented with bitboards and move generation is done with simple magic tables.
The search algorithm uses negamax with alpha-beta pruning, and the current evaluation function uses material count and a clone of [PeSTO's evaluation function](https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function).
When (not if!) I improve this more, the evaluation and search will get some upgrades.
