# Coerceo

[Coerceo](http://coerceo.com/) is a board game like Go, where you can capture your opponent's pieces by enclosing them. However, Coereceo uses tetrahedral pieces on a hexagonal board that shrinks as time goes on. It won the MENSA Mind Games in 2012.

![Coerceo game screenshot](https://raw.githubusercontent.com/NPN/coerceo/master/screenshot.png)

## How to Play

The [official rules](http://coerceo.com/rules.html) are on the Coerceo website. You can download a binary of the game from the [releases page](https://github.com/NPN/coerceo/releases) or compile it yourself with:
```
cargo run --release
```

There are [variations](http://coerceo.com/rules2.html) on the game, but this program does not support any of them.

## Notation

The notation that the AI debug output uses to describe the principal variation (see [Wikipedia](https://en.wikipedia.org/wiki/Principal_variation) or the [Chess Programming Wiki](https://chessprogramming.wikispaces.com/Principal+variation)) does not use the [official notation](http://coerceo.com/Coerceo%20GameNotation.pdf). Rather, it uses an notation based off of the notation for [Gliński's hexagonal chess](https://en.wikipedia.org/wiki/Hexagonal_chess#Gli%C5%84ski's_hexagonal_chess). The files (columns) are lettered 'a-e' from left to right. The ranks (row) are numbered '1-5' from bottom to top. Each row makes a 60 degree bend at the c file, which gives each file a 'V' shape. The fields are lettered from 'a-e' counterclockwise like so:
 ```text
     _______
    /\     /\
   /  \ f /  \
  / a  \ /  e \
 (------X------)
  \ b  / \  d /
   \  / c \  /
    \/_____\/
 ```
 This ordering mirrors the files going left to right, with an extra 'f' field at the top.

 Moves are written as `origin-destination` pairs like `b3f-c4b`. If a piece moves within the same hex, the pair can be shortened to `b3f-d`.

 The beginning of a sample game might look something like this:
 ```text
 1. d1e-c b3f-c4b
 2. c5e-c e3f-b
 ```

 Note that this notation is very incomplete, as it does not have any way to write captures or exchanges.

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

This game uses the font [Fira Sans](https://github.com/mozilla/Fira) which is licensed under the OFL v1.1.

All rights, trademarks, copyrights, concepts, etc. of the game Coerceo belong to the Coerceo Company.
