# Geological landforms integration plan

## Goal

Give Game2 a reusable landform layer inspired by RimWorld's Geological Landforms, adapted to this game's terrain and settlements. Add larger, more varied maps with distinct inhabited clusters, several castles and towns, and roads that connect them. RimWorld API or mod compatibility is explicitly out of scope.

## Research and design constraints

- RimWorld mods are organized around `About/About.xml`, optional version-specific folders resolved by `LoadFolders.xml`, XML `Defs`, `Patches`, textures, and optionally compiled assemblies. Geological Landforms keeps landforms as a biome-independent layer and uses XML data to describe availability requirements and terrain-generation effects.
- The local clone is at tag `v1.7.13.1`; upstream has published `v1.7.14` as a prerelease. Before importing source data, update/inspect the clone at that revision and record the exact upstream commit used.
- Geological Landforms is licensed CC BY-NC-SA 4.0. Any copied landform definitions, text, art, or code must retain attribution and comply with that license, including share-alike and non-commercial terms. Prefer a small, explicitly attributed data subset and original Rust implementation; do not copy Unity integration code or assets that the game cannot use.
- Game2 currently builds a 49x49 map using `Terrain::generate_country`, then composes it with `TileMap::generate`. `Game::new_seeded` has fixed-position river, wall, market, fields, homes, rivals, roads, and world bounds. A fivefold map in each dimension is 245x245 and requires these systems to move together.

## Proposed integration

Represent landforms as data-driven descriptors over Game2's height field and tile map. Each descriptor should express terrain shaping, placement constraints, tags/identity, and a minimum usable-land fraction. A seeded selector chooses among descriptors eligible for the generated terrain. A validation pass measures the final walkable/buildable land fraction and rejects or retries forms that leave too little usable space. Keep a plain fallback generator so every seed can produce a playable map.

Port only the concepts and a curated subset of landform data useful to this game. Prioritize broad valleys, river basins, coasts, lakes, ridges, and open plains. Filter or redesign forms dominated by steep/blocked mountains, narrow canyons, sinkholes, or other layouts that cannot support enough usable ground. Preserve the source attribution and license with every copied data bundle.

Generate the 245x245 terrain in normalized coordinates, then place the main castle, 2-3 castle sites, and 3-4 town clusters against landform suitability and separation constraints. Connect settlements with traversable roads and bridges where needed. Castle footprints should be selected after terrain and river placement, reserve a clear buildable apron, face a usable road approach, and avoid forcing nearby terrain into a flat rectangle. Use seeded placement and a fallback search for each required site.

## Delivery slices (one PR per slice)

1. **Plan and provenance (this PR):** record the design, upstream version, adaptation limits, and attribution/license requirements.
2. **Landform model:** add the Rust descriptor format, seeded selection, terrain constraints, usable-land measurement, fallback behavior, and a small attributed starter set.
3. **Playable-site filtering:** define and validate usable-land thresholds; exclude or adapt forms whose blocked/steep footprint makes settlement play impractical.
4. **Large-map generation:** make the map 245x245 and replace fixed coordinates with normalized, bounds-aware placement for terrain features, fields, and the main settlement.
5. **Castle and settlement layout:** improve castle placement/generation, add 2-3 castle sites and 3-4 clustered towns, and lay roads/bridges between them with minimum-clearance and reachability constraints.
6. **Integration and polish:** update world bounds, pathing, camera/minimap/UI scaling, serialization if needed, and generation diagnostics; tune performance and visuals for the larger world.

Each implementation PR should include focused deterministic generation checks, representative seeded map evidence, and the relevant `cargo test -p game2-core --lib` / `cargo check` commands from the repository instructions. Submit each PR from `main`, wait for AI review comments, address them, then merge before starting the next slice.

## Open implementation decisions

- Decide whether copied upstream XML definitions are converted into Game2-owned data or retained as source XML with a Rust loader. Keep the initial slice small and avoid importing definitions that require RimWorld-only features.
- Measure current playable-land and path connectivity distributions before fixing numerical cutoffs; choose thresholds from observed map behavior rather than guessing.
- Confirm how several castles and towns map onto existing simulation entities and political/economic systems before changing those systems. The initial map-generation slice must keep existing gameplay roles intact.
- Profile memory, generation time, renderer traversal, and pathfinding at 245x245. If needed, stage the dimension increase separately from settlement-count increases while keeping PRs independently reviewable.

## Acceptance criteria

- A fixed seed produces the same landform choice, terrain, sites, and connections.
- Generated maps always have enough usable land and a connected route to all required settlements, with a deterministic fallback if placement fails.
- The main castle's footprint, gates, approaches, and surrounding roads are placed from local terrain and remain playable across representative landforms.
- The requested 245x245 map has space for 2-3 castles and 3-4 town clusters connected by roads.
- Imported material is narrowly scoped, attributed, and tracked with its license; RimWorld-specific assemblies, Harmony patches, and UI are excluded.
