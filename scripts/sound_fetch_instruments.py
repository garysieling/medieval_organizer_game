#!/usr/bin/env python3
"""sound_fetch_instruments.py - fetch the M2 instrument sample set (#1499)

Pulls the picked notes for every sampled patch from the CC0 packs the
design names (sound-system.md "Free sample sources"): VCSL and VSCO 2
CE on GitHub, FreePats bagpipe on GitHub. Each file lands in
assets/sound/_incoming/<id>.ogg (the extension lies - ffmpeg sniffs
the container) plus a <id>.json sidecar sound_process.py reads into
the ledger. Then run:

    for f in assets/sound/_incoming/*.ogg; do
      python scripts/sound_process.py "$(basename "$f" .ogg)" --kind instrument --name "$(basename "$f" .ogg)"
    done

or --process does both steps in one go.
"""
import argparse
import json
import os
import subprocess
import sys
import urllib.parse
import urllib.request

ROOT = os.path.join(os.path.dirname(__file__), "..")
INCOMING = os.path.join(ROOT, "assets", "sound", "_incoming")

VCSL = "https://raw.githubusercontent.com/sgossner/VCSL/master/"
VSCO = "https://raw.githubusercontent.com/sgossner/VSCO-2-CE/master/"
FREEPATS = "https://raw.githubusercontent.com/freepats/bagpipe/main/"

LIC = {
    "vcsl": ("Versilian Studios (VCSL)",
             "https://github.com/sgossner/VCSL/blob/master/LICENSE"),
    "vsco": ("Versilian Studios Chamber Orchestra 2 CE",
             "https://github.com/sgossner/VSCO-2-CE/blob/master/LICENSE"),
    "freepats": ("FreePats project",
                 "https://github.com/freepats/bagpipe/blob/main/LICENSE.txt"),
}

# (asset id, source key, repo-relative path). Root notes are picked one
# per ~4th semitone across the instrument's M1 range; instruments.rs
# holds the matching zone table.
FILES = [
    # VCSL Baroque recorders, Sustain set
    ("recorder-sop-c5",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Soprano Recorder/Sustain/SopRecorder_Sus_C5_rr1_Main.wav"),
    ("recorder-sop-fs5", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Soprano Recorder/Sustain/SopRecorder_Sus_F#5_rr1_Main.wav"),
    ("recorder-sop-c6",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Soprano Recorder/Sustain/SopRecorder_Sus_C6_rr1_Main.wav"),
    ("recorder-alt-fs4", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Alto Recorder/Sustain/AltRecorder_Sus_F#4_rr1_Main.wav"),
    ("recorder-alt-as4", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Alto Recorder/Sustain/AltRecorder_Sus_A#4_rr1_Main.wav"),
    ("recorder-alt-d5",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Alto Recorder/Sustain/AltRecorder_Sus_D5_rr1_Main.wav"),
    ("recorder-alt-e5",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Alto Recorder/Sustain/AltRecorder_Sus_E5_rr1_Main.wav"),
    ("recorder-ten-c4",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Tenor Recorder/Sustain/TenRecorder_Sus_C4_rr1_Main.wav"),
    ("recorder-ten-e4",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Tenor Recorder/Sustain/TenRecorder_Sus_E4_rr1_Main.wav"),
    ("recorder-ten-gs4", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Tenor Recorder/Sustain/TenRecorder_Sus_G#4_rr1_Main.wav"),
    ("recorder-ten-c5",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Tenor Recorder/Sustain/TenRecorder_Sus_C5_rr1_Main.wav"),
    ("recorder-bass-fs3","vcsl", "Aerophones/Edge-blown Aerophones/Baroque Bass Recorder/Sustain/BassRecorder_Sus_F#3_rr1_Main.wav"),
    ("recorder-bass-as3","vcsl", "Aerophones/Edge-blown Aerophones/Baroque Bass Recorder/Sustain/BassRecorder_Sus_A#3_rr1_Main.wav"),
    ("recorder-bass-d4", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Bass Recorder/Sustain/BassRecorder_Sus_D4_rr1_Main.wav"),
    ("recorder-bass-e4", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Bass Recorder/Sustain/BassRecorder_Sus_E4_rr1_Main.wav"),
    # VCSL Renaissance organ, 8' rank
    ("organ-c2",  "vcsl", "Aerophones/Edge-blown Aerophones/Renaissance Organ/8'/RenOrgan_8foot_Room_C2_rr1.wav"),
    ("organ-as2", "vcsl", "Aerophones/Edge-blown Aerophones/Renaissance Organ/8'/RenOrgan_8foot_Room_A#2_rr1.wav"),
    ("organ-d3",  "vcsl", "Aerophones/Edge-blown Aerophones/Renaissance Organ/8'/RenOrgan_8foot_Room_D3_rr1.wav"),
    ("organ-as3", "vcsl", "Aerophones/Edge-blown Aerophones/Renaissance Organ/8'/RenOrgan_8foot_Room_A#3_rr1.wav"),
    ("organ-d4",  "vcsl", "Aerophones/Edge-blown Aerophones/Renaissance Organ/8'/RenOrgan_8foot_Room_D4_rr1.wav"),
    ("organ-as4", "vcsl", "Aerophones/Edge-blown Aerophones/Renaissance Organ/8'/RenOrgan_8foot_Room_A#4_rr1.wav"),
    ("organ-d5",  "vcsl", "Aerophones/Edge-blown Aerophones/Renaissance Organ/8'/RenOrgan_8foot_Room_D5_rr1.wav"),
    # VCSL folk harp, normal velocity layer
    ("harp-c3",  "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_C3_v2_RR1.wav"),
    ("harp-fs3", "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_F#3_v2_RR1.wav"),
    ("harp-as3", "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_A#3_v2_RR1.wav"),
    ("harp-d4",  "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_D4_v2_RR1.wav"),
    ("harp-fs4", "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_F#4_v2_RR1.wav"),
    ("harp-as4", "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_A#4_v2_RR1.wav"),
    ("harp-d5",  "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_D5_v2_RR1.wav"),
    ("harp-fs5", "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_F#5_v2_RR1.wav"),
    ("harp-gs5", "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_G#5_v2_RR1.wav"),
    # VCSL psaltery, plucked
    ("psaltery-as3", "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_A#3_Main_Pluck_rr1.wav"),
    ("psaltery-e4",  "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_E4_Main_Pluck_rr2.wav"),
    ("psaltery-as4", "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_A#4_Main_Pluck_rr1.wav"),
    ("psaltery-e5",  "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_E5_Main_Pluck_rr1.wav"),
    ("psaltery-fs5", "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_F#5_Main_Pluck_rr3.wav"),
    # VSCO 2 CE: piccolo (fife), oboe (shawm), solo violin (vielle)
    ("fife-c6", "vsco", "Woodwinds/Piccolo/Sus/piccolo_C6_sustain1.wav"),
    ("fife-g6", "vsco", "Woodwinds/Piccolo/Sus/piccolo_G6_sustain1.wav"),
    ("shawm-as3", "vsco", "Woodwinds/Oboe/Sus/Oboe_Sus_A#3_v1_Main.wav"),
    ("shawm-d4",  "vsco", "Woodwinds/Oboe/Sus/Oboe_Sus_D4_v1_Main.wav"),
    ("shawm-as4", "vsco", "Woodwinds/Oboe/Sus/Oboe_Sus_A#4_v1_Main.wav"),
    ("shawm-d5",  "vsco", "Woodwinds/Oboe/Sus/Oboe_Sus_D5_v1_Main.wav"),
    ("vielle-g3", "vsco", "Strings/Solo Violin/Arco Vib/LLVln_ArcoVib_G3_f.wav"),
    ("vielle-c4", "vsco", "Strings/Solo Violin/Arco Vib/LLVln_ArcoVib_C4_f.wav"),
    ("vielle-e4", "vsco", "Strings/Solo Violin/Arco Vib/LLVln_ArcoVib_E4_f.wav"),
    ("vielle-a4", "vsco", "Strings/Solo Violin/Arco Vib/LLVln_ArcoVib_A4_f.wav"),
    ("vielle-e5", "vsco", "Strings/Solo Violin/Arco Vib/LLVln_ArcoVib_E5_f.wav"),
    ("vielle-a5", "vsco", "Strings/Solo Violin/Arco Vib/LLVln_ArcoVib_A5_f.wav"),
    ("vielle-e6", "vsco", "Strings/Solo Violin/Arco Vib/LLVln_ArcoVib_E6_f.wav"),
    # FreePats bagpipe in G: chanter layer 1 + the two drones
    ("bagpipe-f4",  "freepats", "samples/F4_31.flac"),
    ("bagpipe-as4", "freepats", "samples/small/A#4_31.flac"),
    ("bagpipe-d5",  "freepats", "samples/small/D5_31.flac"),
    ("bagpipe-f5",  "freepats", "samples/small/F5_31.flac"),
    ("bagpipe-g5",  "freepats", "samples/small/G5_31.flac"),
    ("bagpipe-drone-g2", "freepats", "samples/small/drone_G2_1.flac"),
    ("bagpipe-drone-g3", "freepats", "samples/small/drone_G3_3.flac"),
    # one more oboe top to tighten the shawm's last zone
    ("shawm-f5", "vsco", "Woodwinds/Oboe/Sus/Oboe_Sus_F5_v1_Main.wav"),
    # VCSL percussion
    ("frame-drum-1", "vcsl", "Membranophones/Struck Membranophones/Frame Drum/HDrumL_Hit_v2_rr1_Sum.wav"),
    ("frame-drum-2", "vcsl", "Membranophones/Struck Membranophones/Frame Drum/HDrumL_Hit_v3_rr1_Sum.wav"),
    ("tabor-1", "vcsl", "Membranophones/Struck Membranophones/Snare Drum, Rope Tension/Low/RopeSnare_low_sn_Main_vl2_rr1.wav"),
    ("tabor-2", "vcsl", "Membranophones/Struck Membranophones/Snare Drum, Rope Tension/Low/RopeSnare_low_tsn_Main_vl2_rr1.wav"),
    ("timpani-1", "vcsl", "Membranophones/Struck Membranophones/Timpani 1/Hit/Timpani1_Hit_v3_rr3_Sum.wav"),
    ("timpani-2", "vcsl", "Membranophones/Struck Membranophones/Timpani 1/Hit/Timpani3_Hit_v3_rr1_Sum.wav"),
    ("bass-drum-1", "vcsl", "Membranophones/Struck Membranophones/Bass Drum 2/bassdrum_hit_mf1.wav"),
    ("bass-drum-2", "vcsl", "Membranophones/Struck Membranophones/Bass Drum 2/bassdrum_hit_f.wav"),
    ("tambourine-1", "vcsl", "Idiophones/Struck Idiophones/Tambourine 1/Tamb1_Hit_v2_rr1_Mid.wav"),
    ("tambourine-2", "vcsl", "Idiophones/Struck Idiophones/Tambourine 1/Tamb1_Shake_rr1_Mid.wav"),
    ("finger-cymbals-1", "vcsl", "Idiophones/Struck Idiophones/Finger Cymbals/Fing_Cymb.wav"),
    ("hand-bells-c5",  "vcsl", "Idiophones/Struck Idiophones/Hand Chimes/sus_C5_r01_main.wav"),
    ("hand-bells-gs5", "vcsl", "Idiophones/Struck Idiophones/Hand Chimes/sus_G#5_r01_main.wav"),
    ("hand-bells-c6",  "vcsl", "Idiophones/Struck Idiophones/Hand Chimes/sus_C6_r01_main.wav"),
    ("tubular-c4", "vcsl", "Idiophones/Struck Idiophones/Tubular Bells 1/chimes_C4_ff_rr2.wav"),
    ("tubular-d4", "vcsl", "Idiophones/Struck Idiophones/Tubular Bells 1/chimes_D4_ff_rr2.wav"),
    ("tubular-e4", "vcsl", "Idiophones/Struck Idiophones/Tubular Bells 1/chimes_E4_ff_rr2.wav"),
    ("sleigh-1", "vcsl", "Idiophones/Struck Idiophones/Sleigh Bells/Sleighbells_Hit_rr1_Mid.wav"),
    ("sleigh-2", "vcsl", "Idiophones/Struck Idiophones/Sleigh Bells/sleighbell2_hit_loud.wav"),
    # lower notes to fill each range (VCSL names run an octave low vs
    # sounding pitch - C3 is their middle C)
    ("recorder-sop-c4",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Soprano Recorder/Sustain/SopRecorder_Sus_C4_rr1_Main.wav"),
    ("recorder-sop-e4",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Soprano Recorder/Sustain/SopRecorder_Sus_E4_rr1_Main.wav"),
    ("recorder-sop-gs4", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Soprano Recorder/Sustain/SopRecorder_Sus_G#4_rr1_Main.wav"),
    ("recorder-alt-f3",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Alto Recorder/Sustain/AltRecorder_Sus_F3_rr1_Main.wav"),
    ("recorder-alt-as3", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Alto Recorder/Sustain/AltRecorder_Sus_A#3_rr1_Main.wav"),
    ("recorder-alt-d4",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Alto Recorder/Sustain/AltRecorder_Sus_D4_rr1_Main.wav"),
    ("recorder-alt-e4",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Alto Recorder/Sustain/AltRecorder_Sus_E4_rr1_Main.wav"),
    ("recorder-ten-c3",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Tenor Recorder/Sustain/TenRecorder_Sus_C3_rr1_Main.wav"),
    ("recorder-ten-e3",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Tenor Recorder/Sustain/TenRecorder_Sus_E3_rr1_Main.wav"),
    ("recorder-ten-gs3", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Tenor Recorder/Sustain/TenRecorder_Sus_G#3_rr1_Main.wav"),
    ("recorder-ten-as3", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Tenor Recorder/Sustain/TenRecorder_Sus_A#3_rr1_Main.wav"),
    ("recorder-ten-d4",  "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Tenor Recorder/Sustain/TenRecorder_Sus_D4_rr1_Main.wav"),
    ("recorder-bass-f2", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Bass Recorder/SusVib/BassRecorder_SusVib_F2_rr1_Main.wav"),
    ("recorder-bass-as2","vcsl", "Aerophones/Edge-blown Aerophones/Baroque Bass Recorder/Sustain/BassRecorder_Sus_A#2_rr1_Main.wav"),
    ("recorder-bass-d3", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Bass Recorder/Sustain/BassRecorder_Sus_D3_rr1_Main.wav"),
    ("recorder-bass-f3", "vcsl", "Aerophones/Edge-blown Aerophones/Baroque Bass Recorder/SusVib/BassRecorder_SusVib_F3_rr1_Main.wav"),
    ("organ-c1",  "vcsl", "Aerophones/Edge-blown Aerophones/Pipe Organ/Quiet/NT5_Man3Quiet_C1_rr1.wav"),
    ("organ-a1",  "vcsl", "Aerophones/Edge-blown Aerophones/Pipe Organ/Quiet/NT5_Man3Quiet_A1_rr1.wav"),
    ("organ-fs2", "vcsl", "Aerophones/Edge-blown Aerophones/Pipe Organ/Quiet/NT5_Man3Quiet_F#2_rr1.wav"),
    ("organ-ds3", "vcsl", "Aerophones/Edge-blown Aerophones/Pipe Organ/Quiet/NT5_Man3Quiet_D#3_rr1.wav"),
    ("organ-a3",  "vcsl", "Aerophones/Edge-blown Aerophones/Pipe Organ/Quiet/NT5_Man3Quiet_A3_rr1.wav"),
    ("organ-ds4", "vcsl", "Aerophones/Edge-blown Aerophones/Pipe Organ/Quiet/NT5_Man3Quiet_D#4_rr1.wav"),
    ("organ-a4",  "vcsl", "Aerophones/Edge-blown Aerophones/Pipe Organ/Quiet/NT5_Man3Quiet_A4_rr1.wav"),
    ("organ-c5",  "vcsl", "Aerophones/Edge-blown Aerophones/Pipe Organ/Quiet/NT5_Man3Quiet_C5_rr1.wav"),
    ("harp-c2",  "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_C2_v2_RR1.wav"),
    ("harp-e2",  "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_E2_v2_RR1.wav"),
    ("harp-as2", "vcsl", "Chordophones/Composite Chordophones/Folk Harp/EWHarp_Normal_A#2_v2_RR1.wav"),
    ("hand-bells-c4",  "vcsl", "Idiophones/Struck Idiophones/Hand Chimes/sus_C4_r01_main.wav"),
    ("hand-bells-fs4", "vcsl", "Idiophones/Struck Idiophones/Hand Chimes/sus_F#4_r01_main.wav"),
    ("hand-bells-e5",  "vcsl", "Idiophones/Struck Idiophones/Hand Chimes/sus_E5_r01_main.wav"),
    ("hand-bells-as5", "vcsl", "Idiophones/Struck Idiophones/Hand Chimes/sus_A#5_r01_main.wav"),
    ("psaltery-c4",  "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_C4_Main_Pluck_rr1.wav"),
    ("psaltery-d4",  "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_D4_Main_Pluck_rr1.wav"),
    ("psaltery-fs4", "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_F#4_Main_Pluck_rr1.wav"),
    ("psaltery-gs4", "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_G#4_Main_Pluck_rr3.wav"),
    ("psaltery-c5",  "vcsl", "Chordophones/Zithers/Psaltery, Bowed and Plucked/Pluck/BowedPsaltery_C5_Main_Pluck_rr1.wav"),
    # VSCO oboe names run a written octave low too
    ("shawm-as2", "vsco", "Woodwinds/Oboe/Sus/Oboe_Sus_A#2_v1_Main.wav"),
    ("shawm-f3",  "vsco", "Woodwinds/Oboe/Sus/Oboe_Sus_F3_v1_Main.wav"),
    ("shawm-f4",  "vsco", "Woodwinds/Oboe/Sus/Oboe_Sus_F4_v1_Main.wav"),
    # VCSL strumstick, finger-picked, across the three strings
    ("strumstick-d2",  "vcsl", "Chordophones/Composite Chordophones/Strumstick/Finger/Strumstick_Finger_Str1_Main_D2_vl1_rr1.wav"),
    ("strumstick-a2",  "vcsl", "Chordophones/Composite Chordophones/Strumstick/Finger/Strumstick_Finger_Str2_Main_A2_vl1_rr1.wav"),
    ("strumstick-e3",  "vcsl", "Chordophones/Composite Chordophones/Strumstick/Finger/Strumstick_Finger_Str3_Main_E3_vl1_rr1.wav"),
    ("strumstick-a3",  "vcsl", "Chordophones/Composite Chordophones/Strumstick/Finger/Strumstick_Finger_Str3_Main_A3_vl1_rr1.wav"),
    ("strumstick-e4",  "vcsl", "Chordophones/Composite Chordophones/Strumstick/Finger/Strumstick_Finger_Str3_Main_E4_vl1_rr1.wav"),
    ("strumstick-g4",  "vcsl", "Chordophones/Composite Chordophones/Strumstick/Finger/Strumstick_Finger_Str3_Main_G4_vl1_rr1.wav"),
]

BASE = {"vcsl": VCSL, "vsco": VSCO, "freepats": FREEPATS}


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--process", action="store_true",
                    help="run sound_process.py on each file after downloading")
    args = ap.parse_args()

    os.makedirs(INCOMING, exist_ok=True)
    proc = os.path.join(ROOT, "scripts", "sound_process.py")
    for aid, src, rel in FILES:
        url = BASE[src] + urllib.parse.quote(rel)
        dst = os.path.join(INCOMING, aid + ".ogg")
        if not os.path.isfile(dst):
            print(f"fetch {aid} <- {rel}")
            try:
                with urllib.request.urlopen(url, timeout=60) as r:
                    open(dst, "wb").write(r.read())
            except Exception as e:
                print(f"  FAILED {url}: {e}", file=sys.stderr)
                continue
        author, license_url = LIC[src]
        json.dump({"source_url": url, "author": author, "license": license_url},
                  open(os.path.join(INCOMING, aid + ".json"), "w"), indent=2)
        if args.process:
            r = subprocess.run([sys.executable, proc, aid,
                                "--kind", "instrument", "--name", aid])
            if r.returncode != 0:
                sys.exit(r.returncode)


if __name__ == "__main__":
    main()
