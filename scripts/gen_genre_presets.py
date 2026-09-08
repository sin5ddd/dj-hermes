#!/usr/bin/env python3
"""Rewrite bundled genre presets from strudel-genre-* skill fences.

Holds out catalog families (clockwork / glass-garden / night-market / tide-lantern).
01 keeps e2e titles; 02–30 transpose and apply named skill variants.
Pitched `.s()` cycles the genre timbre palette (not a copy of the fence sounds).
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from gen_half_time import (  # noqa: E402
    HALF_TIME_FENCES,
    HALF_TIME_PALETTES,
    render_half_time,
)

ROOT = Path(__file__).resolve().parents[1]
SONGS = ROOT / "songs"

HOLD = {"clockwork", "glass-garden", "night-market", "tide-lantern"}

# 10 parent keys, three skill variants → 30 slots.
KEYS = [0, 2, 3, 4, 5, 7, 8, 9, 10, 11]
# Acid: not a rising C D Eb walk. Mix of up/down fifths and steps (01 forced to 0).
ACID_KEYS = [
    0,
    7,
    -5,
    5,
    -2,
    3,
    -7,
    10,
    -4,
    8,
    2,
    -3,
    9,
    -8,
    4,
    -6,
    11,
    -1,
    6,
    -9,
    1,
    -10,
    8,
    4,
    -5,
    7,
    -2,
    10,
    3,
    -7,
]
PC_NAMES = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"]

SCALE_TOKEN = re.compile(r"([A-G](?:#|b)?)(\d):([A-Za-z]+)")
TITLE_RE = re.compile(r"(// @title )[^\n]+")

# e2e-locked titles for slot 01.
TITLE_01 = {
    "house": "warehouse-intro",
    "four-on-the-floor": "sine-pulse",
    "dnb": "dnb-01",
    "techno-duck": "pump-core",
    "acid": "saw-303",
    "dnb-reese": "dnb-reese-01",
}

FENCES: dict[str, str] = {}

FENCES["house"] = r'''// @title warehouse-intro
// @genre house
setcpm(124/4)
// drums
$: s("bd:hf*4, [~ cp]*2, [~ hh:hs]*4, <~ ~ ~ [~@3 bd:hf ~@4]>").gain(0.58)
// bass
$: note("0 0 4 <0 2 4 0>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("bs:hf").gain(0.42)
// lead
$: note("~ 7 4 <7 9 4 2>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// hook
$: note("4 ~ 7 4  2 0 ~ -1").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:lp").gain(0.22).cut(1)
// arp
$: note("~ 0 4 7  ~ 4 0 2").scale("<C5:minor C5:minor G5:dorian C5:minor>")
  .s("plk:hd").gain(0.14).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ep:ky").gain(0.22)
// pad
$: note("0").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("pf:ff").gain(0.16).room(0.3).orbit(2)
'''

FENCES["four-on-the-floor"] = r'''// @title sine-pulse
// @genre techno
setcpm(124/4)
// drums
$: s("bd*4, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>").gain(0.62)
// bass
$: note("0 0 2 <4 0 3 0>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sawtooth").lpf(400).gain(0.44)
  .attack(0.001).decay(0.08).sustain(0.2).release(0.05)
// lead
$: note("~ 7 6 <4 9 3 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ld:ss").gain(0.15).cut(1)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:s5").gain(0.18).cut(1)
// arp
$: note("0 3 0 7  3 0 5 ~").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:ac").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:ky").gain(0.22)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.16).room(0.3).orbit(2)
'''

FENCES["techno-duck"] = r'''// @title pump-core
// @genre techno
setcpm(126/4)
// kick
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.04).duckdepth(0.85)
// hats
$: s("[~ hh]*4, <~ ~ ~ hh*8>").gain(0.38)
// bass
$: note("0 0 2 <4 6 2 0>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sine").fm(3).fmh(1.5).lpf(500).gain(0.52)
  .attack(0.005).decay(0.1).sustain(0.3).release(0.08)
  .orbit(2)
// lead
$: note("~ 7 4 <9 7 4 2>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("sine").fm(3).fmh(2).fmatt(0.01).fmdec(0.3).fmsus(0.25)
  .lpf(1800).lpenv(2).gain(0.16)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:s5").gain(0.16).cut(1)
// arp
$: s("<~ perc:tm ~ perc:st>").gain(0.18)
// chords
$: note("[0,2,4] ~ ~ ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:ky").gain(0.18).orbit(2)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").orbit(2).gain(0.22).room(0.25)
'''

FENCES["acid"] = r'''// @title saw-303
// @genre acid techno
setcpm(130/4)
// drums
$: s("bd*4, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>").gain(0.62)
// bass
$: note("0 ~ 0 <0 0 3 0>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sine").lpf(180).gain(0.32)
// hook
$: note("0 0 3 0  7 3 2 0  4 4 3 0  -1 3 0 <2 5>")
  .scale("C2:minor")
  .s("sawtooth")
  .lpf("260 260 720 260  760 260 260 260  260 680 260 260  740 720 260 260")
  .lpq(14)
  .lpenv(3)
  .lpattack(0.001)
  .lpdecay(0.09)
  .lpsustain(0.05)
  .cut(1)
  .gain(0.44)
  .attack(0.001)
  .decay(0.1)
  .sustain(0.12)
  .release(0.04)
// lead
$: note("~ 7 ~ <9 7 4 12>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:lp").gain(0.12).cut(1)
// arp
$: s("<~ perc:mh ~ perc:tm>").gain(0.14)
// chords
$: note("[0,4] ~ ~ [0,4]").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.16)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.25).orbit(2)
'''

FENCES["dnb"] = r'''// @title dnb-01
// @genre drum and bass
setcpm(174/4)
// drums
$: s("[bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]]*2").gain(0.7).lpf(4000)
// bass
$: note("0 3 0 <0 -1 0 3>").scale("C2:minor")
  .s("square").lpf(120).gain(0.42)
  .attack(0.01).decay(0.5).release(0.4)
// bass-mid
$: note("0 3 0 <0 -1 0 3>").scale("C2:minor")
  .s("sawtooth").lpf(1000).gain(0.32)
  .attack(0.01).decay(0.4).release(0.3)
// lead
$: note("~ 7 ~ <9 7 4 11>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ld:ss").gain(0.14).cut(1)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:dt").gain(0.16).cut(1)
// arp
$: s("<~ perc:st ~ perc:tm>").gain(0.16)
// chords
$: note("[0,4] ~ [0,4] ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.16)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.22).orbit(2)
'''

FENCES["dnb-reese"] = r'''// @title dnb-reese-01
// @genre drum and bass
setcpm(174/4)
// drums
$: s("[bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]]*2").gain(0.7).lpf(4000)
// bass
$: note("0 3 0 <0 -1>").scale("C2:minor").s("square").lpf(120).gain(0.42)
// bass-mid
$: note("0 3 0 <0 -1>").scale("C4:minor").s("bs:rm").gain(0.36)
// hook
$: note("~ 4 ~ <7 4>").scale("C4:minor").s("plk:s5").gain(0.2).cut(1)
// lead
$: note("~ 11 7 <12 9 7 4>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ld:ss").gain(0.12).cut(1)
// arp
$: note("~ 0 7 12  7 0 ~ 4").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:dt").gain(0.12).cut(1)
// chords
$: note("[0,4] ~ ~ [0,4]").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.14)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.2).orbit(2)
'''

FENCES.update(HALF_TIME_FENCES)

FENCES["chill-pop"] = r'''// @title chill-pop-01
// @genre chill-pop
setcpm(100/4)
// drums
$: s("bd ~ bd ~, [~ sd]*2, hh*8, <~ ~ ~ [bd sd bd sd]>").gain(0.5)
// bass
$: note("0 ~ 4 0  0 ~ <4 7 2 0>").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("bs:su").gain(0.42)
// lead
$: note("~ 2 4 6  4 2 ~ <0 2 4 6>").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("plk:ps").gain(0.16).cut(1)
// hook
$: note("0 ~ 2 6  ~ 4 2 <6 4 2 0>").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("ep:rs").gain(0.22)
// arp
$: note("0 2 4 6  4 2 0 ~").scale("<F5:lydian E5:phrygian D5:dorian C5:major>")
  .s("plk:ny").gain(0.12).cut(1)
// chords
$: note("[0,2,6] ~ [0,2,6] ~").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("ep:mt").gain(0.24).room(0.25).orbit(2)
// pad
$: note("0").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("pf:ff").gain(0.14).room(0.3).orbit(2)
'''

FENCES["chill"] = r'''// @title chill-01
// @genre chill
setcpm(90/4)
// drums
$: s("bd:hf ~ ~ bd:hf ~ ~ sd ~, [~ hh]*4, <~ ~ ~ oh>").gain(0.42)
// bass
$: note("0 ~ 2 ~ 0 <3 4 2 0>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:hf").gain(0.4)
// lead
$: note("~ 4 ~ 7 ~ <6 9 7 4>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:ps").gain(0.16).cut(1)
// hook
$: note("0@2 4 7@2 ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:am").gain(0.18).cut(1)
// arp
$: note("~ 7 12 7  4 0 ~ 2").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:hp").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:rs").gain(0.22).room(0.3).orbit(2)
// pad
$: note("0").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("pf:ff").gain(0.16).room(0.35).orbit(2)
'''

FENCES["ambient"] = r'''// @title ambient-01
// @genre ambient
setcpm(70/4)
// drums
$: s("bd:lf ~ ~ ~").gain(0.18)
// bass
$: note("0 ~ ~ <0 0 ~ 0>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("bs:su").gain(0.28)
// lead
$: note("~ 7 ~ <9 7 4 11>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:bl").gain(0.1).cut(1)
// hook
$: note("0@2 ~ 4@2 ~").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:am").gain(0.16).cut(1)
// arp
$: s("<~ perc:cm ~ perc:tg>").gain(0.1)
// chords
$: note("[0,2,4] ~ ~ ~").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ep:mt").gain(0.18)
// pad
$: note("0").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("pf:ff").gain(0.24).room(0.5).orbit(1)
'''

FENCES["dubstep"] = r'''// @title dubstep-01
// @genre dubstep
setcpm(140/4)
// drums
$: s("bd ~ ~ ~ bd ~ sd ~, hh*8, <~ ~ ~ [bd sd bd sd]>").gain(0.72)
// bass
$: note("0 0 3 <0 0 3 -1>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sawtooth").lpf(sine.rangex(80, 600)).lpq(6).gain(0.5)
// lead
$: note("~ 7 ~ <10 7 3 7>").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("sawtooth").fm(4).fmh(1).fmdec(0.25).fmsus(0.2)
  .lpf(800).gain(0.16)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:s5").gain(0.18).cut(1)
// arp
$: note("~ ~ 12 ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:dt").gain(0.1).cut(1)
// chords
$: note("[0,4] ~ ~ [0,4]").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.16)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.25).orbit(2)
'''

FENCES["electro"] = r'''// @title electro-01
// @genre electro
setcpm(126/4)
// drums
$: s("bd*4, ~ sd ~ sd, hh*8, <~ ~ ~ [bd sd bd sd]>").gain(0.62)
// bass
$: note("0 ~ 0 <3 0 0 5>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("square").lpf(500).gain(0.46)
  .attack(0.001).decay(0.08).sustain(0.15).release(0.04)
// lead
$: note("~ 7 4 <9 7 12 7>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("ld:pu").gain(0.14).cut(1)
// hook
$: note("12 ~ 7 <12 15 12 7>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// arp
$: note("~ 0 3 7  3 0 ~ 5").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("plk:cv").gain(0.14).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("plk:sp").gain(0.18)
// pad
$: note("0").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("pf:ff").gain(0.14).room(0.25).orbit(2)
'''

FENCES["lofi-hiphop"] = r'''// @title lofi-hiphop-01
// @genre lofi-hiphop
setcpm(84/4)
// drums
$: s("bd:lf ~ ~ sd, [~ hh]*4, <~ ~ ~ [bd:lf sd bd:lf sd]>").gain(0.44)
// bass
$: note("0 ~ 2 <0 0 3 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:su").gain(0.38)
// lead
$: note("~ 4 7 <5 4 0 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:lf").gain(0.18).cut(1)
// hook
$: note("[0,2,4] ~ [0,3,5] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:rs").gain(0.22)
  .attack(0.05).decay(0.3).sustain(0.5).release(0.3)
// arp
$: note("~ 0 4 7  ~ 4 0 2").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:ny").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:mt").gain(0.2).room(0.3).orbit(1)
// pad
$: note("0").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("pf:ff").gain(0.14).room(0.35).orbit(1)
'''

FENCES["minimal"] = r'''// @title minimal-01
// @genre minimal
setcpm(126/4)
// kick
$: s("bd*4").gain(0.7)
// ohh
$: s("[~ oh]*4").gain(0.28)
// bass
$: note("0 ~ 3 ~").scale("<C2:minor C2:minor C2:minor G2:phrygian>")
  .s("sawtooth").lpf(280).lpq(4).gain(0.45)
// chh
$: s("<~ ~ ~ ~ [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 [hh hh ~ hh]*4 ~ ~ [hh hh ~ hh]*4 [hh hh ~ hh]*4>").gain(0.2)
// perc
$: note("<[0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] [0 ~ ~ ~] ~ ~ [0 ~ ~ ~] [0 ~ ~ ~]>")
  .scale("<C5:minor C5:minor C5:minor G5:phrygian>").s("perc:tm").gain(0.1)
// tom
$: note("<~ ~ ~ ~ ~ ~ ~ ~ [~ 0 ~ ~] [~ 0 ~ ~] [~ 0 ~ ~] [~ 0 ~ ~] ~ ~ ~ ~>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>").s("tom:lo").gain(0.16)
// metal
$: note("<~ ~ ~ ~ [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] [~ ~ 7 ~] ~ ~ ~ ~>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>").s("perc:mh").gain(0.08)
// clap
$: s("<~ ~ ~ cp ~ ~ ~ cp ~ ~ ~ cp ~ ~ ~ cp>").gain(0.22)
// synth
$: note("<~ ~ ~ ~ [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] [~ ~ 4 ~] ~ ~ [~ ~ 4 ~] [~ ~ 4 ~]>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("ld:in").lpf(900).gain(0.16).cut(1)
// pluck
$: note("<~ ~ ~ ~ ~ ~ ~ ~ [~ 7 ~ 1] [~ 7 ~ 1] [~ 7 ~ 1] [~ 7 ~ 1] ~ ~ ~ ~>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:nn").gain(0.12).cut(1)
// stab
$: note("<~ ~ ~ ~ [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] [~ [0,4] ~ ~] ~ ~ [~ [0,4] ~ ~] [~ [0,4] ~ ~]>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:s5").gain(0.12).cut(1)
// chords
$: note("<~ ~ ~ ~ ~ ~ ~ ~ [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] [~ [0,2,4] ~ ~] ~ ~>")
  .scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:sf").gain(0.12)
// pad
$: note("<~ ~ ~ ~ ~ ~ ~ ~ 0 ~ ~ ~ ~ ~ ~ ~>").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("dr:pd").orbit(2).cut(1).gain(0.1).room(0.25)
// drone
$: note("<~ ~ ~ ~ 0 ~ ~ ~ 0 ~ ~ ~ ~ ~ ~ ~>").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("dr:mb").orbit(2).cut(1).gain(0.08)
// texture
$: s("<~ ~ ~ ~ ~ ~ fx:ha ~ ~ ~ fx:nh ~ ~ ~ ~ ~>").gain(0.12).cut(1)
// fx
$: s("<~ ~ ~ fx:rd ~ ~ fx:rk ~ fx:cg ~ ~ ~ fx:sd ~ ~ fx:sw>").gain(0.18).cut(1)
'''

FENCES["progressive-house"] = r'''// @title progressive-house-01
// @genre progressive-house
setcpm(128/4)
// drums
$: s("bd*4, [~ cp]*2, hh*8, <~ ~ ~ oh>").gain(0.58)
// bass
$: note("0 0 4 <0 2 4 0>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:hf").gain(0.42)
// lead
$: note("4@2 7 9@3 ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// hook
$: note("~ 7 4 <9 7 4 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:hb").gain(0.18).cut(1)
// arp
$: note("0 2 4 7  4 2 0 ~").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:hd").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:ky").gain(0.24)
// pad
$: note("0").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("pf:ff").gain(0.18).room(0.4).orbit(2)
'''


# Timbre palettes from strudel-genre-* (alts first; fence sound last if listed).
# n==1 uses index 0 unless the slot is locked (e2e / genre core).
PALETTES: dict[str, dict[str, list]] = {
    "house": {
        "drums": [
            {"bd:hf": "bd:dc"},
            {"hh:hs": "hh:cl"},
            {"cp": "cp:rm"},
            {"bd:hf": "bd:dc", "hh:hs": "hh:cl"},
            {"cp": "cp:gt"},
            {"hh:hs": "hh:cl", "cp": "cp:rm"},
        ],
        "bass": ["bs:ht", "bs:sw", "bs:hf"],
        "lead": ["ld:hu", "ld:sw", "ld:us", "wt_organ", "ld:ss"],
        "hook": ["plk:hb", "plk:ep", "plk:lp"],
        "arp": ["plk:aj", "plk:hb", "plk:hd"],
        "chords": ["ep:wr", "plk:sm", "ep:ky"],
        "pad": ["pf:ju", "pf:mn", "pf:cs", "pf:fo", "pf:ff"],
    },
    "four-on-the-floor": {
        "drums": [
            {"bd": "bd:tc"},
            {"bd": "bd:9p"},
            {"hh": "hh:dk"},
            {"bd": "bd:tc", "hh": "hh:dk"},
        ],
        "bass": ["square", "bs:ht"],
        "lead": ["ld:pu", "ld:sw", "ld:si"],
        "hook": ["plk:ac", "plk:pk"],
        "arp": ["plk:pk", "plk:ac"],
        "chords": ["plk:sf", "ep:ky"],
        "pad": ["pf:pu", "pf:cs", "pf:or", "pf:ff"],
    },
    "techno-duck": {
        "drums": [
            {"bd": "bd:tc"},
            {"bd": "bd:9p"},
            {"hh": "hh:cl"},
            {"bd": "bd:tc", "hh": "hh:cl"},
        ],
        "bass": ["sawtooth", "bs:ht"],
        "lead": ["ld:pu", "plk:pk", "wt_bright", "ld:sw"],
        "hook": ["plk:pk", "plk:ac"],
        "arp": ["perc:ti", "perc:st", "perc:tm"],
        "chords": ["plk:sf", "ep:ky"],
        "pad": ["pf:pu", "pf:cs", "pf:or", "pf:ff"],
    },
    "acid": {
        "drums": [
            {"bd": "bd:tc"},
            {"bd": "bd:9p"},
            {"hh": "hh:cl"},
            {"bd": "bd:ez", "hh": "hh:ch"},
            {"bd": "bd:hf", "hh": "hh:hs"},
            {"bd": "bd:8t", "hh": "hh:dk"},
            {"bd": "bd:ng", "hh": "hh:ns"},
            {"bd": "bd:mt", "hh": "hh:pd"},
            {"bd": "bd:dc", "hh": "hh:tt"},
            {"bd": "bd:lf", "hh": "hh:dk"},
        ],
        "bass": ["bs:su", "sine"],
        "lead": ["plk:lp", "plk:pk", "ld:fm", "sine", "ld:sf", "plk:fc", "ld:mt"],
        "arp": ["perc:tm", "perc:mh", "perc:st", "perc:tk", "perc:cb", "perc:sk"],
        "chords": ["plk:sf", "ep:mt", "plk:s5", "ep:ky", "plk:an"],
        "pad": ["pf:pu", "dr:pd", "pf:ff", "pf:cs", "dr:fg", "pf:fo"],
    },
    "dnb": {
        "drums": [
            {"bd": "bd:dn", "sd": "sd:dn", "hh": "hh:dn", "oh": "oh:dn"},
            {"bd": "bd:jg", "sd": "sd:jg"},
            {"hh": "hh:dn"},
        ],
        "lead": ["ld:ds", "plk:nn", "ld:dp"],
        "hook": ["plk:nn", "plk:dt"],
        "arp": ["perc:tm", "perc:st"],
        "chords": ["plk:s5", "ep:mt"],
        "pad": ["pf:fo", "dr:rd", "pf:ff"],
    },
    "dnb-reese": {
        "drums": [
            {"bd": "bd:dn", "sd": "sd:dn", "hh": "hh:dn", "oh": "oh:dn"},
            {"bd": "bd:jg", "sd": "sd:jg"},
            {"hh": "hh:dn"},
        ],
        "lead": ["ld:ds", "plk:nn", "ld:dp"],
        "arp": ["plk:nn", "plk:dt"],
        "chords": ["plk:sf", "ep:mt"],
        "pad": ["pf:fo", "dr:rd", "pf:ff"],
    },
    **HALF_TIME_PALETTES,
    "chill-pop": {
        "drums": [
            {"bd": "bd:dc"},
            {"bd": "bd:lf"},
            {"sd": "sd:pp"},
            {"sd": "sd:br"},
            {"bd": "bd:dc", "sd": "sd:br"},
        ],
        "bass": ["bs:ht", "bs:su"],
        "lead": ["ld:ny", "plk:gm", "plk:ps"],
        "hook": ["ep:wr", "ep:rh", "ep:rs"],
        "arp": ["plk:hp", "plk:kt", "plk:ny"],
        "chords": ["ep:ky", "plk:ep", "ep:mt"],
        "pad": ["pf:iv", "pf:ln", "pf:cl", "pf:wa", "pf:ff"],
    },
    "chill": {
        "drums": [
            {"sd": "sd:br"},
            {"bd:hf": "bd:lf"},
        ],
        "lead": ["ld:ny", "plk:am", "plk:ps"],
        "hook": ["ep:rs", "plk:am"],
        "arp": ["plk:kl", "plk:hp"],
        "chords": ["ep:mt", "ep:rs"],
        "pad": ["pf:cl", "pf:ln", "dr:fg", "pf:ff"],
    },
    "ambient": {
        "lead": ["ld:et", "ld:fl", "plk:bl", "ld:si"],
        "hook": ["ld:cr", "plk:am"],
        "arp": ["perc:tg", "perc:cm"],
        "chords": ["ld:fp", "ep:mt"],
        "pad": ["dr:ad", "dr:fg", "pf:cl", "pf:wa", "ps:sh", "dr:uw", "pf:ff"],
    },
    "dubstep": {
        "drums": [
            {"bd": "bd:ng"},
            {"sd": "sd:ng"},
            {"hh": "hh:dk"},
            {"bd": "bd:ng", "sd": "sd:ng", "hh": "hh:dk"},
        ],
        "bass": ["bs:wb"],
        "lead": ["ld:gr", "ld:wb", "ld:dp"],
        "hook": ["plk:nn", "plk:s5"],
        "arp": ["plk:dt"],
        "chords": ["plk:sf", "ep:mt"],
        "pad": ["dr:rd", "pf:fo", "pf:ff"],
    },
    "electro": {
        "drums": [
            {"bd": "bd:ez"},
            {"bd": "bd:9p"},
            {"sd": "sd:rm"},
            {"hh": "hh:ch"},
            {"bd": "bd:ez", "hh": "hh:ch"},
        ],
        "bass": ["bs:dq"],
        "lead": ["ld:pu", "ld:ch", "ld:dp"],
        "hook": ["ld:ss", "ld:st", "ld:us"],
        "arp": ["plk:cv"],
        "chords": ["plk:sf", "plk:s5"],
        "pad": ["pf:pu", "ld:hf", "pf:ff"],
    },
    "lofi-hiphop": {
        "drums": [
            {"sd": "sd:br"},
            {"hh": "hh:dk"},
            {"sd": "sd:br", "hh": "hh:dk"},
        ],
        "lead": ["ld:ny", "plk:lf"],
        "hook": ["ep:wr", "ep:rs"],
        "arp": ["plk:lf", "plk:ny"],
        "chords": ["ep:mt"],
        "pad": ["pf:cl", "dr:th", "pf:ff"],
    },
    "minimal": {
        "drums": [
            {"bd": "bd:tc", "oh": "oh:op", "hh": "hh:dk"},
            {"bd": "bd:hf", "oh": "oh:dn", "hh": "hh:tt"},
            {"bd": "bd:tc", "hh": "hh:cl"},
            {"oh": "oh:op", "hh": "hh:dk"},
            {"bd": "bd:hf", "oh": "oh:dn", "hh": "hh:cl"},
            {"hh": "hh:tt", "oh": "oh:op"},
        ],
        "bass": ["bs:ht", "bs:rd", "sawtooth"],
        "perc": ["perc:tk", "perc:st", "perc:ti", "perc:tm"],
        "tom": ["tom:md", "perc:gl", "tom:lo"],
        "metal": ["perc:fm", "perc:cw", "perc:mh"],
        "clap": [
            {"cp": "cp:dr"},
            {"cp": "perc:rm"},
            {},
        ],
        "synth": ["ld:mt", "ld:nb", "ld:gr", "ld:pu", "ld:in"],
        "pluck": ["plk:gm", "plk:kl", "plk:nn", "plk:s5"],
        "stab": ["plk:sf", "plk:nn", "plk:s5"],
        "chords": ["ep:mt", "plk:sf"],
        "pad": ["dr:fg", "pf:ff", "dr:pd"],
        "drone": ["dr:hr", "dr:md", "dr:id", "dr:mb"],
        "texture": [
            {"fx:ha": "fx:ck", "fx:nh": "fx:wd"},
            {"fx:ha": "fx:wd"},
            {"fx:nh": "fx:ck"},
        ],
        "fx": [
            {
                "fx:rd": "fx:nb",
                "fx:rk": "fx:ry",
                "fx:cg": "fx:mc",
                "fx:sw": "fx:dn",
            },
            {
                "fx:rd": "fx:ck",
                "fx:rk": "fx:rl",
                "fx:cg": "fx:mc",
                "fx:sw": "fx:wh",
            },
            {
                "fx:rd": "fx:nb",
                "fx:rk": "fx:ry",
                "fx:cg": "fx:mc",
                "fx:sw": "fx:dn",
            },
            {},
        ],
    },
    "progressive-house": {
        "drums": [
            {"bd": "bd:hf"},
            {"cp": "cp:rm"},
            {"hh": "hh:hs"},
            {"bd": "bd:hf", "cp": "cp:rm"},
        ],
        "bass": ["bs:ht", "bs:sw", "bs:hf"],
        "lead": ["ld:an", "ld:tg", "ld:us", "ld:ss"],
        "hook": ["plk:tg", "plk:ss", "plk:hb"],
        "arp": ["plk:aj", "plk:hd"],
        "chords": ["ep:wr", "plk:sm", "ep:ky"],
        "pad": ["pf:hz", "pf:wm", "pf:ju", "ld:fp", "pf:ff"],
    },
}

# Slots that stay on the fence sound for every n (genre core).
IDENTITY_ALWAYS = {
    "dnb": {"bass", "bass-mid"},
    "dnb-reese": {"bass", "bass-mid", "hook"},
    "acid": {"hook"},
    "future-bass": set(),
    "kawaii-future-bass": set(),
    "chill": {"bass"},
    "lofi-hiphop": {"bass"},
}

# Extra 01 locks for e2e / empty-bank energy / showcase FM.
LOCKED_01 = {
    "house": {"drums", "hook"},
    "four-on-the-floor": {"drums", "bass"},
    "dnb": {"bass", "bass-mid"},
    "dnb-reese": {"bass", "bass-mid", "hook"},
    "acid": {"hook", "bass", "lead"},
    "techno-duck": {"bass"},
    "electro": {"bass", "hook"},
    "dubstep": {"bass"},
}

# Electro writes pitched PCM two octaves below catalog native (C4 → C2).
PCM_MIN_OCT = {
    "electro": 2,
}

WAVEFORMS = {"sawtooth", "square", "sine", "triangle", "wt_organ", "wt_bright", "wt_sine"}
KIT_ATOMS = {"bd", "sd", "hh", "oh", "cp"}
LONG_PLK = {"plk:fp", "plk:sp"}
PCM_PARTS = {"plk", "ep", "ld", "pf", "ps", "dr", "bs", "perc", "tom"}
STRIP_SYNTH = [
    "fmatt",
    "fmdec",
    "fmsus",
    "fmh",
    "fm",
    "pattack",
    "pdecay",
    "penv",
    "lpattack",
    "lpdecay",
    "lpsustain",
    "lpenv",
    "lpf",
    "lpq",
]
STRIP_ADSR = ["attack", "decay", "sustain", "release"]
PITCHED_SLOTS = {
    "bass",
    "bass-mid",
    "lead",
    "hook",
    "arp",
    "chords",
    "pad",
    "strings",
    "synth",
    "pluck",
    "stab",
    "drone",
    "perc",
    "tom",
    "metal",
}
TRACK_HEADER = re.compile(r"^// ([a-z][a-z0-9-]*)\n", re.M)
DOT_S = re.compile(r'\.s\("([^"]+)"\)')
INDEX_CALL = re.compile(r"`([a-z]{2,4}:[a-z0-9]{1,3})`")

_INDEX_KEYS: set[str] | None = None


def index_keys() -> set[str]:
    global _INDEX_KEYS
    if _INDEX_KEYS is None:
        path = (
            ROOT
            / "docs/profile/dj-hermes/skills/creative/strudel-pcm-catalog/INDEX.md"
        )
        _INDEX_KEYS = set(INDEX_CALL.findall(path.read_text(encoding="utf-8")))
    return _INDEX_KEYS


def is_long(sound: str) -> bool:
    if sound in LONG_PLK:
        return True
    if sound == "pf:ff":
        return False
    return sound.startswith(("ld:", "dr:", "pf:", "ps:"))


def is_pcm_key(sound: str) -> bool:
    if ":" not in sound:
        return False
    return sound.split(":", 1)[0] in PCM_PARTS


def map_tracks(text: str, fn) -> str:
    matches = list(TRACK_HEADER.finditer(text))
    if not matches:
        return text
    parts = [text[: matches[0].start()]]
    for i, m in enumerate(matches):
        end = matches[i + 1].start() if i + 1 < len(matches) else len(text)
        name = m.group(1)
        parts.append(fn(name, text[m.start() : end]))
    return "".join(parts)


def dot_s(chunk: str) -> str | None:
    m = DOT_S.search(chunk)
    return m.group(1) if m else None


def set_dot_s(chunk: str, new: str) -> str:
    if DOT_S.search(chunk):
        return DOT_S.sub(f'.s("{new}")', chunk, count=1)
    return chunk


def replace_atom(inner: str, old: str, new: str) -> str:
    if old == new:
        return inner
    pat = re.compile(rf"(?<![:\w]){re.escape(old)}(?![:\w])")
    return pat.sub(new, inner)


def strip_method(body: str, name: str) -> str:
    needle = f".{name}("
    out: list[str] = []
    i = 0
    while True:
        j = body.find(needle, i)
        if j < 0:
            out.append(body[i:])
            break
        out.append(body[i:j])
        k = j + len(needle)
        depth = 1
        while k < len(body) and depth:
            ch = body[k]
            if ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
            k += 1
        i = k
    return "".join(out)


def strip_methods(body: str, names: list[str]) -> str:
    for name in sorted(names, key=len, reverse=True):
        body = strip_method(body, name)
    body = re.sub(r"\n[ \t]*\n", "\n", body)
    return body


def inject_after_s(body: str, snippet: str) -> str:
    return re.sub(
        r'\.s\("[^"]+"\)', lambda m: m.group(0) + snippet, body, count=1
    )


def replace_method(body: str, name: str, args: str) -> str | None:
    needle = f".{name}("
    j = body.find(needle)
    if j < 0:
        return None
    k = j + len(needle)
    depth = 1
    while k < len(body) and depth:
        if body[k] == "(":
            depth += 1
        elif body[k] == ")":
            depth -= 1
        k += 1
    return body[:j] + f".{name}({args})" + body[k:]


def set_method(body: str, name: str, args: str) -> str:
    replaced = replace_method(body, name, args)
    if replaced is not None:
        return replaced
    return inject_after_s(body, f".{name}({args})")


# n=2.. : bass cutoff / LFO. Not applied to 01 (e2e 303 bed).
ACID_BASS_FILTER = [
    ("120", None),
    ("sine.rangex(70, 200).slow(8)", 3),
    ("280", None),
    ("sine.rangex(90, 260).slow(4)", 2),
    ("150", 4),
    ("sine.range(80, 240).slow(16)", None),
    ("220", None),
    ("sine.rangex(60, 180).slow(6)", 3),
    ("100", None),
]


def apply_acid_color(text: str, n: int) -> str:
    if n == 1:
        return text
    lpf_args, lpq = ACID_BASS_FILTER[(n - 2) % len(ACID_BASS_FILTER)]

    def on_bass(name: str, chunk: str) -> str:
        if name != "bass":
            return chunk
        chunk = set_method(chunk, "lpf", lpf_args)
        if lpq is not None:
            chunk = set_method(chunk, "lpq", str(lpq))
        return chunk

    text = map_tracks(text, on_bass)
    if n % 4 == 0:
        text = re.sub(
            r"\[~ (hh(?::[a-z0-9]+)?)\]\*4",
            r"\1*8",
            text,
            count=1,
        )
    return text


def ensure_cut1(body: str) -> str:
    if ".cut(" in body:
        return body
    if ".gain(" in body:
        return body.replace(".gain(", ".cut(1).gain(", 1)
    return body.rstrip() + ".cut(1)\n"


def octave_swap(body: str, src: int, dst: int) -> str:
    return re.sub(rf"([A-G](?:#|b)?){src}:", rf"\g<1>{dst}:", body)


def bump_scale_oct(body: str, min_oct: int = 4) -> str:
    def repl(m: re.Match[str]) -> str:
        name, oct_s, mode = m.group(1), int(m.group(2)), m.group(3)
        if oct_s < min_oct:
            oct_s = min_oct
        return f"{name}{oct_s}:{mode}"

    return SCALE_TOKEN.sub(repl, body)


def maybe_thin_note(body: str, sound: str, slot: str) -> str:
    if not is_long(sound):
        return body
    m = re.search(r'note\("([^"]*)"\)', body)
    if not m:
        return body
    inner = m.group(1)
    if "<" in inner:
        return body
    if slot == "pad" and inner.strip() == "0":
        new = "<0 ~ ~ ~>"
    elif slot == "chords":
        new = f"<[{inner}] ~ ~ ~>"
    else:
        return body
    return body.replace(f'note("{inner}")', f'note("{new}")', 1)


def apply_drum_map(chunk: str, mapping: dict[str, str]) -> str:
    def repl(m: re.Match[str]) -> str:
        inner = m.group(1)
        for old in sorted(mapping, key=len, reverse=True):
            inner = replace_atom(inner, old, mapping[old])
        return f's("{inner}")'

    return re.sub(r's\("([^"]*)"\)', repl, chunk, count=1)


def set_perc_pair(chunk: str, a: str, b: str) -> str:
    def repl(m: re.Match[str]) -> str:
        inner = m.group(1)
        parts = re.split(r"(perc:[a-z0-9]+)", inner)
        news = [a, b]
        n = 0
        out: list[str] = []
        for p in parts:
            if p.startswith("perc:"):
                out.append(news[n % 2])
                n += 1
            else:
                out.append(p)
        return f'$: s("{"".join(out)}")'

    return re.sub(r'\$: s\("([^"]+)"\)', repl, chunk, count=1)


def is_perc_track(chunk: str) -> bool:
    return "note(" not in chunk and re.search(r"\$: s\(", chunk) is not None


def pick_unique(picks: list[str], n: int, used: set[str]) -> str | None:
    if not picks:
        return None
    start = (n - 1) % len(picks)
    for i in range(len(picks)):
        cand = picks[(start + i) % len(picks)]
        if cand not in used:
            return cand
    return None


def apply_bass_sound(chunk: str, new: str, pcm_min_oct: int = 4) -> str:
    old = dot_s(chunk)
    if old == new:
        return chunk
    chunk = set_dot_s(chunk, new)
    new_pcm = is_pcm_key(new)
    old_pcm = bool(old) and is_pcm_key(old)
    had_fm = ".fm(" in chunk or ".fmh(" in chunk
    if new_pcm:
        chunk = strip_methods(chunk, STRIP_SYNTH + STRIP_ADSR)
        if pcm_min_oct >= 4:
            chunk = octave_swap(chunk, 2, 4)
            chunk = bump_scale_oct(chunk, 4)
        else:
            chunk = bump_scale_oct(chunk, pcm_min_oct)
    elif old_pcm and not new_pcm:
        chunk = octave_swap(chunk, 4, 2)
        if ".lpf(" not in chunk:
            chunk = inject_after_s(chunk, ".lpf(400)")
    elif had_fm and new != "sine":
        chunk = strip_methods(chunk, ["fmatt", "fmdec", "fmsus", "fmh", "fm"])
    return chunk


def apply_acid_fm_lead(chunk: str) -> str:
    """Live 2-op FM lead (sound-design signed-off numbers). Not a second 303."""
    chunk = set_dot_s(chunk, "sine")
    chunk = strip_methods(chunk, STRIP_SYNTH)
    chunk = inject_after_s(
        chunk,
        ".fm(3).fmh(2).fmatt(0.01).fmdec(0.3).fmsus(0.25).lpf(1800).lpenv(2)",
    )
    return ensure_cut1(chunk)


def apply_pitched_sound(
    chunk: str, new: str, slot: str, pcm_min_oct: int = 4
) -> str:
    old = dot_s(chunk)
    if old != new:
        chunk = set_dot_s(chunk, new)
        if is_pcm_key(new) or new.startswith("wt_"):
            chunk = strip_methods(chunk, STRIP_SYNTH)
        if is_pcm_key(new):
            chunk = bump_scale_oct(chunk, pcm_min_oct)
        if new == "square" and ".lpf(" not in chunk:
            chunk = inject_after_s(chunk, ".lpf(3200)")
    if slot != "chords" and (
        is_long(new) or new.startswith(("plk:", "ld:"))
    ):
        chunk = ensure_cut1(chunk)
    if slot in {"pad", "chords", "strings"}:
        thin_slot = "pad" if slot == "strings" else slot
        chunk = maybe_thin_note(chunk, new, thin_slot)
    return chunk


def apply_palette(genre: str, text: str, n: int) -> str:
    pal = PALETTES.get(genre)
    if not pal:
        return text
    if genre == "minimal" and n == 1:
        return text
    locked = set(IDENTITY_ALWAYS.get(genre, set()))
    if n == 1:
        locked |= LOCKED_01.get(genre, set())
    used: set[str] = set()
    pcm_min_oct = PCM_MIN_OCT.get(genre, 4)

    def apply_one(name: str, chunk: str) -> str:
        slot = "drums" if name in {"kick", "hats", "ohh", "chh"} else name
        if name in locked or slot in locked:
            cur = dot_s(chunk)
            if cur:
                used.add(cur)
            return chunk
        picks = pal.get(slot)
        if not picks:
            return chunk
        if isinstance(picks[0], dict):
            mapping = picks[(n - 1) % len(picks)]
            return apply_drum_map(chunk, mapping)
        str_picks = [p for p in picks if isinstance(p, str)]
        if is_perc_track(chunk):
            perc_picks = [p for p in str_picks if p.startswith("perc:")]
            a = pick_unique(perc_picks, n, used)
            if not a:
                return chunk
            used.add(a)
            b = pick_unique(perc_picks, n + 1, used) or a
            used.add(b)
            return set_perc_pair(chunk, a, b)
        if name in {"perc", "tom", "metal"}:
            new = pick_unique(str_picks, n, used)
            if not new:
                return chunk
            used.add(new)
            return apply_pitched_sound(chunk, new, name, pcm_min_oct)
        pitched = [p for p in str_picks if not p.startswith("perc:")]
        new = pick_unique(pitched, n, used)
        if not new:
            return chunk
        used.add(new)
        if name == "bass":
            return apply_bass_sound(chunk, new, pcm_min_oct)
        if genre == "acid" and name == "lead" and new == "sine":
            return apply_acid_fm_lead(chunk)
        return apply_pitched_sound(chunk, new, name, pcm_min_oct)

    return map_tracks(text, apply_one)


def check_palette_keys() -> None:
    keys = index_keys()
    for genre, pal in PALETTES.items():
        for slot, picks in pal.items():
            for item in picks:
                vals: list[str]
                if isinstance(item, dict):
                    vals = list(item.values())
                else:
                    vals = [item]
                for v in vals:
                    if v in WAVEFORMS or v in KIT_ATOMS:
                        continue
                    if v not in keys:
                        raise SystemExit(f"palette {genre}/{slot}: unknown {v}")


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


def scale4(oct_: int, pairs: list[tuple[int, str]]) -> str:
    """pairs: (semitone offset from C, mode) at the given octave."""
    parts = [f"{pc_name(off)}{oct_}:{mode}" for off, mode in pairs]
    return "<" + " ".join(parts) + ">"


ODO = [(5, "lydian"), (7, "mixolydian"), (4, "phrygian"), (9, "minor")]  # IV V iii vi
ODO_REV = [(9, "minor"), (4, "phrygian"), (7, "mixolydian"), (5, "lydian")]  # vi iii V IV
CLICHE = [(0, "major"), (11, "locrian"), (9, "minor"), (7, "mixolydian")]  # I vii vi V
KOMURO = [(9, "minor"), (5, "lydian"), (7, "mixolydian"), (0, "major")]  # vi IV V I
KOMURO_REV = [(0, "major"), (7, "mixolydian"), (5, "lydian"), (9, "minor")]  # I V IV vi
CANON = [(0, "major"), (7, "mixolydian"), (9, "minor"), (5, "lydian")]  # I V vi IV
CANON_REV = [(5, "lydian"), (9, "minor"), (7, "mixolydian"), (0, "major")]  # IV vi V I
CITY = [(5, "lydian"), (4, "phrygian"), (2, "dorian"), (0, "major")]  # IV iii ii I
WEST = [(0, "major"), (9, "mixolydian"), (2, "mixolydian"), (7, "mixolydian")]
TWO5 = [(2, "dorian"), (7, "mixolydian"), (0, "major"), (9, "minor")]


def replace_scale_inners(text: str, pairs: list[tuple[int, str]]) -> str:
    def repl(m: re.Match[str]) -> str:
        inner = m.group(1)
        toks = inner.split()
        if len(toks) != len(pairs):
            return m.group(0)
        octs = []
        for tok in toks:
            mm = SCALE_TOKEN.fullmatch(tok)
            if not mm:
                return m.group(0)
            octs.append(int(mm.group(2)))
        # keep each token's octave, rebuild degrees/modes from pairs
        rebuilt = []
        for oct_, (off, mode) in zip(octs, pairs):
            rebuilt.append(f"{pc_name(off)}{oct_}:{mode}")
        return '.scale("<' + " ".join(rebuilt) + '>")'

    return re.sub(r'\.scale\("<([^"]+)>"\)', repl, text)


def apply_variant(genre: str, text: str, variant: int) -> str:
    if variant == 0:
        return text
    if genre in {"future-bass", "kawaii-future-bass"}:
        return text
    if genre == "chill-pop":
        return replace_scale_inners(text, WEST if variant == 1 else TWO5)
    if genre == "house" and variant == 1:
        return text.replace("[~ hh:hs]*4", "hh:hs*8")
    if genre == "house" and variant == 2:
        return text.replace(
            's("bd:hf*4, [~ cp]*2, [~ hh:hs]*4, <~ ~ ~ [~@3 bd:hf ~@4]>").gain(0.58)',
            's("bd:hf*4, [~ cp]*2, [~ hh:hs]*4, <~ ~ ~ [~@3 bd:hf ~@4]>").gain(0.58).lpf(4000)',
        )
    if genre == "four-on-the-floor" and variant >= 1:
        return text.replace(
            's("bd*4, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>")',
            's("bd*4, [~ hh]*4, <~ ~ ~ [bd sd bd sd]>")',
        )
    if genre == "acid" and variant == 1:
        return text.replace('.s("sawtooth")', '.s("square")', 1)
    if genre == "acid" and variant == 2:
        return text.replace(
            '.lpf("260 260 720 260  760 260 260 260  260 680 260 260  740 720 260 260")',
            '.lpf("[260 260 720 260]*4")',
        )
    if genre == "dnb" and variant == 1:
        return text.replace("hh*4", "hh*8")
    if genre == "dnb" and variant == 2:
        return text.replace(".lpf(4000)", ".lpf(2500)")
    if genre == "dnb-reese" and variant == 1:
        return text.replace("hh*4", "hh*8")
    if genre == "dnb-reese" and variant == 2:
        return text.replace(".lpf(4000)", ".lpf(2500)")
    if genre == "techno-duck" and variant == 1:
        return text.replace("duckdepth(0.85)", "duckdepth(0.95)")
    if genre == "techno-duck" and variant == 2:
        return text.replace("[~ hh]*4", "hh*8")
    if genre == "chill" and variant == 1:
        return text.replace("[~ hh]*4", "[~ hh]*4, perc:sk")
    if genre == "ambient" and variant >= 1:
        return text.replace('s("bd:lf ~ ~ ~")', 's("~ ~ ~ ~")')
    if genre == "lofi-hiphop" and variant == 1:
        return text.replace("[~ hh]*4", "[~ hh]*4, perc:cb")
    if genre == "electro" and variant == 1:
        return text.replace("hh*8", "hh*16")
    if genre == "minimal" and variant == 1:
        return text.replace("[~ oh]*4", "[~ oh ~ ~]*2")
    if genre == "minimal" and variant == 2:
        text = text.replace(
            "~ ~ [hh hh ~ hh]*4 [hh hh ~ hh]*4>",
            "~ ~ ~ ~>",
        )
        text = text.replace("~ ~ [~ ~ 4 ~] [~ ~ 4 ~]>", "~ ~ ~ ~>")
        text = text.replace(
            "~ ~ [~ [0,4] ~ ~] [~ [0,4] ~ ~]>",
            "~ ~ ~ ~>",
        )
        text = text.replace("~ ~ [0 ~ ~ ~] [0 ~ ~ ~]>", "~ ~ ~ ~>")

        def drop(name: str, chunk: str) -> str:
            if name in {"clap", "texture"}:
                return ""
            return chunk

        return map_tracks(text, drop)
    if genre == "progressive-house" and variant == 1:
        return text.replace("hh*8", "hh*16")
    if genre == "dubstep" and variant == 1:
        return text.replace("hh*8", "hh*16")
    if variant == 2 and genre in {
        "house",
        "chill",
        "ambient",
        "lofi-hiphop",
        "electro",
        "progressive-house",
        "dubstep",
        "four-on-the-floor",
        "techno-duck",
    }:
        # Darker kit / lead lpf drop on a pitched track — skip drums-only files.
        return text.replace(".lpf(2800)", ".lpf(1800)").replace(".lpf(3200)", ".lpf(2200)")
    return text


def set_title(text: str, title: str) -> str:
    if TITLE_RE.search(text):
        return TITLE_RE.sub(rf"\g<1>{title}", text, count=1)
    return f"// @title {title}\n" + text


def validate(path: Path, text: str) -> None:
    genre = path.parent.name
    n = len(re.findall(r"^\$:", text, re.M))
    if genre in {"future-bass", "kawaii-future-bass"}:
        min_n, max_n = 9, 9
    elif genre == "minimal":
        min_n, max_n = 14, 16
    else:
        min_n, max_n = 7, 8
    if n < min_n or n > max_n:
        raise SystemExit(f"{path}: expected {min_n}–{max_n} $: tracks, got {n}")
    if "setcpm(" not in text:
        raise SystemExit(f"{path}: missing setcpm")
    for bad in ("stack(", ".cpm(", ".lfo(", ".fast(", "kit:bd"):
        if bad in text:
            raise SystemExit(f"{path}: forbidden {bad}")
    if re.search(r"\bdb\b", text):
        raise SystemExit(f"{path}: sample db is silent")
    keys = index_keys()
    slug_re = re.compile(r"([a-z]{2,4}:[a-z0-9]{1,3})")
    for sound in DOT_S.findall(text):
        if sound in WAVEFORMS or sound in KIT_ATOMS:
            continue
        slugs = slug_re.findall(sound)
        if slugs:
            for slug in slugs:
                if slug not in keys and slug not in WAVEFORMS:
                    raise SystemExit(f"{path}: unknown .s({slug}) in {sound!r}")
            continue
        if ":" in sound and sound not in keys:
            raise SystemExit(f"{path}: unknown .s({sound})")
    pitched: dict[str, str] = {}

    def collect(name: str, chunk: str) -> str:
        if name in PITCHED_SLOTS and "note(" in chunk:
            sound = dot_s(chunk)
            if sound:
                if sound in pitched:
                    raise SystemExit(
                        f"{path}: pitched .s({sound}) on {pitched[sound]} and {name}"
                    )
                pitched[sound] = name
        return chunk

    map_tracks(text, collect)
    if path.parent.name in {"future-bass", "kawaii-future-bass"}:
        validate_half_time(path, text)
    if path.parent.name == "minimal":
        validate_minimal(path, text)


def mini_inner(chunk: str) -> str:
    m = re.search(r'(?:note|s)\("([^"]*)"\)', chunk)
    return m.group(1) if m else ""


def validate_minimal(path: Path, text: str) -> None:
    names = TRACK_HEADER.findall(text)
    for need in (
        "kick",
        "ohh",
        "bass",
        "chh",
        "perc",
        "tom",
        "metal",
        "synth",
        "pluck",
        "stab",
        "chords",
        "pad",
        "drone",
        "fx",
    ):
        if need not in names:
            raise SystemExit(f"{path}: missing //{need}")
    if "drums" in names:
        raise SystemExit(f"{path}: kick/ohh/chh must not be folded into // drums")
    if "hh*16" in text or "hh*8" in text or "[~ cp]*2" in text:
        raise SystemExit(f"{path}: forbidden hat/clap grid")
    keys = index_keys()
    for slug in re.findall(r"([a-z]{2,4}:[a-z0-9]{1,3})", text):
        if slug not in keys:
            raise SystemExit(f"{path}: unknown slug {slug}")

    def check(name: str, chunk: str) -> str:
        inner = mini_inner(chunk)
        if name in {"kick", "ohh"} and "<" in inner:
            raise SystemExit(f"{path}: {name} must stay always-on (no 16-child mute)")
        if name == "bass":
            note_m = re.search(r'note\("([^"]*)"\)', chunk)
            if note_m and "<" in note_m.group(1):
                raise SystemExit(f"{path}: bass must be a 1-bar ostinato, not 16-child mute")
            if ".lpf(" not in chunk:
                raise SystemExit(f"{path}: bass needs .lpf")
        if name == "chh" and not re.search(
            r"hh(?::[a-z0-9]+)? hh(?::[a-z0-9]+)? ~ hh(?::[a-z0-9]+)?",
            inner,
        ):
            raise SystemExit(f"{path}: chh must skip the offbeat &")
        if name == "synth" and ".lpf(" not in chunk:
            raise SystemExit(f"{path}: synth needs .lpf")
        return chunk

    map_tracks(text, check)


MINIMAL_BASS_PAT = ["0 ~ 3 ~", "0 0 ~ 3", "0 ~ ~ 3", "0 3 0 ~"]
MINIMAL_SYNTH_LOOP = ["[~ ~ 4 ~]", "[~ ~ 7 ~]", "[4 ~ ~ ~]", "[~ 4 ~ ~]"]
MINIMAL_PLUCK_LOOP = ["[~ 7 ~ 1]", "[~ 1 ~ 7]", "[7 ~ 1 ~]", "[~ 7 1 ~]"]
MINIMAL_BASS_LPF = [280, 240, 320, 260, 300, 220]
MINIMAL_SYNTH_LPF = [900, 800, 1100, 700, 1000, 1200]


def apply_minimal_post(text: str, n: int) -> str:
    bass_pat = MINIMAL_BASS_PAT[(n - 1) % len(MINIMAL_BASS_PAT)]
    synth_loop = MINIMAL_SYNTH_LOOP[(n - 1) % len(MINIMAL_SYNTH_LOOP)]
    pluck_loop = MINIMAL_PLUCK_LOOP[(n - 1) % len(MINIMAL_PLUCK_LOOP)]
    bass_lpf = MINIMAL_BASS_LPF[(n - 1) % len(MINIMAL_BASS_LPF)]
    synth_lpf = MINIMAL_SYNTH_LPF[(n - 1) % len(MINIMAL_SYNTH_LPF)]

    def on_track(name: str, chunk: str) -> str:
        if n > 1 and name == "bass":
            chunk = chunk.replace('note("0 ~ 3 ~")', f'note("{bass_pat}")', 1)
        if n > 1 and name == "synth":
            chunk = chunk.replace("[~ ~ 4 ~]", synth_loop)
        if n > 1 and name == "pluck":
            chunk = chunk.replace("[~ 7 ~ 1]", pluck_loop)
        if name == "bass":
            if ".lpf(" in chunk:
                chunk = set_method(chunk, "lpf", str(bass_lpf))
            else:
                chunk = inject_after_s(chunk, f".lpf({bass_lpf})")
            sound = dot_s(chunk)
            if sound in WAVEFORMS and ".lpq(" not in chunk:
                chunk = inject_after_s(chunk, ".lpq(4)")
        if name == "synth":
            if ".lpf(" in chunk:
                chunk = set_method(chunk, "lpf", str(synth_lpf))
            else:
                chunk = inject_after_s(chunk, f".lpf({synth_lpf})")
        return chunk

    return map_tracks(text, on_track)


def validate_half_time(path: Path, text: str) -> None:
    genre = path.parent.name
    if "[hh*16]" in text:
        raise SystemExit(f"{path}: 4-bar hh*16 roll")
    if "[0,4,9]" in text:
        raise SystemExit(f"{path}: [0,4,9] is not add9")
    if "bd*4" in text:
        raise SystemExit(f"{path}: four-on-the-floor kick")
    scales = re.findall(r'\.scale\("<([^"]+)>"\)', text)
    if not scales:
        raise SystemExit(f"{path}: missing 16-bar .scale")
    for inner in scales:
        ntok = len(inner.split())
        if ntok != 16:
            raise SystemExit(f"{path}: scale has {ntok} children, want 16")
    n_tracks = len(re.findall(r"^\$:", text, re.M))
    if n_tracks != 9:
        raise SystemExit(f"{path}: want 9 $: tracks, got {n_tracks}")
    if genre == "future-bass":
        banned = (
            "plk:mx",
            "ld:mx",
            "plk:bl",
            "plk:mb",
            "plk:ch",
            "ps:mx",
            "ps:gb",
            "ld:cr",
            "plk:fg",
            "plk:fc",
        )
        label = "kawaii"
    else:
        banned = (
            "ld:ss",
            "plk:ss",
            "dr:sl",
            "ld:ap",
            "ld:st",
            "ld:an",
            "ld:us",
            "bs:sw",
        )
        label = "supersaw"
    for bad in banned:
        if f'.s("{bad}")' in text:
            raise SystemExit(f"{path}: {label} sound {bad}")


def slot_spec(n: int, genre: str = "") -> tuple[int, int]:
    idx = n - 1
    keys = ACID_KEYS if genre == "acid" else KEYS
    return keys[idx % len(keys)], idx // 10


def render(genre: str, n: int) -> str:
    if genre in HALF_TIME_FENCES:
        return render_half_time(genre, n)
    text = FENCES[genre].strip() + "\n"
    semis, variant = slot_spec(n, genre)
    if n == 1:
        variant = 0
        semis = 0
    text = apply_variant(genre, text, variant)
    text = apply_palette(genre, text, n)
    if genre == "acid":
        text = apply_acid_color(text, n)
    if genre == "minimal":
        text = apply_minimal_post(text, n)
    text = transpose_text(text, semis)
    if n == 1 and genre in TITLE_01:
        title = TITLE_01[genre]
    else:
        title = f"{genre}-{n:02d}"
    text = set_title(text, title)
    # four-on-the-floor 01 must not mention sd (e2e).
    if genre == "four-on-the-floor" and n == 1 and "sd" in text:
        raise SystemExit("four-on-the-floor 01 still contains sd")
    return text


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--genre", action="append", default=[])
    args = ap.parse_args()
    genres = args.genre or sorted(FENCES)
    check_palette_keys()
    written = 0
    for genre in genres:
        if genre in HOLD:
            continue
        if genre not in FENCES:
            raise SystemExit(f"unknown genre {genre}")
        dest = SONGS / genre
        dest.mkdir(parents=True, exist_ok=True)
        prev_kick = None
        for n in range(1, 31):
            text = render(genre, n)
            path = dest / f"{n:02d}.strudel"
            validate(path, text)
            if genre in HALF_TIME_FENCES:
                km = re.search(r'// kick\n\$: s\("([^"]+)"\)', text)
                kick = km.group(1) if km else None
                if prev_kick and kick == prev_kick:
                    raise SystemExit(f"{path}: same kick string as previous song")
                prev_kick = kick
            path.write_text(text, encoding="utf-8", newline="\n")
            written += 1
    print(f"wrote {written} songs")


if __name__ == "__main__":
    main()
