# Coerceo

This is an unofficial clone of the strategic board game [Coerceo](http://coerceo.com/). Coerceo is like Go in that you enclose your opponent's pieces to capture them. However, Coereceo uses tetrahedral pieces on a hexagonal board that shrinks as time goes on. It won the MENSA Mind Games in 2012.

![The game in the Laurentius opening position](https://raw.githubusercontent.com/NPN/coerceo/master/assets/screenshot.png)

## How to Play

The [official rules](http://coerceo.com/rules3.html) are on the Coerceo website. Download a binary of the game from the [releases page](https://github.com/NPN/coerceo/releases) or see the [Compiling](#compiling) section below.

This game supports the Ocius or "short game" variation, which is [explained here](http://coerceo.com/Coerceo%20variation2%20shortgame.pdf).

### Game Interface Help

#### How do I exchange tiles?

When you have captured enough tiles, the "Exchange" button will appear at the bottom of the screen. Press it to start exchanging. Click an opponent's piece to exchange for it, or press the button again to stop exchanging and make a normal move.

#### How do I change the number of tiles needed to exchange for a piece?

By default, it takes two tiles to exchange for one piece. You can change this by selecting _Game_ → _One tile to exchange_ in the menu.

**Note**: Any changes to this setting will take place in the next game. You can see the setting for the current game in the status line at the top of the screen under "Welcome to Coerceo!"

#### How do I change the computer difficulty?

You can change the difficulty with the _Computer_ → _Search depth_ slider. Search depth is how many plies (a single turn taken by a player) ahead the computer will search. A smaller search depth makes the computer easier and faster. A larger search depth makes the computer more difficult and slower.

**Note**: Search depth is only an approximation of difficulty. At a depth of one, the computer is very easy to beat. With successively larger depths, the search gets exponentially slower and delivers diminishing returns on engine strength.

## Supported Platforms

OpenGL 2.0+ is required on desktop, and OpenGL ES 2.0+ is required on Android.

### Great

Run with few issues that do not significantly impact the experience.

* Linux x86-64
* macOS x86-64

### Okay

Still playable, but have issues that hamper the experience.

* Android ARMv7
  * Suspending the app in the middle of a game will cause the app to crash on reload. Thus, all games must be finished in one sitting.
  * When rotating the device, the screen does not redraw properly. (You can tap the screen after a rotation to fix this.)
  * The _Game_ → _Quit_ menu item freezes the game instead of quitting.
  * The `INTERNET`, `READ_EXTERNAL_STORAGE`, and `WRITE_EXTERNAL_STORAGE` permissions are requested even though the game does not use them.
* Windows x86-64
  * Moving the mouse causes lag. Thus, the game (and especially menus) must be interacted with slowly and patiently.
  * When minimizing or maximizing the window, the screen does not redraw properly. (You can move your mouse anywhere in the game to fix this.)

### Untested/Fail to Compile

* iOS
  * Currently [not supported](https://github.com/tomaka/glutin/issues/29) by glutin.
* macOS i686
  * [Not supported by winit](https://github.com/tomaka/winit/issues/78) (and probably soon to be deprecated).
* Other 32-bit and Android architectures 
  * Untested, but they would probably work if you could get them to compile.

## Computer AI

### Features

  * Negamax search
  * Alpha-beta pruning
  * Iterative deepening
  * Transposition table
  * Aspiration windows
  * Quiescence search
  * Delta pruning

### Notes

The main weakness of the AI is that its evaluation function only considers material, and does not take into account other factors like mobility. Thus, in quiet positions where there aren't many captures to be made, most positions will have the same score. This causes the AI to move the same piece back and forth until the contempt factor forces it to avoid a draw by threefold repetition.

Unfortunately, this makes the opening of a Laurentius game boring, as the computer is completely passive. (In Ocius mode, the board is small enough that this isn't a problem.) To get anything to happen, the human player must engage the computer. What would happen if you didn't engage the computer and also played as passively as possible? Just watch the computer play a Laurentius game against itself with a search depth of 2.

Other than that, I'm pretty pleased with how the AI turned out. It could be stronger, but that's out of the scope of this project. See the [Future Development](#future-development) section for possible avenues of improvement, though.

### Principal Variation Notation

**Note**: The following explanations assume a Laurentius board. An Ocius board is just a Laurentius board with the outer tiles removed.

The debug output (_Computer_ → _Show debug info_) prints the principal variation found at each depth of the iterative deepening search. The notation used is not the [official notation](http://coerceo.com/Coerceo%20GameNotation.pdf), but a notation based off of the notation for [Gliński's hexagonal chess](https://en.wikipedia.org/wiki/Hexagonal_chess#Gli%C5%84ski's_hexagonal_chess).

Currently, the notation is incomplete and suited for debug usage only. It consists of two forms, `Move(_, _)` and `Exchange(_)`, where the underscores stand for the origin/destination fields and the field of the piece to be exchanged, respectively.

A field is represented as a triplet of `[file][rank][field]`. The file (column) and rank (row) specify a tile, and the field identifies a field on that tile. The files are lettered 'a-e' from left to right. The ranks are numbered '1-5' from bottom to top. Each rank makes a 60 degree bend at the c-file, which gives each rank a 'V' shape. The fields are lettered from 'a-e' counterclockwise like so:

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

This ordering mirrors the files going left to right, with an extra f-field at the top.

Now, finding the file and field are simple, but the rank is more difficult. The simplest way to do it is to count from the bottom of the board. For example, the center tile of the board has the coordinates `c3` because it is the third tile from the bottom of the board in the c-file. Because of this system, the tiles at the bottom of the board are all in the same rank (`a1`, `b1`, `c1`, etc.) whereas the tiles at the top of the board all have different ranks (`a3`, `b4`, `c5`, `d4`, `e3`). In fact, `c5` is the only tile in the fifth rank.

As an example, let's find the coordinates of the top field of the bottom-right tile. We know this field is in the e-file because it is in the last column of the board. This tile is also the first in its file, so it is in the first rank. Looking at the diagram above, the top field is the f-field. Putting it all together, the coordinates of the this field are `e1f`.

Here is a sequence of possible moves for the start of a Laurentius game:

```
Move(d1e, d1c)
Move(b3f, c4b)
Move(c5e, c5c)
Move(e3f, e3b)

[a lot of moves and captures later...]

Exchange(a3f)
```

As mentioned earlier, this notation is incomplete. There is no way to notate piece or tile captures. See [Future Development](#future-development) for more information.
 
## Compiling

This game requires a minimum Rust version of 1.27.

### Linux and macOS

`cargo run --release` is enough to build the game. [cargo-bundle](https://github.com/burtonageo/cargo-bundle) is used to create the `.app` bundle for releases.

### Windows

Only the `x86_64-pc-windows-msvc` toolchain has been tested. If you are having trouble installing the Visual C++ Build Tools, there is an [illustrated guide](https://github.com/rust-lang-nursery/rustup.rs/issues/1363#issuecomment-369953262).

After installing Rust, compile with `cargo run --release`. Consider adding `-C target-feature=+crt-static` to `RUSTFLAGS` so that the resulting binary will run without requiring the CRT DLLs to be installed.

### Android

This game uses [android-rs-glue](https://github.com/tomaka/android-rs-glue) to compile for Android.

Follow the instructions in the README to compile the game. Remember to pass the build target to `cargo apk build` or set it in `[package.metadata.android]`. If you get an error about [`jni.h: No such file or directory`](https://github.com/tomaka/android-rs-glue/issues/163), downgrade the NDK to version r15c.

If you compile a release APK (and you should to improve performance), you must [sign it](https://developer.android.com/studio/publish/app-signing) before you can load it on your phone.

## Future Development

This game is not in development anymore. Version 1.0.0, as it stands, is "finished" and no new features will be added. Bug fixes or library upgrades may be made as time permits.

That being said, however, there is no shortage of possible features to add. Below is a list of ideas which I did not get to implement. If you decide to implement any of these, feel free to open a pull request.

If you happen to find any bugs, feel free to open an issue, but do note that I may choose to ignore them if they aren't too serious (like the bugs in the [Supported Platforms](#supported-platforms) section).

### Feature Ideas

* Better platform support
* Move explorer
* Save/restore games
* Time control
* Networked play
* Fancier UI (animated/draggable pieces)
* Sound effects
* Draw offering
* In-game tutorial
* Cura/12-piece winning rule
* More color schemes and piece styles
* Internationalization/accessibility

### AI Ideas

* Stronger AI
  * Better evaluation function
    * Mobility
    * Piece structure (chains, tile domination, etc.)
  * Better search algorithms
  * Shared transposition table
* Improved performance
  * Faster quiescence
  * Multithreading
  * SIMD
* Better difficulty levels
* Computer analysis/hint

### Notation Ideas

The notation could be converted to a more natural one where moves are written as `origin-destination` pairs like `b3f-b3d`. If a piece moves within the same tile, the pair could be shortened to `b3f-d`.

The beginning of a sample game, then, might look something like this:

```text
1. d1e-c b3f-c4b
2. c5e-c e3f-b
```

Ideas from other games and the official notation could be used to find a way to represent piece and tile captures. The biggest challenge is coming up with an effective way to represent the multiple piece and tile captures that can occur from a single move.

## Special Thanks

* [Chess Programming Wiki](https://www.chessprogramming.org/)
* [Mediocre Chess](https://mediocrechess.blogspot.com/)
* [Pleco](https://github.com/sfleischman105/Pleco)
* [Little Wing](https://github.com/vinc/littlewing)
* [Blecki's engine](https://github.com/Blecki/coerceo)
* [Amit's guide to hex grids](https://www.redblobgames.com/grids/hexagons/)
* The now-defunct codelympics, for starting off this whole thing.
* And the creators of Coerceo, for making such an interesting game.

## Legal

This game is an unofficial clone. Please support the official game by buying it at [coerceo.com](http://coerceo.com/order.html).

This game is licensed under the AGPLv3. A copy is available in the `LICENSE` file or online at [gnu.org](https://www.gnu.org/licenses/).

This game uses code from the library [imgui-rs](https://github.com/Gekkio/imgui-rs), which is licensed under the MIT License, and the font [Fira Sans](https://github.com/mozilla/Fira), which is licensed under the OFL v1.1. See the `NOTICE` file for more information.

All rights, trademarks, copyrights, concepts, etc. of the game Coerceo belong to the Coerceo Company.
