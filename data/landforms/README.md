# Geological Landforms data attribution

`geological-landforms.json` includes a small adapted subset of the world-tile profile data from **Geological Landforms** by m00nl1ght. The source profiles are Coast, Valley, and Lake from the RimWorld 1.6 data folder, at upstream release `v1.7.14` (commit `872cf77f4cf1dbcfd28d261f2842a00a7c68b63e`).

Source: <https://github.com/m00nl1ght-dev/GeologicalLandforms>

The adapted data in this file is licensed under [Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International](https://creativecommons.org/licenses/by-nc-sa/4.0/). Changes include converting RimWorld world-tile requirements into Game2's usable-land fractions, adjusting selection weights for this game's map model, and excluding forms that leave too little playable ground. The Rust terrain-shaping implementation is newly written for Game2; RimWorld assemblies, XML node graphs, and art assets are not included.
