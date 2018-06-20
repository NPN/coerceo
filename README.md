# Coerceo

[Coerceo](http://coerceo.com/) is a board game like Go, where you can capture your opponent's pieces by enclosing them. However, Coereceo uses tetrahedral pieces on a hexagonal board that shrinks as time goes on. It won the MENSA Mind Games in 2012.

![Coerceo game screenshot](https://raw.githubusercontent.com/NPN/coerceo/master/screenshot.png)

## How to Play

The [official rules](http://coerceo.com/rules.html) are on the Coerceo website. You can download a binary of the game from the [releases page](https://github.com/NPN/coerceo/releases) or compile it yourself with:
```
cargo run --release
```

## AI Features

  * Negamax search
  * Alpha-beta pruning
  * Iterative deepening
  * Transposition table
  * Aspiration windows
  * Quiescence search
  * Delta pruning

## Legal

This game is licensed under the AGPLv3.

All rights, trademarks, copyrights, concepts, etc. of the game Coerceo belong to the Coerceo Company.
