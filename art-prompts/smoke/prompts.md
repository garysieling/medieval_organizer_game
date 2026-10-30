# Retro Diffusion art prompts

21 prompts, est. $2.83 USD at list prices.
Ids are stable: re-run with `--state art-prompts/generated.json` to skip done work.


## P0 - needed now (replaces assets the game already draws) / tiles/ground-spring

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| ground-grass-spring-ee148db8-smoke | single isometric medieval village asset: flat grassy ground block, fresh green spring, small flowers, new leaves, square isometric ground block | rd_pro__isometric | 128x128 | 1 | 0.18 | validates: isometric ground block + tile_x/tile_y seamless flags \| ground block like the Isometric Medieval Pack; tile_x+tile_y set so edges meet |

## P0 - needed now (replaces assets the game already draws) / tiles/wang-spring

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| road-wang-spring-4278a7b3-smoke | dirt road tiles for a medieval village, wheel-rutted earth, fresh green spring, small flowers, new leaves | rd_tile__tileset | 32x32 | 1 | 0.10 | validates: wang tileset mode - the whole road set from one call \| wang tileset: 16px or 32px tiles, all edge combinations in one sheet |

## P0 - needed now (replaces assets the game already draws) / tiles/trees-spring

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| tree-oak-spring-d80b35b2-smoke | single isometric medieval village asset: broad spreading oak tree, fresh green spring, small flowers, new leaves | rd_tile__tile_object | 96x96 | 1 | 0.06 | validates: tile_object prop mode \| num_images=2 gives two variants of the same species |

## P0 - needed now (replaces assets the game already draws) / buildings/shops

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| shop-bakery-1026b2e1-smoke | single isometric medieval village asset: bakery with brick oven chimney and bread sign | rd_pro__isometric | 160x160 | 1 | 0.18 | validates: rd_pro isometric building |

## P0 - needed now (replaces assets the game already draws) / people/roles

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| person-minstrel-1026f9b4-smoke | small chibi medieval character, front view, full body, on empty background: travelling minstrel with a lute, colourful patched cloak, mid-performance, fair skin and light brown hair | rd_pro__default | 48x48 | 1 | 0.18 | validates: rd_pro character sprite - keep the output, it feeds the two-stage animations \| use a saved reference_image of your base villager for look consistency |

## P0 - needed now (replaces assets the game already draws) / icons/actions

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| icon-talk-e3097146-smoke | small game UI icon of speech bubble, simple, readable at tiny size | rd_mini__skill_icon | 32x32 | 1 | 0.06 | validates: mini icon readability at tiny size \| batch of 4 variants; keep the clearest |

## P0 - needed now (replaces assets the game already draws) / icons/goods

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| good-bread-5f6af63a-smoke | small game item icon of round bread loaf | rd_mini__topdown_item | 32x32 | 1 | 0.06 | validates: mini item icon |
| goods-sheet-644710b6-smoke | inventory item sheet: wheat sheaf, bread loaf, flour sack, stone block, log stack, sand pile, paper, ink pot, pamphlet, pigment pots, cloth bolt, glass pane, garments, hammer tools, lute, gold coins | rd_pro__inventory_items | 256x256 | 1 | 0.18 | validates: inventory_items atlas mode + include_downloadable_data \| one call draws a matched set; include_downloadable_data returns the item atlas |

## P0 - needed now (replaces assets the game already draws) / icons/ui

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| ui-panel-e028fe6e-smoke | carved wooden-framed parchment UI panel, empty centre, game UI element | rd_pro__ui_panel | 256x256 | 1 | 0.18 | validates: ui_panel chrome |

## P1 - fills gaps in shipped features / festivals

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| festival-summer-solstice-2724705f-smoke | single isometric medieval village asset, wide festive scene: summer solstice festival, great bonfire, flower crowns, longest day celebration, villagers of varied ages and skin tones | rd_pro__isometric | 256x256 | 1 | 0.18 | validates: wide isometric scene |

## ANIMATED - premium tier, all animation styles / anim/walk

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| walk4-peasant-m-35444cad-smoke | male peasant villager, medieval | rd_animation__four_angle_walking | 48x48 | 1 | 0.07 | validates: prompt-driven 4-angle walk GIF \| 48px 4-direction walk cycle, transparent GIF |

## ANIMATED - premium tier, all animation styles / anim/battle

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| battle-knight-bba7a199-smoke | knight in mail with sword | rd_animation__battle_sprites | 64x64 | 1 | 0.07 | validates: battle sprites set (idle/walk/jump/attack) \| idle/walk/jump/attack set |

## ANIMATED - premium tier, all animation styles / anim/vfx

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| vfx-bonfire-cd308abf-smoke | large bonfire burning, festival night | rd_animation__vfx | 64x64 | 1 | 0.07 | validates: vfx animation loop |

## ANIMATED - premium tier, all animation styles / anim/ambient

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| amb-mill-sails-a80df364-smoke | isometric medieval: windmill sails slowly turning | rd_animation__any_animation | 64x64 | 1 | 0.25 | validates: any_animation ambient loop |
| amb-festival-dance-d34c0154-smoke | isometric medieval: villagers dancing in a ring at festival | rd_animation__big_animation | 128x128 | 1 | 0.25 | validates: big_animation 128px scene loop |

## ANIMATED - premium tier, all animation styles / anim/rotate8

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| rot8-player-8c8c63e7-smoke | young organizer in green tunic, medieval | rd_animation__8_dir_rotation | 64x64 | 1 | 0.25 | validates: 8-direction rotation sheet \| 8 directional views; return_spritesheet gives the 3x3 sheet |

## ANIMATED - premium tier, all animation styles / anim/two-stage

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| adv-walking-676c6372-smoke | slow heavy steps | rd_advanced_animation__walking | 64x64 | 1 | 0.14 | validates: two-stage advanced animation - needs a person-* output as input_image \| two-stage: input_image = the person-* P0 sprite, native res, padded to 64px. Send native-res frame, pad to 64px, retry once on provider_busy |
| adv-rotate-4a8ed779-smoke | the uploaded subject | rd_advanced_animation__rotate | 64x64 | 1 | 0.10 | validates: advanced rotate - 8 views of an uploaded still \| two-stage: input_image = any P0 building/prop still; returns the 8 views. Send native-res frame, pad to 64px, retry once on provider_busy |

## P0 - needed now (replaces assets the game already draws) / icons/ui

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| ui-title-keyart-8651a36f-smoke | sweeping isometric medieval valley town at golden hour, river and bridge, walled town, fields - title screen key art | rd_pro__painterly | 256x256 | 1 | 0.18 | validates: painterly key art |

## P1 - fills gaps in shipped features / compare/tiers

| id | prompt | style | size | n | est $ | notes |
|---|---|---|---|---|---|---|
| compare-cottage-fast-cbcf31b7-smoke | isometric medieval cottage, timber-frame and thatch | rd_fast__game_asset | 64x64 | 1 | 0.03 | validates: rd_fast quality floor \| quality floor check: if Fast looks fine, filler assets get much cheaper |
| compare-cottage-plus-77832dea-smoke | isometric medieval cottage, timber-frame and thatch | rd_plus__default | 128x128 | 1 | 0.06 | validates: rd_plus middle tier \| middle tier, same subject as the rd_pro cottages-set |
