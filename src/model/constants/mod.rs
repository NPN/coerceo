/*
 * Copyright (C) 2017-2018 Ryan Huang
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]

mod tests;

use model::bitboard::BitBoard;
use model::{Color, ColorMap, FieldCoord};

// 19 hexes * 3 bits per hex = 57 set bits
pub const HEX_STARTING_POSITION: BitBoard = 0x1FF_FFFF_FFFF_FFFF;

#[cfg_attr(rustfmt, rustfmt_skip)]
/// Generate the Laurentius starting position.
pub fn generate_laurentius() -> ColorMap<BitBoard> {
    let mut white = 0;
    let mut black = 0;

    // (0, 0) is the only empty hex.
    // All other hexes have exactly two pieces on them in the starting position.
    let piece_locations = [
        (-2,  2, 0, 4),
        (-2,  1, 0, 3),
        (-2,  0, 3, 5),
        (-1,  2, 1, 4),
        (-1,  1, 0, 4),
        (-1,  0, 3, 5),
        (-1, -1, 2, 5),
        ( 0,  2, 1, 5),
        ( 0,  1, 1, 5),
        ( 0, -1, 2, 4),
        ( 0, -2, 2, 4),
        ( 1,  1, 2, 5),
        ( 1,  0, 0, 2),
        ( 1, -1, 1, 3),
        ( 1, -2, 1, 4),
        ( 2,  0, 0, 2),
        ( 2, -1, 0, 3),
        ( 2, -2, 1, 3),
    ];

    {
        let mut set_field = |coord: FieldCoord| match coord.color() {
            Color::White => white |= coord.to_bitboard(),
            Color::Black => black |= coord.to_bitboard(),
        };
        for &(x, y, f1, f2) in &piece_locations {
            set_field(FieldCoord::new(x, y, f1));
            set_field(FieldCoord::new(x, y, f2));
        }
    }
    ColorMap::new(white, black)
}

pub struct LookupTable<T>(ColorMap<T>);

impl LookupTable<[BitBoard; 57]> {
    pub fn bb_get(&self, bb: BitBoard, color: Color) -> BitBoard {
        self.0.get_ref(color)[bb.trailing_zeros() as usize]
    }
}

impl LookupTable<[BitBoard; 19]> {
    pub fn index_get(&self, index: usize, color: Color) -> BitBoard {
        self.0.get_ref(color)[index]
    }
}

macro_rules! lookup_table {
    ($name:ident, $len:expr, $white:expr, $black:expr) => {
        pub const $name: LookupTable<[BitBoard; $len]> = LookupTable(ColorMap {
            white: $white,
            black: $black,
        });
    };
}

#[cfg_attr(rustfmt, rustfmt_skip)]
lookup_table!(
    EDGE_NEIGHBORS,
    57,
    [
        35, 6, 1029, 280, 48, 8232, 192, 384, 65856, 17920, 3072, 4196864, 143360, 24577, 33574912, 1146880, 196616, 268599296, 786432, 1572928,
        2148794368, 73400320, 12582912, 10485760, 587202560, 100663808, 137522839552, 4697620480, 805310464, 1100182716416, 37580963840, 6442483712,
        8801461731328, 25769803776, 51539869696, 70411693850624, 2405181685760, 412318957568, 343597383680, 19241453486080, 3298551660544,
        565698732490752, 153931627888640, 26388413284352, 4525589859926016, 105553116266496, 211107306274816, 36204718879408128, 9851624184872960,
        1688918579740672, 1407374883553280, 78812993478983680, 13511348637925376, 11258999068426240, 54043195528445952, 108090789103403008,
        90071992547409920
    ],
    [
        8197, 3, 6, 65576, 24, 49, 524608, 192, 392, 33556992, 1540, 3072, 268455936, 12320, 25088, 2147647488, 98560, 200704, 17181179904, 786432,
        1605632, 137449439232, 6293504, 12582912, 1099595513856, 50348032, 102760448, 8796764110848, 402784256, 822083584, 70374112886784, 3222274048,
        6576668672, 42949672960, 25769803776, 52613349376, 563293550804992, 206225539072, 412316860416, 4506348406439936, 1649804312576,
        3367254360064, 36050787251519488, 13198434500608, 26938034880512, 175921860444160, 105587476004864, 215504279044096, 1407374883553280,
        846623953387520, 1688849860263936, 11258999068426240, 6772991627100160, 13792273858822144, 90071992547409920, 54183933016801280,
        110338190870577152
    ]
);

#[cfg_attr(rustfmt, rustfmt_skip)]
lookup_table!(
    VERTEX_NEIGHBORS,
    57,
    [
        8246, 5, 9731, 65968, 41, 77849, 524672, 328, 622792, 33582084, 2564, 39847424, 268656672, 21029, 318779904, 2149253376, 168232, 2550239232,
        17181442048, 1343808, 20401913856, 137552201728, 10487808, 137445244928, 1100417613824, 86002176, 1305722486784, 8803340910592, 688017408,
        10445779894272, 70426727284736, 5504139264, 83566239154176, 51539607552, 44024725504, 105579959812096, 566660872273920, 343674978304,
        563156111851520, 4533286978191360, 2818119303168, 5349742544420864, 36266295825530880, 22544954425344, 42797940355366912, 211140592271360,
        180359635402752, 54153146691223552, 15201847765630976, 1409917504192512, 844424930131968, 121614782125047808, 11560815010250752,
        7036874417766400, 108227128545247232, 92486520082006016, 56294995342131200
    ],
    [
        25638, 37, 1027, 205104, 296, 8219, 1638784, 320, 65752, 104877056, 18949, 4195840, 839016448, 151593, 33568257, 6712131584, 1212744,
        268546056, 53688664064, 1310784, 2148368448, 412396552192, 77597184, 6291456, 3436611371008, 620777984, 137495577088, 27492890968064,
        4966223872, 1099964616704, 219943127744512, 39729790976, 8799716933632, 70420283785216, 42949935104, 70397735469056, 1691461200379904,
        2542706622464, 206160527360, 14094639556460544, 20341652979712, 564805396070400, 112757116451684352, 162733223837696, 4518443168563200,
        36239903251496960, 175965883858944, 36147545348505600, 10696049115004928, 10417391636840448, 844493649608704, 85568392920039424,
        83339133094723584, 7600374127001600, 108086391056891904, 90252312454365184, 60802993016012800
    ]
);

#[cfg_attr(rustfmt, rustfmt_skip)]
lookup_table!(
    HEX_FIELD_NEIGHBORS,
    19,
    [
        8192, 65537, 524296, 33554436, 268436000, 2147488000, 17179901952, 137438955520, 1099513741312, 8796109930496, 70368879443968, 1073741824,
        562950020530176, 4503668883718144, 36029351069745152, 4432406249472, 2199023255552, 299067162755072, 2392537302040576
    ],
    [
        1056, 8448, 65536, 4210688, 33685505, 269484040, 2147483712, 67108864, 137975824896, 1103806599168, 8830452793344, 70368744439808,
        2199025352704, 580542156242944, 4644337249943552, 36028798092705792, 9007267974217728, 72058143793741824, 4398046511104
    ]
);

pub const HEX_MASK: [BitBoard; 19] = [
    0b111,
    0b111000,
    0b111000000,
    0b111000000000,
    0b111000000000000,
    0b111000000000000000,
    0b111000000000000000000,
    0b111000000000000000000000,
    0b111000000000000000000000000,
    0b111000000000000000000000000000,
    0b111000000000000000000000000000000,
    0b111000000000000000000000000000000000,
    0b111000000000000000000000000000000000000,
    0b111000000000000000000000000000000000000000,
    0b111000000000000000000000000000000000000000000,
    0b111000000000000000000000000000000000000000000000,
    0b111000000000000000000000000000000000000000000000000,
    0b111000000000000000000000000000000000000000000000000000,
    0b111000000000000000000000000000000000000000000000000000000,
];

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const REMOVABLE_HEX_COMBS: [BitBoard; 342] = [
    9248, 9216, 8224, 1024, 8192, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 73729, 73984, 8193, 73728, 65792, 1, 8192, 65536, 256, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 589832, 65544, 589824, 8, 65536, 524288, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37765120, 33570820, 37748736, 33570816, 16388, 4194304,
    33554432, 16384, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 268566560, 131105, 545, 33554945, 301990400, 302120960, 268566528, 131104, 33, 513, 33554944,
    301989888, 268435456, 131072, 32, 1, 512, 33554432, 2148532480, 1048840, 4360, 268439560, 2415923200, 2416967680, 2148532224, 1048832, 264, 4104,
    268439552, 2415919104, 2147483648, 1048576, 256, 8, 4096, 268435456, 2147516480, 19327385600, 32832, 2147516416, 19327352832, 64, 32768,
    2147483648, 17179869184, 0, 0, 0, 0, 0, 0, 0, 0, 0, 137506064384, 137506062336, 67110912, 137438953472, 67108864, 2048, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 1100048515072, 536887808, 2114048, 137441051136, 1236952678400, 1237487452160, 1100048498688, 536887296, 16896, 2097664, 137441050624,
    1236950581248, 1099511627776, 536870912, 16384, 512, 2097152, 137438953472, 8800388120576, 4295102464, 16912384, 1099528409088, 9895621427200,
    9899899617280, 8800387989504, 4295098368, 135168, 16781312, 1099528404992, 9895604649984, 8796093022208, 4294967296, 131072, 4096, 16777216,
    1099511627776, 70403104964608, 34360819712, 135299072, 8796227272704, 79164971417600, 79199196938240, 70403103916032, 34360786944, 1081344,
    134250496, 8796227239936, 79164837199872, 70368744177664, 34359738368, 1048576, 32768, 134217728, 8796093022208, 70369818181632, 1074003968,
    70369817919488, 262144, 1073741824, 70368744177664, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 565149043785728, 2199092461568, 565148976676864,
    2199090364416, 69206016, 562949953421312, 2199023255552, 67108864, 2097152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4521192350285824, 17592739692544,
    69273124864, 563018689675264, 5066618300268544, 5084141766836224, 4521191813414912, 17592722915328, 553648128, 68736253952, 563018672898048,
    5066549580791808, 4503599627370496, 17592186044416, 536870912, 16777216, 68719476736, 562949953421312, 36169538802286592, 140741917540352,
    554184998912, 4504149517402112, 40532946402148352, 40673134134689792, 36169534507319296, 140741783322624, 4429185024, 549890031616,
    4504149383184384, 40532396646334464, 36028797018963968, 140737488355328, 4294967296, 134217728, 549755813888, 4503599627370496, 4433479991296,
    36033196139216896, 35433480192, 4399120252928, 36033195065475072, 34359738368, 1073741824, 4398046511104, 36028797018963968, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 9009466997473280, 9009398277996544, 2267742732288, 9007199254740992, 2199023255552, 68719476736, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    72075735979786240, 299616918568960, 72075186223972352, 18141941858304, 282024732524544, 72057594037927936, 17592186044416, 549755813888,
    281474976710656, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2396935348551680, 145135534866432, 2256197860196352, 140737488355328, 4398046511104, 2251799813685248,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];
