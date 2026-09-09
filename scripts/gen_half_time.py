#!/usr/bin/env python3
"""Future Bass / Kawaii Future Bass bundled songs (strudel-genre-* v8 / v1).

01 is the skill fence. 02–30 rewrite kick, bass degrees, lead refrain, and
timbre so they are not a key-only copy of the fence.
"""
from __future__ import annotations

import re

PC_NAMES = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"]
SCALE_TOKEN = re.compile(r"([A-G](?:#|b)?)(\d):([A-Za-z]+)")
TITLE_RE = re.compile(r"(// @title )[^\n]+")
KEYS = [0, 2, 3, 4, 5, 7, 8, 9, 10, 11]

# Parent-key C offsets. Future Bass fence is already Am (semis=0 keeps A).
ANTHEM = [(9, "minor"), (5, "lydian"), (0, "major"), (7, "mixolydian")]
ANTHEM_LIFT = [(0, "major"), (7, "mixolydian"), (9, "minor"), (5, "lydian")]
ANTHEM_FALL = [(9, "minor"), (7, "mixolydian"), (5, "lydian"), (4, "phrygian")]
ODO = [(5, "lydian"), (7, "mixolydian"), (4, "phrygian"), (9, "minor")]
ODO_REV = [(9, "minor"), (4, "phrygian"), (7, "mixolydian"), (5, "lydian")]
CLICHE = [(0, "major"), (11, "locrian"), (9, "minor"), (7, "mixolydian")]
KOMURO = [(9, "minor"), (5, "lydian"), (7, "mixolydian"), (0, "major")]
KOMURO_REV = [(0, "major"), (7, "mixolydian"), (5, "lydian"), (9, "minor")]
CANON = [(0, "major"), (7, "mixolydian"), (9, "minor"), (5, "lydian")]
CANON_REV = [(5, "lydian"), (9, "minor"), (7, "mixolydian"), (0, "major")]

KICKS = {
    "skip": "<[bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd] [bd ~ ~ ~]>",
    "sparse": "<[bd ~ ~ ~] [bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ ~]>",
    "bounce": "<[bd ~ [bd ~] ~] [bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd]>",
    "two-step": "<[bd ~ bd ~] [bd ~ ~ ~] [bd ~ bd ~] [bd ~ ~ bd]>",
}
KICK_ORDER = ["skip", "sparse", "bounce", "two-step"]
ROLL = (
    "<~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]] "
    "~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]]>"
)
BOUNCE_OH = "<~ ~ ~ oh ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~>"

KITS = [
    {"bd": "bd", "sd": "sd", "hh": "hh"},
    {"bd": "bd:8t", "sd": "sd:tr", "hh": "hh:ch"},
    {"bd": "bd:8d", "sd": "sd:tr", "hh": "hh:tt"},
    {"bd": "bd:8t", "sd": "sd:tr", "hh": "hh:hs"},
    {"bd": "bd:8d", "sd": "sd", "hh": "hh:ch"},
    {"bd": "bd:8t", "sd": "sd:tr", "hh": "hh:tt"},
]

# Each child is eight 8ths (weight 8). Octave 7 is Future Bass flash.
BASS_FB = [
    "<[0 0 ~ 0  0 ~ 7 4] [0 0 0 ~  0 4 ~ 7] [0 ~ 0 0  4 0 7 ~] [0 0 ~ 4  0 ~ 2 7] [0 0 0 0  ~ 0 4 7] [0 ~ 0 4  0 7 ~ 2] [0 0 ~ 0  4 ~ 7 0] [0 4 0 ~  7 0 2 0]>",
    "<[0 0 7 0  4 ~ 0 7] [0 ~ 0 4  7 0 ~ 4] [0 4 ~ 0  7 ~ 0 4] [0 0 4 7  ~ 0 2 0] [0 7 0 ~  4 0 7 4] [0 0 ~ 7  4 0 0 2] [0 ~ 4 0  7 4 ~ 0] [0 4 7 0  2 ~ 0 7]>",
    "<[0 ~ 0 7  0 4 0 ~] [0 4 0 0  ~ 7 4 0] [0 0 7 ~  0 4 7 2] [0 7 4 0  ~ 0 4 7] [0 0 ~ 0  7 4 0 2] [0 4 7 ~  0 0 4 7] [0 ~ 7 0  4 0 ~ 7] [0 0 4 0  7 ~ 2 0]>",
    "<[0 4 0 7  0 ~ 4 0] [0 0 7 4  0 2 ~ 0] [0 7 ~ 0  4 0 7 4] [0 0 0 7  4 ~ 0 2] [0 4 ~ 7  0 0 4 7] [0 7 0 4  ~ 2 0 0] [0 0 4 ~  7 0 4 0] [0 ~ 0 7  4 2 0 7]>",
    "<[0 0 0 7  ~ 4 0 0] [0 7 4 ~  0 0 7 4] [0 ~ 0 4  7 0 4 ~] [0 4 0 7  0 2 7 0] [0 0 7 0  4 ~ 0 7] [0 4 ~ 0  7 4 0 2] [0 7 0 0  ~ 4 7 0] [0 0 4 7  2 0 ~ 4]>",
    "<[0 7 0 0  4 0 ~ 7] [0 0 4 7  0 ~ 2 0] [0 4 0 ~  7 0 4 0] [0 ~ 7 4  0 0 7 2] [0 0 0 4  7 ~ 0 4] [0 7 ~ 4  0 4 0 7] [0 4 7 0  ~ 0 2 7] [0 0 ~ 7  4 0 0 4]>",
    "<[0 0 4 0  7 0 4 ~] [0 7 ~ 0  4 7 0 0] [0 0 7 4  ~ 0 4 7] [0 4 0 0  7 2 ~ 0] [0 ~ 0 7  4 0 0 4] [0 4 7 0  0 ~ 7 4] [0 0 ~ 4  7 0 2 0] [0 7 4 0  ~ 4 0 7]>",
    "<[0 4 7 0  0 7 ~ 4] [0 0 ~ 0  7 4 0 2] [0 7 0 4  ~ 0 7 0] [0 0 4 ~  0 7 4 0] [0 ~ 4 7  0 0 4 7] [0 4 0 7  2 ~ 0 0] [0 7 4 ~  0 4 7 0] [0 0 7 0  4 0 ~ 2]>",
]

BASS_KW = [
    "<[0 0 ~ 0  0 ~ 0 4] [0 0 0 ~  0 4 ~ 0] [0 ~ 0 0  4 0 0 ~] [0 0 ~ 4  0 ~ 2 0] [0 0 0 0  ~ 0 4 0] [0 ~ 0 4  0 0 ~ 2] [0 0 ~ 0  4 ~ 0 0] [0 4 0 ~  0 0 2 0]>",
    "<[0 0 0 4  0 ~ 0 0] [0 ~ 4 0  0 0 4 ~] [0 4 0 0  ~ 0 2 0] [0 0 ~ 0  4 0 0 2] [0 0 4 ~  0 0 0 4] [0 4 0 0  2 ~ 0 0] [0 ~ 0 4  0 4 ~ 0] [0 0 0 2  0 4 0 ~]>",
    "<[0 ~ 0 0  4 0 ~ 0] [0 0 4 0  0 ~ 4 0] [0 4 ~ 0  0 0 2 ~] [0 0 0 4  ~ 2 0 0] [0 4 0 ~  0 0 4 0] [0 0 ~ 4  2 0 0 ~] [0 ~ 4 0  0 4 0 2] [0 0 2 0  4 ~ 0 0]>",
    "<[0 4 0 0  ~ 0 4 0] [0 0 ~ 4  0 2 0 0] [0 0 4 0  2 ~ 0 4] [0 ~ 0 0  4 0 2 0] [0 0 0 ~  4 0 0 4] [0 4 ~ 0  0 2 ~ 0] [0 0 4 2  0 ~ 0 0] [0 2 0 4  ~ 0 0 2]>",
    "<[0 0 4 ~  0 0 0 4] [0 4 0 0  ~ 2 0 0] [0 ~ 0 4  0 0 4 2] [0 0 2 ~  0 4 0 0] [0 4 0 4  0 ~ 0 0] [0 0 ~ 0  4 2 0 4] [0 2 0 0  4 ~ 0 2] [0 0 4 0  ~ 0 2 0]>",
    "<[0 4 ~ 0  0 4 0 ~] [0 0 0 4  2 0 ~ 0] [0 0 4 0  ~ 0 2 4] [0 2 ~ 0  0 4 0 0] [0 ~ 0 4  0 0 ~ 4] [0 4 0 2  0 0 4 ~] [0 0 2 0  ~ 4 0 0] [0 4 0 ~  2 0 0 4]>",
    "<[0 0 0 0  4 ~ 0 4] [0 4 0 ~  0 0 2 0] [0 ~ 4 0  0 2 0 4] [0 0 2 4  0 ~ 0 0] [0 4 0 0  2 0 ~ 4] [0 0 ~ 2  0 4 0 0] [0 2 4 0  ~ 0 0 4] [0 0 4 2  0 0 ~ 0]>",
    "<[0 ~ 4 0  0 0 4 0] [0 0 0 ~  4 0 2 0] [0 4 0 2  ~ 0 0 4] [0 0 2 0  4 0 ~ 0] [0 4 ~ 0  0 4 2 0] [0 0 4 0  ~ 2 0 4] [0 2 0 ~  0 0 4 0] [0 0 ~ 4  0 2 4 0]>",
]

INTROS = [
    ["[~ 4 ~ 7  ~ 9 4 2]", "[~ 7 4 9  7 ~ 4 2]", "[4 ~ 9 7  ~ 4 2 0]", "[~ 4 7 9  4 2 ~ 7]"],
    ["[~ 4 ~ 7  ~ 9 7 4]", "[~ 7 4 9  7 ~ 4 2]", "[4 ~ 9 7  ~ 4 2 0]", "[~ 4 7 9  4 7 ~ 11]"],
    ["[~ 9 ~ 4  ~ 7 9 2]", "[9 ~ 7 4  ~ 2 4 7]", "[~ 7 9 4  2 ~ 7 4]", "[4 ~ 7 9  ~ 4 2 0]"],
    ["[7 ~ 4 ~  9 4 ~ 2]", "[~ 4 7 2  9 ~ 4 7]", "[4 9 ~ 7  ~ 2 4 0]", "[~ 7 4 9  2 0 ~ 4]"],
    ["[~ 2 4 7  ~ 9 4 2]", "[4 ~ 7 9  ~ 4 2 7]", "[~ 9 4 2  7 ~ 4 0]", "[7 4 ~ 9  4 2 ~ 7]"],
    ["[4 ~ ~ 7  9 ~ 4 2]", "[~ 7 ~ 9  4 7 ~ 2]", "[9 4 ~ 7  ~ 2 0 4]", "[~ 4 9 7  2 ~ 4 0]"],
    ["[~ 7 ~ 11  ~ 9 7 4]", "[11 ~ 7 4  9 ~ 4 2]", "[~ 4 7 11  7 4 ~ 0]", "[7 ~ 9 4  ~ 2 7 4]"],
    ["[~ 4 2 ~  7 9 4 2]", "[2 4 ~ 7  9 ~ 4 0]", "[~ 7 2 4  9 4 ~ 2]", "[4 7 9 ~  2 4 ~ 7]"],
]

BRIDGES = [
    ["[~ 9 7 4  2 0 ~ 4]", "[9 ~ 7 4  ~ 2 0 4]", "[7 4 ~ 2  0 ~ 4 7]", "[~ 4 2 0  4 7 ~ 9]"],
    ["[9 7 ~ 4  2 ~ 0 4]", "[~ 7 4 2  0 4 ~ 7]", "[4 2 0 ~  7 4 2 0]", "[~ 2 4 7  0 4 ~ 2]"],
    ["[~ 11 7 4  9 2 ~ 4]", "[11 4 ~ 7  ~ 2 0 4]", "[7 ~ 4 2  0 4 7 ~]", "[4 2 ~ 0  7 ~ 4 9]"],
    ["[7 9 4 ~  2 0 4 7]", "[~ 4 9 2  0 ~ 7 4]", "[9 ~ 2 4  7 0 ~ 2]", "[4 0 7 ~  2 4 0 9]"],
    ["[~ 4 9 7  11 4 ~ 2]", "[4 11 ~ 7  9 2 0 ~]", "[~ 7 11 4  2 0 4 7]", "[11 7 4 2  ~ 0 4 9]"],
    ["[2 4 7 9  ~ 4 2 0]", "[~ 9 4 2  7 0 ~ 4]", "[4 2 ~ 9  0 4 7 2]", "[~ 0 4 2  7 9 4 ~]"],
    ["[9@2 4 7@2 2 0 4]", "[~ 7 9 2  4 0 ~ 7]", "[4@2 2 0@2 7 4 2]", "[~ 2 0 4  7 4 9 0]"],
    ["[4 7 11 9  7 ~ 4 2]", "[11 ~ 9 4  7 2 0 ~]", "[~ 9 7 11  4 0 2 4]", "[7 4 2 0  ~ 4 7 11]"],
]

REFRAINS = {
    "lift": [
        "[4@2 7 9@2 7 4 2]",
        "[4@2 7 9@2 7 4 0]",
        "[4@2 7 9@2 7 2 ~]",
        "[4@2 7 9@2 7 4 2]",
    ],
    "fall": [
        "[9@2 7 4@2 2 0 4]",
        "[9@2 7 4@2 2 0 2]",
        "[9@2 7 4@2 2 ~ 4]",
        "[9@2 7 4@2 2 0 4]",
    ],
    "skip": [
        "[4 7 ~ 9  7@2 4 2]",
        "[4 7 ~ 9  7@2 4 0]",
        "[4 7 ~ 9  7@2 2 ~]",
        "[4 7 ~ 9  7@2 4 2]",
    ],
    "hold": [
        "[4@4 7@2 9 7]",
        "[4@4 7@2 9 4]",
        "[4@4 7@2 2 7]",
        "[4@4 7@2 9 7]",
    ],
}
MOTIVE_ORDER = ["lift", "fall", "skip", "hold"]

HOOK_FB = [
    "<[~ 7 ~ 11  ~ 9 ~ 7] [~ 11 ~ 9  ~ 7 ~ 4] [7 ~ 11 9  ~ 7 4 2] [~ 9 ~ 7  ~ 4 ~ 11] [7 11 ~ 12  11 7 9 7] [~ 11 ~ 12  9 ~ 7 4] [7 12 ~ 11  9 ~ 12 7] [7 11 12 9  11 7 4 2] [~ 12 ~ 9  ~ 7 ~ 4] [~ 11 ~ 7  ~ 4 ~ 0] [~ 9 ~ 4  ~ 7 ~ 2] [~ 7 ~ 4  ~ 2 ~ 0] [7 11 ~ 12  11 7 9 7] [~ 11 ~ 12  9 ~ 7 4] [7 12 ~ 11  9 ~ 12 7] [7 11 12 9  11 7 4 2]>",
    "<[~ 11 ~ 7  ~ 12 ~ 9] [11 ~ 9 7  ~ 4 ~ 11] [~ 12 7 11  9 ~ 7 4] [7 ~ 11 12  ~ 9 4 2] [11 12 7 9  12 11 7 4] [~ 12 ~ 9  11 7 ~ 4] [12 9 ~ 11  7 12 9 7] [11 7 12 9  7 4 2 0] [~ 9 ~ 12  ~ 4 ~ 7] [12 ~ 7 4  ~ 0 ~ 11] [~ 4 ~ 9  7 ~ 2 0] [9 4 ~ 0  ~ 7 4 2] [11 12 7 9  12 11 7 4] [~ 12 ~ 9  11 7 ~ 4] [12 9 ~ 11  7 12 9 7] [11 7 12 9  7 4 2 0]>",
    "<[7 ~ 9 ~  11 7 ~ 12] [~ 9 11 7  12 ~ 9 4] [9 7 12 ~  11 4 7 9] [~ 11 ~ 12  7 9 ~ 4] [7 9 11 12  9 7 11 7] [12 ~ 11 9  7 ~ 4 2] [~ 7 12 11  9 12 7 4] [9 11 7 12  11 9 4 2] [11 ~ 4 ~  7 9 ~ 0] [~ 12 9 4  2 0 ~ 7] [7 4 ~ 11  ~ 2 0 4] [~ 9 4 2  0 7 ~ 4] [7 9 11 12  9 7 11 7] [12 ~ 11 9  7 ~ 4 2] [~ 7 12 11  9 12 7 4] [9 11 7 12  11 9 4 2]>",
    "<[~ 12 ~ 7  11 ~ 9 4] [12 7 ~ 11  ~ 9 7 2] [~ 7 12 9  11 4 ~ 7] [9 ~ 11 7  12 4 2 0] [12 11 9 7  11 12 7 9] [~ 9 12 11  7 4 9 7] [11 7 ~ 12  9 7 4 2] [12 9 11 7  4 2 0 7] [~ 4 12 9  ~ 7 0 4] [9 ~ 7 2  0 4 ~ 11] [~ 2 9 4  7 0 2 ~] [4 0 ~ 7  2 4 0 9] [12 11 9 7  11 12 7 9] [~ 9 12 11  7 4 9 7] [11 7 ~ 12  9 7 4 2] [12 9 11 7  4 2 0 7]>",
]

HOOK_KW = [
    "<[~ 11 ~ 12  ~ 9 ~ 11] [~ 12 ~ 9  ~ 11 ~ 7] [~ 11 ~ 12  ~ 9 ~ 4] [~ 9 ~ 7  ~ 4 ~ 11] [11 ~ 12 9  ~ 11 12 9] [~ 11 ~ 12  9 ~ 11 7] [11 12 ~ 9  11 ~ 12 9] [11 12 9 11  12 9 11 7] [~ 12 ~ 9  ~ 7 ~ 4] [~ 11 ~ 7  ~ 4 ~ 0] [~ 9 ~ 4  ~ 7 ~ 2] [~ 7 ~ 4  ~ 2 ~ 0] [11 ~ 12 9  ~ 11 12 9] [~ 11 ~ 12  9 ~ 11 7] [11 12 ~ 9  11 ~ 12 9] [11 12 9 11  12 9 11 7]>",
    "<[~ 12 ~ 11  ~ 7 ~ 9] [12 ~ 9 11  ~ 4 ~ 12] [~ 11 9 12  ~ 7 4 9] [9 ~ 12 7  ~ 11 4 0] [12 11 9 12  11 9 7 11] [~ 12 9 11  7 ~ 12 9] [11 ~ 12 7  9 12 11 9] [12 9 11 12  9 7 11 4] [~ 9 ~ 12  ~ 4 ~ 7] [11 ~ 4 0  ~ 7 ~ 9] [~ 7 12 4  0 ~ 2 4] [9 4 ~ 0  2 7 ~ 4] [12 11 9 12  11 9 7 11] [~ 12 9 11  7 ~ 12 9] [11 ~ 12 7  9 12 11 9] [12 9 11 12  9 7 11 4]>",
    "<[11 ~ ~ 12  9 ~ 11 7] [~ 12 11 ~  9 12 ~ 4] [12 9 ~ 11  ~ 7 9 12] [~ 9 11 4  12 7 ~ 9] [11 12 11 9  12 11 9 7] [12 ~ 11 12  9 7 11 9] [~ 11 12 9  11 12 7 12] [9 11 12 9  11 7 4 12] [~ 12 4 ~  9 7 ~ 0] [12 7 ~ 4  ~ 0 11 7] [~ 4 9 0  7 ~ 2 11] [4 ~ 7 2  0 9 ~ 4] [11 12 11 9  12 11 9 7] [12 ~ 11 12  9 7 11 9] [~ 11 12 9  11 12 7 12] [9 11 12 9  11 7 4 12]>",
    "<[~ 9 12 ~  11 9 ~ 12] [9 ~ 11 12  7 ~ 9 11] [~ 12 7 9  11 4 12 ~] [11 9 ~ 7  12 4 ~ 11] [9 12 11 9  ~ 12 11 7] [11 9 12 ~  9 11 7 12] [12 11 ~ 9  12 7 11 9] [11 12 9 11  7 12 9 4] [9 ~ 4 12  ~ 7 0 9] [~ 11 7 0  4 ~ 9 2] [12 4 ~ 7  0 2 4 ~] [~ 7 0 4  2 9 4 0] [9 12 11 9  ~ 12 11 7] [11 9 12 ~  9 11 7 12] [12 11 ~ 9  12 7 11 9] [11 12 9 11  7 12 9 4]>",
]

VOX_FB = [
    "<[~ 0 ~ ~  ~ 4 ~ ~] [~ ~ 0 ~  ~ ~ 4 ~] [0 ~ ~ 4  ~ 0 ~ ~] [~ 4 ~ ~  0 ~ ~ 7] [~ 0 ~ 4  ~ ~ 0 ~] [4 ~ ~ ~  ~ 0 ~ 4] [~ ~ 0 ~  4 ~ ~ ~] [0 ~ 4 ~  ~ ~ 0 ~]>",
    "<[0 ~ ~ ~  ~ ~ 4 ~] [~ 0 ~ ~  4 ~ ~ ~] [~ ~ 0 ~  ~ 4 ~ ~] [4 ~ ~ 0  ~ ~ ~ 7] [0 ~ 4 ~  ~ ~ ~ ~] [~ ~ ~ 0  ~ 4 ~ ~] [~ 0 ~ ~  ~ ~ 4 ~] [0 ~ ~ 4  ~ ~ 0 ~]>",
    "<[~ ~ 0 ~  ~ ~ ~ 4] [~ 0 ~ 4  ~ ~ ~ ~] [0 ~ ~ ~  4 ~ ~ ~] [~ ~ 4 ~  0 ~ ~ 7] [~ 0 ~ ~  ~ 4 ~ ~] [4 ~ 0 ~  ~ ~ ~ ~] [~ ~ ~ 0  ~ ~ 4 ~] [0 ~ ~ ~  ~ 4 ~ 0]>",
    "<[~ 0 ~ 4  ~ ~ ~ ~] [0 ~ ~ ~  ~ ~ 4 ~] [~ ~ 0 4  ~ ~ ~ ~] [~ 4 ~ ~  ~ 0 ~ ~] [0 ~ ~ 4  ~ ~ ~ 7] [~ ~ 0 ~  ~ ~ ~ 4] [4 ~ ~ ~  0 ~ ~ ~] [~ 0 ~ ~  4 ~ ~ 0]>",
]

VOX_KW = [
    "<[~ 0 ~ ~  ~ ~ ~ ~] [0 ~ ~ ~  ~ 4 ~ ~] [~ ~ 0 ~  ~ ~ ~ ~] [~ 4 ~ ~  ~ ~ 0 ~] [0 ~ ~ ~  ~ ~ 4 ~] [~ ~ ~ 0  ~ ~ ~ ~] [~ 0 ~ 4  ~ ~ ~ ~] [4 ~ ~ ~  ~ 0 ~ ~]>",
    "<[0 ~ ~ ~  ~ ~ ~ ~] [~ ~ 0 ~  ~ ~ ~ 4] [~ 0 ~ ~  ~ ~ ~ ~] [~ ~ ~ 4  ~ 0 ~ ~] [0 ~ ~ 4  ~ ~ ~ ~] [~ ~ 0 ~  ~ ~ ~ ~] [~ 4 ~ ~  0 ~ ~ ~] [~ ~ ~ 0  ~ 4 ~ ~]>",
    "<[~ ~ 0 ~  ~ ~ ~ ~] [~ 0 ~ ~  ~ ~ 4 ~] [0 ~ ~ ~  ~ ~ ~ ~] [~ ~ 4 ~  ~ ~ 0 ~] [~ 0 ~ ~  ~ ~ ~ 4] [4 ~ ~ ~  ~ ~ 0 ~] [~ ~ ~ 0  ~ ~ ~ ~] [0 ~ 4 ~  ~ ~ ~ ~]>",
    "<[~ 0 ~ 4  ~ ~ ~ ~] [~ ~ ~ 0  ~ ~ ~ ~] [0 ~ ~ ~  ~ 4 ~ ~] [~ ~ 0 ~  ~ ~ ~ 4] [~ 4 ~ ~  ~ ~ ~ ~] [0 ~ ~ ~  ~ ~ 0 ~] [~ ~ 4 ~  ~ 0 ~ ~] [~ 0 ~ ~  ~ ~ 4 ~]>",
]

CHORDS = [
    "<[[0,4,8] ~ ~ [0,4,8]  [0,4,8] ~ [0,4,8] ~] [[0,2,8] ~ [0,2,8] ~  ~ [0,4,8] ~ [0,4,8]] [[0,4,8] ~ ~ [0,4,8]  [0,2,8] ~ [0,2,8] ~] [[0,4,8] ~ [0,4,8] [0,4,8]  ~ [0,2,8] ~ ~]>",
    "<[[0,4,8] ~ [0,4,8] ~  [0,2,8] ~ ~ [0,4,8]] [[0,4,8] [0,4,8] ~ [0,2,8]  ~ [0,4,8] ~ ~] [[0,2,8] ~ ~ [0,4,8]  [0,4,8] ~ [0,2,8] ~] [[0,4,8] ~ [0,2,8] ~  [0,4,8] [0,4,8] ~ ~]>",
    "<[[0,4,8] [0,4,8] ~ ~  ~ [0,2,8] ~ [0,4,8]] [[0,2,8] ~ [0,4,8] [0,4,8]  ~ ~ [0,2,8] ~] [[0,4,8] ~ ~ [0,2,8]  [0,4,8] ~ ~ [0,4,8]] [[0,2,8] [0,4,8] ~ [0,4,8]  ~ [0,2,8] ~ ~]>",
    "<[~ [0,4,8] ~ [0,4,8]  [0,2,8] ~ [0,4,8] ~] [[0,4,8] ~ ~ [0,2,8]  [0,4,8] ~ [0,2,8] [0,4,8]] [~ [0,2,8] [0,4,8] ~  [0,4,8] ~ ~ [0,2,8]] [[0,4,8] ~ [0,4,8] ~  ~ [0,2,8] [0,4,8] ~]>",
]

PAD_GATES = [
    "<0 ~ ~ ~ 0 ~ ~ ~ ~ ~ 0 ~ 0 ~ ~ ~>",
    "<0 ~ ~ 0 ~ ~ ~ ~ 0 ~ ~ ~ ~ 0 ~ ~>",
    "<0 ~ 0 ~ ~ ~ 0 ~ ~ 0 ~ ~ 0 ~ ~ ~>",
    "<~ 0 ~ ~ 0 ~ ~ ~ ~ ~ 0 ~ ~ 0 ~ 0>",
]
STR_GATES = [
    "<~ ~ 0 ~ ~ ~ 0 ~ 0 ~ ~ ~ ~ ~ 0 ~>",
    "<~ 0 ~ ~ ~ ~ 0 ~ ~ ~ 0 ~ ~ ~ ~ 0>",
    "<~ ~ ~ 0 ~ 0 ~ ~ 0 ~ ~ 0 ~ ~ ~ ~>",
    "<0 ~ ~ ~ ~ 0 ~ ~ ~ ~ ~ 0 ~ ~ 0 ~>",
]

FENCE_FB = r'''// @title future-bass-01
// @genre future-bass
setcpm(140/4)
// kick
$: s("<[bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd] [bd ~ ~ ~]>").gain(0.85).duckorbit(2).duckattack(0.05).duckdepth(0.8)
// hats
$: s("~ ~ sd ~, hh*8, <~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]] ~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]]>").gain(0.42)
// bass
$: note("<[0 0 ~ 0  0 ~ 7 4] [0 0 0 ~  0 4 ~ 7] [0 ~ 0 0  4 0 7 ~] [0 0 ~ 4  0 ~ 2 7] [0 0 0 0  ~ 0 4 7] [0 ~ 0 4  0 7 ~ 2] [0 0 ~ 0  4 ~ 7 0] [0 4 0 ~  7 0 2 0]>")
  .scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("bs:sw").gain(0.42).cut(1).orbit(2)
// lead
$: note("<[~ 4 ~ 7  ~ 9 7 4] [~ 7 4 9  7 ~ 4 2] [4 ~ 9 7  ~ 4 2 0] [~ 4 7 9  4 7 ~ 11] [4@2 7 9@2 7 4 2] [4@2 7 9@2 7 4 0] [4@2 7 9@2 7 2 ~] [4@2 7 9@2 7 4 2] [~ 9 7 4  2 0 ~ 4] [9 ~ 7 4  ~ 2 0 4] [7 4 ~ 2  0 ~ 4 7] [~ 4 2 0  4 7 ~ 9] [4@2 7 9@2 7 4 2] [4@2 7 9@2 7 4 0] [4@2 7 9@2 7 2 ~] [4@2 7 9@2 7 4 2]>")
  .scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("ld:ss").gain(0.16).cut(1)
// hook
$: note("<[~ 7 ~ 11  ~ 9 ~ 7] [~ 11 ~ 9  ~ 7 ~ 4] [7 ~ 11 9  ~ 7 4 2] [~ 9 ~ 7  ~ 4 ~ 11] [7 11 ~ 12  11 7 9 7] [~ 11 ~ 12  9 ~ 7 4] [7 12 ~ 11  9 ~ 12 7] [7 11 12 9  11 7 4 2] [~ 12 ~ 9  ~ 7 ~ 4] [~ 11 ~ 7  ~ 4 ~ 0] [~ 9 ~ 4  ~ 7 ~ 2] [~ 7 ~ 4  ~ 2 ~ 0] [7 11 ~ 12  11 7 9 7] [~ 11 ~ 12  9 ~ 7 4] [7 12 ~ 11  9 ~ 12 7] [7 11 12 9  11 7 4 2]>")
  .scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("ld:st").gain(0.18).cut(1)
// arp
$: note("<[~ 0 ~ ~  ~ 4 ~ ~] [~ ~ 0 ~  ~ ~ 4 ~] [0 ~ ~ 4  ~ 0 ~ ~] [~ 4 ~ ~  0 ~ ~ 7] [~ 0 ~ 4  ~ ~ 0 ~] [4 ~ ~ ~  ~ 0 ~ 4] [~ ~ 0 ~  4 ~ ~ ~] [0 ~ 4 ~  ~ ~ 0 ~]>")
  .scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("vc:pa").gain(0.14).cut(1)
// chords
$: note("<[[0,4,8] ~ ~ [0,4,8]  [0,4,8] ~ [0,4,8] ~] [[0,2,8] ~ [0,2,8] ~  ~ [0,4,8] ~ [0,4,8]] [[0,4,8] ~ ~ [0,4,8]  [0,2,8] ~ [0,2,8] ~] [[0,4,8] ~ [0,4,8] [0,4,8]  ~ [0,2,8] ~ ~]>")
  .scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("plk:ss").gain(0.2).orbit(2)
// pad
$: note("<0 ~ ~ ~ 0 ~ ~ ~ ~ ~ 0 ~ 0 ~ ~ ~>").scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("pf:sp").gain(0.14).room(0.25).orbit(2)
// strings
$: note("<~ ~ 0 ~ ~ ~ 0 ~ 0 ~ ~ ~ ~ ~ 0 ~>").scale("<A4:minor F4:lydian C4:major G4:mixolydian A4:minor F4:lydian C4:major G4:mixolydian C4:major G4:mixolydian A4:minor F4:lydian A4:minor G4:mixolydian F4:lydian E4:phrygian>")
  .s("dr:sl").gain(0.12).orbit(2)
'''

FENCE_KW = r'''// @title kawaii-future-bass-01
// @genre kawaii-future-bass
setcpm(140/4)
// kick
$: s("<[bd ~ ~ ~] [bd [~ bd] ~ ~] [bd ~ ~ bd] [bd ~ ~ ~]>").gain(0.85).duckorbit(2).duckattack(0.05).duckdepth(0.8)
// hats
$: s("~ ~ sd ~, hh*8, <~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]] ~ ~ ~ ~ ~ ~ ~ [~ ~ ~ ~ ~ ~ ~ [hh*4]]>").gain(0.42)
// bass
$: note("<[0 0 ~ 0  0 ~ 0 4] [0 0 0 ~  0 4 ~ 0] [0 ~ 0 0  4 0 0 ~] [0 0 ~ 4  0 ~ 2 0] [0 0 0 0  ~ 0 4 0] [0 ~ 0 4  0 0 ~ 2] [0 0 ~ 0  4 ~ 0 0] [0 4 0 ~  0 0 2 0]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("bs:ht").gain(0.4).cut(1).orbit(2)
// lead
$: note("<[~ 4 ~ 7  ~ 9 4 2] [~ 7 4 9  7 ~ 4 2] [4 ~ 9 7  ~ 4 2 0] [~ 4 7 9  4 2 ~ 7] [4@2 7 9@2 7 4 2] [4@2 7 9@2 7 4 0] [4@2 7 9@2 7 2 ~] [4@2 7 9@2 7 4 2] [~ 9 7 4  2 0 ~ 4] [9 ~ 7 4  ~ 2 0 4] [7 4 ~ 2  0 ~ 4 7] [~ 4 2 0  4 7 ~ 9] [4@2 7 9@2 7 4 2] [4@2 7 9@2 7 4 0] [4@2 7 9@2 7 2 ~] [4@2 7 9@2 7 4 2]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("ld:mx").gain(0.16).cut(1)
// hook
$: note("<[~ 11 ~ 12  ~ 9 ~ 11] [~ 12 ~ 9  ~ 11 ~ 7] [~ 11 ~ 12  ~ 9 ~ 4] [~ 9 ~ 7  ~ 4 ~ 11] [11 ~ 12 9  ~ 11 12 9] [~ 11 ~ 12  9 ~ 11 7] [11 12 ~ 9  11 ~ 12 9] [11 12 9 11  12 9 11 7] [~ 12 ~ 9  ~ 7 ~ 4] [~ 11 ~ 7  ~ 4 ~ 0] [~ 9 ~ 4  ~ 7 ~ 2] [~ 7 ~ 4  ~ 2 ~ 0] [11 ~ 12 9  ~ 11 12 9] [~ 11 ~ 12  9 ~ 11 7] [11 12 ~ 9  11 ~ 12 9] [11 12 9 11  12 9 11 7]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("plk:mx").gain(0.18).cut(1)
// arp
$: note("<[~ 0 ~ ~  ~ ~ ~ ~] [0 ~ ~ ~  ~ 4 ~ ~] [~ ~ 0 ~  ~ ~ ~ ~] [~ 4 ~ ~  ~ ~ 0 ~] [0 ~ ~ ~  ~ ~ 4 ~] [~ ~ ~ 0  ~ ~ ~ ~] [~ 0 ~ 4  ~ ~ ~ ~] [4 ~ ~ ~  ~ 0 ~ ~]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("vc:ya").gain(0.14).cut(1)
// chords
$: note("<[[0,4,8] ~ ~ [0,4,8]  [0,4,8] ~ [0,4,8] ~] [[0,2,8] ~ [0,2,8] ~  ~ [0,4,8] ~ [0,4,8]] [[0,4,8] ~ ~ [0,4,8]  [0,2,8] ~ [0,2,8] ~] [[0,4,8] ~ [0,4,8] [0,4,8]  ~ [0,2,8] ~ ~]>")
  .scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("plk:ch").gain(0.2).orbit(2)
// pad
$: note("<0 ~ ~ ~ 0 ~ ~ ~ ~ ~ 0 ~ 0 ~ ~ ~>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("ps:mx").gain(0.16).room(0.35).orbit(2)
// strings
$: note("<~ ~ 0 ~ ~ ~ 0 ~ 0 ~ ~ ~ ~ ~ 0 ~>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor F4:lydian G4:mixolydian E4:phrygian A4:minor A4:minor E4:phrygian G4:mixolydian F4:lydian C4:major B4:locrian A4:minor G4:mixolydian>")
  .s("ld:cr").gain(0.12).room(0.45).orbit(2)
'''

HALF_TIME_FENCES = {
    "future-bass": FENCE_FB,
    "kawaii-future-bass": FENCE_KW,
}

HALF_TIME_PALETTES = {
    "future-bass": {
        "drums": [
            {"bd": "bd:8t"},
            {"sd": "sd:tr"},
            {"hh": "hh:tt"},
            {"bd": "bd:8d", "hh": "hh:hs"},
        ],
        "bass": ["bs:sw", "bs:ht", "bs:8s", "bs:su"],
        "lead": ["ld:ss", "ld:st", "ld:an", "ld:us"],
        "hook": ["ld:st", "ld:an", "plk:ss"],
        "arp": ["vc:pa", "vc:ya", "vc:tu"],
        "chords": ["plk:ss", "ld:st"],
        "pad": ["pf:sp", "pf:ff", "pf:fo", "pf:cs", "pf:hz", "pf:ju"],
        "strings": ["dr:sl", "dr:rw", "ld:an"],
    },
    "kawaii-future-bass": {
        "drums": [
            {"bd": "bd:8t"},
            {"sd": "sd:tr"},
            {"hh": "hh:ch"},
            {"bd": "bd:8d", "hh": "hh:tt"},
        ],
        "bass": ["bs:ht", "bs:8s", "bs:su"],
        "lead": ["ld:mx", "ld:gl", "ld:cy", "square"],
        "hook": ["plk:mx", "plk:ch", "plk:bl", "plk:mb"],
        "arp": ["vc:ya", "vc:na", "vc:pa"],
        "chords": ["plk:ch", "plk:sm"],
        "pad": ["ps:mx", "ps:gb", "pf:ga", "pf:sp"],
        "strings": ["ld:cr", "pf:ca", "pf:hl", "pf:wm"],
    },
}


def pc_name(pc: int) -> str:
    return PC_NAMES[pc % 12]


def note_pc(name: str) -> int:
    letter = name[0]
    base = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}[letter]
    i = 1
    while i < len(name):
        if name[i] == "#":
            base += 1
            i += 1
        elif name[i] == "b":
            base -= 1
            i += 1
        else:
            break
    return base % 12


def transpose_token(name: str, oct_: int, semis: int) -> tuple[str, int]:
    midi = (oct_ + 1) * 12 + note_pc(name) + semis
    new_oct = midi // 12 - 1
    return pc_name(midi), new_oct


def transpose_text(text: str, semis: int) -> str:
    if semis == 0:
        return text

    def repl(m: re.Match[str]) -> str:
        name, oct_s, mode = m.group(1), int(m.group(2)), m.group(3)
        new_name, new_oct = transpose_token(name, oct_s, semis)
        return f"{new_name}{new_oct}:{mode}"

    return SCALE_TOKEN.sub(repl, text)


def set_title(text: str, title: str) -> str:
    if TITLE_RE.search(text):
        return TITLE_RE.sub(rf"\g<1>{title}", text, count=1)
    return f"// @title {title}\n" + text


def replace_atom(inner: str, old: str, new: str) -> str:
    if old == new:
        return inner
    pat = re.compile(rf"(?<![:\w]){re.escape(old)}(?![:\w])")
    return pat.sub(new, inner)


def apply_kit(text: str, kit: dict[str, str]) -> str:
    for old in sorted(kit, key=len, reverse=True):
        text = replace_atom(text, old, kit[old])
    return text


def scale_inner(pairs: list[tuple[int, str]], oct_: int) -> str:
    parts = [f"{pc_name(off)}{oct_}:{mode}" for off, mode in pairs]
    return "<" + " ".join(parts) + ">"


def form_pairs(genre: str, variant: int, n: int) -> list[tuple[int, str]]:
    if genre == "future-bass":
        a, b, c = ANTHEM, ANTHEM_LIFT, ANTHEM_FALL
        if variant == 0:
            blocks = (a, a, b, c)
        elif variant == 1:
            blocks = (a, b, a, c)
        else:
            blocks = (CANON, CANON, CANON_REV, ANTHEM_FALL)
    else:
        if variant == 0:
            blocks = (ODO, ODO, ODO_REV, CLICHE)
        elif variant == 1:
            blocks = (KOMURO, KOMURO, KOMURO_REV, CLICHE)
        else:
            blocks = (CANON, CANON, CANON_REV, CLICHE)
    if n % 7 == 0:
        blocks = (blocks[1], blocks[0], blocks[3], blocks[2])
    out: list[tuple[int, str]] = []
    for blk in blocks:
        out.extend(blk)
    return out


def wrap_note(children: list[str]) -> str:
    return "<" + " ".join(children) + ">"


def lead_pattern(n: int) -> str:
    intro = INTROS[(n - 1) % len(INTROS)]
    motive = MOTIVE_ORDER[(n - 1) % len(MOTIVE_ORDER)]
    refrain = REFRAINS[motive]
    bridge = BRIDGES[(n * 3) % len(BRIDGES)]
    return wrap_note(list(intro) + list(refrain) + list(bridge) + list(refrain))


def hats_for(kick_name: str) -> str:
    if kick_name == "sparse":
        return "~ ~ sd ~, [~ hh]*4"
    if kick_name == "bounce":
        return f"~ ~ sd ~, hh*8, {BOUNCE_OH}, {ROLL}"
    return f"~ ~ sd ~, hh*8, {ROLL}"


def palette_fb(n: int) -> dict[str, str]:
    leads = ["ld:ss", "ld:st", "ld:an", "ld:us"]
    basses = ["bs:sw", "bs:ht", "bs:8s", "bs:su"]
    arps = ["vc:pa", "vc:ya", "vc:tu"]
    pads = ["pf:sp", "pf:ff", "pf:fo", "pf:cs", "pf:hz", "pf:ju"]
    i = n - 1
    lead = leads[i % 4]
    bass = basses[(i // 2) % 4]
    arp = arps[i % 3]
    pad = pads[i % 6]
    strings = ["dr:sl", "dr:rw"][i % 2]
    if lead == "ld:ss":
        hook = ["ld:st", "ld:an", "plk:ss"][i % 3]
    elif lead == "ld:st":
        hook = ["ld:an", "plk:ss"][i % 2]
    elif lead == "ld:an":
        hook = ["ld:st", "plk:ss"][i % 2]
    else:
        hook = ["ld:st", "ld:an", "plk:ss"][(i // 3) % 3]
    chords = "plk:ss" if hook != "plk:ss" else "ld:st"
    if lead == "ld:st" and chords == "ld:st":
        chords = "plk:ss"
        if hook == "plk:ss":
            hook = "ld:an"
            chords = "plk:ss"
    used = {bass, lead, hook, arp, chords, pad, strings}
    if len(used) < 7:
        if strings == lead or strings == hook:
            strings = "dr:rw" if strings != "dr:rw" else "dr:sl"
        if pad in {bass, lead, hook, arp, chords, strings}:
            for cand in pads:
                if cand not in {bass, lead, hook, arp, chords, strings}:
                    pad = cand
                    break
        if arp in {bass, lead, hook, chords, pad, strings}:
            for cand in arps:
                if cand not in {bass, lead, hook, chords, pad, strings}:
                    arp = cand
                    break
    return {
        "bass": bass,
        "lead": lead,
        "hook": hook,
        "arp": arp,
        "chords": chords,
        "pad": pad,
        "strings": strings,
    }


def palette_kw(n: int) -> dict[str, str]:
    leads = ["ld:mx", "ld:gl", "ld:cy", "square"]
    basses = ["bs:ht", "bs:8s", "bs:su"]
    hooks = ["plk:mx", "plk:ch", "plk:bl", "plk:mb"]
    arps = ["vc:ya", "vc:na", "vc:pa"]
    pads = ["ps:mx", "ps:gb", "pf:ga", "pf:sp"]
    strings_opts = ["ld:cr", "pf:ca", "pf:hl", "pf:wm"]
    i = n - 1
    lead = leads[i % 4]
    bass = basses[i % 3]
    hook = hooks[(i // 2) % 4]
    arp = arps[i % len(arps)]
    pad = pads[(i // 3) % 4]
    strings = strings_opts[(i * 3) % 4]
    chords = "plk:sm" if hook == "plk:ch" else "plk:ch"
    used = {bass, lead, hook, arp, chords, pad, strings}
    if pad in used - {pad}:
        for cand in pads:
            if cand not in used - {pad}:
                pad = cand
                break
    if strings in {bass, lead, hook, arp, chords, pad}:
        for cand in strings_opts:
            if cand not in {bass, lead, hook, arp, chords, pad}:
                strings = cand
                break
    return {
        "bass": bass,
        "lead": lead,
        "hook": hook,
        "arp": arp,
        "chords": chords,
        "pad": pad,
        "strings": strings,
    }


def top_weight(token: str) -> int:
    """Weight of one <> child (8ths if the child is a bar of 8ths)."""
    s = token.strip()
    if s.startswith("[") and s.endswith("]"):
        s = s[1:-1].strip()
    if not s:
        return 0
    total = 0
    depth = 0
    buf: list[str] = []
    for ch in s:
        if ch == "[":
            depth += 1
            buf.append(ch)
        elif ch == "]":
            depth -= 1
            buf.append(ch)
        elif ch.isspace() and depth == 0:
            atom = "".join(buf).strip()
            buf = []
            if atom:
                total += atom_weight(atom)
        else:
            buf.append(ch)
    atom = "".join(buf).strip()
    if atom:
        total += atom_weight(atom)
    return total


def atom_weight(atom: str) -> int:
    if atom.startswith("[") and atom.endswith("]"):
        return top_weight(atom)
    m = re.search(r"@(\d+(?:\.\d+)?)$", atom)
    if m:
        return int(float(m.group(1)))
    return 1


def split_angle_children(inner: str) -> list[str]:
    s = inner.strip()
    if s.startswith("<") and s.endswith(">"):
        s = s[1:-1].strip()
    children: list[str] = []
    depth = 0
    buf: list[str] = []
    for ch in s:
        if ch == "[":
            depth += 1
            buf.append(ch)
        elif ch == "]":
            depth -= 1
            buf.append(ch)
        elif ch.isspace() and depth == 0:
            tok = "".join(buf).strip()
            buf = []
            if tok:
                children.append(tok)
        else:
            buf.append(ch)
    tok = "".join(buf).strip()
    if tok:
        children.append(tok)
    return children


def assert_weight8(label: str, pattern: str) -> None:
    kids = split_angle_children(pattern)
    for i, kid in enumerate(kids):
        w = top_weight(kid)
        if w != 8:
            raise SystemExit(f"{label} child {i} weight {w} (want 8): {kid}")


def lead_s_chain(sound: str) -> str:
    if sound == "square":
        return '.s("square").lpf(3200).gain(0.16).cut(1)'
    return f'.s("{sound}").gain(0.16).cut(1)'


def render_variant(genre: str, n: int) -> str:
    idx = n - 1
    kick_name = KICK_ORDER[idx % 4]
    kit = KITS[(idx // 4) % len(KITS)]
    if n > 1 and kick_name == "skip" and kit == KITS[0]:
        kit = KITS[1]
    kick = apply_kit(KICKS[kick_name], kit)
    hats = apply_kit(hats_for(kick_name), kit)
    variant = idx // 10
    pairs = form_pairs(genre, variant, n)
    scale4 = scale_inner(pairs, 4)
    lead = lead_pattern(n)
    chords = CHORDS[idx % len(CHORDS)]
    pad = PAD_GATES[idx % len(PAD_GATES)]
    strings = STR_GATES[idx % len(STR_GATES)]
    if pad == strings:
        strings = STR_GATES[(idx + 1) % len(STR_GATES)]
    if genre == "future-bass":
        pal = palette_fb(n)
        bass = BASS_FB[idx % len(BASS_FB)]
        hook = HOOK_FB[idx % len(HOOK_FB)]
        arp = VOX_FB[idx % len(VOX_FB)]
        bass_gain = "0.42"
        pad_gain, pad_room = "0.14", "0.25"
        str_extra = ""
    else:
        pal = palette_kw(n)
        bass = BASS_KW[idx % len(BASS_KW)]
        hook = HOOK_KW[idx % len(HOOK_KW)]
        arp = VOX_KW[idx % len(VOX_KW)]
        bass_gain = "0.4"
        pad_gain, pad_room = "0.16", "0.35"
        str_extra = ".room(0.45)"
    assert_weight8(f"{genre}/{n} bass", bass)
    assert_weight8(f"{genre}/{n} lead", lead)
    assert_weight8(f"{genre}/{n} hook", hook)
    assert_weight8(f"{genre}/{n} arp", arp)
    sounds = [pal[k] for k in ("bass", "lead", "hook", "arp", "chords", "pad", "strings")]
    if len(set(sounds)) != 7:
        raise SystemExit(f"{genre}/{n} pitched .s() collision: {pal}")
    lead_chain = lead_s_chain(pal["lead"])
    text = f'''// @title {genre}-{n:02d}
// @genre {genre}
setcpm(140/4)
// kick
$: s("{kick}").gain(0.85).duckorbit(2).duckattack(0.05).duckdepth(0.8)
// hats
$: s("{hats}").gain(0.42)
// bass
$: note("{bass}")
  .scale("{scale4}")
  .s("{pal["bass"]}").gain({bass_gain}).cut(1).orbit(2)
// lead
$: note("{lead}")
  .scale("{scale4}")
  {lead_chain}
// hook
$: note("{hook}")
  .scale("{scale4}")
  .s("{pal["hook"]}").gain(0.18).cut(1)
// arp
$: note("{arp}")
  .scale("{scale4}")
  .s("{pal["arp"]}").gain(0.14).cut(1)
// chords
$: note("{chords}")
  .scale("{scale4}")
  .s("{pal["chords"]}").gain(0.2).orbit(2)
// pad
$: note("{pad}").scale("{scale4}")
  .s("{pal["pad"]}").gain({pad_gain}).room({pad_room}).orbit(2)
// strings
$: note("{strings}").scale("{scale4}")
  .s("{pal["strings"]}").gain(0.12){str_extra}.orbit(2)
'''
    return text


def render_half_time(genre: str, n: int) -> str:
    if genre not in HALF_TIME_FENCES:
        raise SystemExit(f"not a half-time genre: {genre}")
    if n == 1:
        text = HALF_TIME_FENCES[genre].strip() + "\n"
        return set_title(text, f"{genre}-01")
    text = render_variant(genre, n)
    semis, _variant = KEYS[(n - 1) % len(KEYS)], (n - 1) // 10
    text = transpose_text(text, semis)
    return set_title(text, f"{genre}-{n:02d}")
