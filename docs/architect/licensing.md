# Licensing requirements

These rules are taken from the ORC and Paizo licensing research dated 2026-10-21. That research is not legal advice. A claim the research marks unverified or ambiguous is marked **UNVERIFIED** here. No CI check enforces these rules yet. Each rule is written so a later check can test it.

Primary sources named in the research:

- ORC License and the AxE: https://paizo.com/orclicense
- Paizo licenses overview: https://paizo.com/licenses
- Paizo Compatibility License: https://paizo.com/licenses/compatibility
- Paizo Community Use Policy: https://paizo.com/licenses/communityuse
- Archives of Nethys licenses page: https://2e.aonprd.com/Licenses.aspx

## LIC-1. Source code is MIT

Source code is under the MIT License in `LICENSE`. That file licenses the code. The `license` field on the crate manifest comes in a later spec.

Source-code paths are `crates/`, `scripts/`, and `.github/workflows/`. The AxE states: "No license is granted under the ORC to specific software code." The research report's conclusion, which is not license text, is that the source code can stay closed and that there is no copyleft on code.

Check: `LICENSE` is the MIT license, and those three trees contain no ORC grant of the code itself. This check does not read a `license` field from `crates/rules/Cargo.toml`.

## LIC-2. Rules mechanics are ORC Licensed Material

Game rules mechanics are Licensed Material under the ORC License. That is the mechanics derived from the books listed in the Attribution section of `LICENSES/ORC-NOTICE`, plus the mechanics in `openspec/specs/` (dice and checks, creature template, modifiers, and grid movement), plus the mechanics stated in `docs/designer/`. The text of `LICENSES/ORC-NOTICE` is a legal notice. It states Reserved Material, which ORC I.a excludes, so that notice is not Licensed Material. The code under `crates/rules/` stays MIT source code under LIC-1.

The research quotes ORC I.e(2): expressions reasonably necessary to convey the functional ideas and methods of operation of a game, including character statistics, actions, encounters, combat, conditions, difficulty classes, and dice rolling. ORC I.h reserves trademarks, art, music, sound, and delineated characters, settings, plots, and proper nouns.

Check: a rules spec or a designer rules document that states a mechanic is treated as ORC Licensed Material and satisfies LIC-3 and LIC-4. The text of `LICENSES/ORC-NOTICE` is not treated as Licensed Material. A source file under `crates/` is not reclassified as ORC text.

## LIC-3. Sources are three Remaster ORC books

The only permitted rules sources are:

- Pathfinder Player Core
- Pathfinder GM Core
- Pathfinder Monster Core

The research quotes Paizo's licenses page: ORC content cannot be used to turn Open Game Content that exists only under the OGL into ORC Licensed Material. Player Core, GM Core, and Monster Core list only Paizo on the Archives of Nethys licenses page. The research says other Remaster-era supplements (Player Core 2, Monster Core 2, NPC Core, Howl of the Wild, and others) show Paizo-only license blocks on that page and appear to be ORC as well. **UNVERIFIED**: confirm each of those books' own notices before any use. Until that confirmation, they are not permitted sources.

Check: every rules entry names one of the three books. Any other book name fails the check.

## LIC-4. Every rules entry records its source book and page

Every rules entry in a rules spec, a rules document, or rules data records the source book and the page. The research checklist says to record the source book for every rules entity. The page is recorded with the book so the entry can be checked against that book.

Check: a rules entry with no source book, or with no page, fails the check.

## LIC-5. No OGL-only or pre-Remaster content, including alignment

Do not implement content that exists only under the OGL, and do not implement pre-Remaster content. The research quotes the Remaster announcement: the game moved from the Open Game License to the ORC License and removed alignment and a small number of creatures, spells, and magic items exclusive to the OGL. Monster Core cut creatures that had been used from the OGL.

Do not use these books, which the research names: Core Rulebook, Gamemastery Guide, Bestiary 1–3, Advanced Player's Guide, Secrets of Magic, and any pre-Remaster book whose Archives of Nethys license block cites the Core Rulebook (2019) or a System Reference Document or Tome of Horrors entry. Do not use alignment. Do not use a legacy spell name the Remaster renamed, or a monster the Remaster dropped.

**UNVERIFIED**: the research did not compile a per-item list of renamed spells and monsters.

Check: a rules entry whose source is one of those books, or whose text states alignment, fails the check.

## LIC-6. Strip Paizo proper nouns, lore, art, and trademarks

Rules text and rules data do not include Paizo proper nouns (characters, deities, locations, organizations, iconics), adjectives or titles derived from those proper nouns, lore paragraphs, or Paizo art, maps, or trade dress. A mechanic whose name embeds a proper noun is renamed to a generic name. The research quotes the AxE: stat-block contents are predominantly Licensed Material, and proper nouns and trademarks have to be edited out. Descriptive language that supports a mechanic can stay. A named inventor and a backstory do not.

**UNVERIFIED**: the research marks this ambiguous for a deity-specific domain, sanctification and deity selection, a spell or feat named after a person or place, an ancestry or heritage tied to a region, and a class feature that requires a setting deity. Those are renamed or left out until that line is confirmed.

Check: a rules entry that contains a Paizo proper noun, a lore paragraph, or Paizo art fails the check.

## LIC-7. No Pathfinder branding beyond the Attribution book titles, and no compatibility claim

Do not put "Pathfinder" in the product title, a logo, or other branding. Do not use the Paizo compatibility logo. Do not claim that the product is compatible with Pathfinder, and do not claim that Paizo sponsors, endorses, or approves it. Pathfinder may be named only as the book titles in the Attribution section of `LICENSES/ORC-NOTICE`. Repeating those same book titles as the permitted sources (LIC-3) or as a rules entry's source (LIC-4) is that Attribution naming. Internal repository file and directory names are not product branding and are exempt; this does not cover product titles, store pages, marketing, or in-game UI.

The store page and marketing text carry no compatibility wording and no endorsement wording.

The research quotes ORC I.b and II.d.i: the ORC grants no trademark rights and no permission to imply a connection with the licensor. The Compatibility License grant is limited to printed books and electronic books, and it says standalone game systems are not authorized (https://paizo.com/licenses/compatibility). This product is not under that license. Do not add a Paizo trademark line or a compatibility line to `LICENSES/ORC-NOTICE`.

**UNVERIFIED**: the research marks nominative fair use of a compatibility phrase as a lawyer question, not a grant from Paizo. This project does not use that phrase.

Check: the product title, logos, store page, marketing text, `README.md`, and `LICENSES/ORC-NOTICE` contain no compatibility claim and no endorsement claim. A Pathfinder mention is one of the book titles in the Attribution section of `LICENSES/ORC-NOTICE`, or the same title repeated under LIC-3, LIC-4, or LIC-9.

## LIC-8. Where the notice is placed

The notice has the four parts required by ORC Art. III, as named in the AxE sample: ORC Notice, Attribution, Reserved Material, and Expressly Designated Licensed Material. The research quotes Art. III: the grant is conditioned on including those notices in a reasonable manner for the medium.

Required places:

1. An in-game Credits or Legal screen reachable from the main menu.
2. A notice file bundled with the build.
3. `LICENSES/ORC-NOTICE` in this repository.

Recommended, as project policy and not as an ORC requirement (research §1.2): the store page. The research says the license does not require the store page. The store page is not release-blocking.

**UNVERIFIED**: the research marks placement ambiguous. Art. III names no location. The AxE says "a notice in your published work." A public repository is a distribution and needs the notice (research §6). Today only the repository copy exists. The in-game screen and the bundled file are still required and are not shipped.

Check: `LICENSES/ORC-NOTICE` has the four headings. A release is incomplete while the in-game credits screen or the bundled notice file is missing. A missing store-page notice does not block the release.

## LIC-9. Attribution names the three books and their authors

`LICENSES/ORC-NOTICE` attribution lists Pathfinder Player Core © 2023 Paizo Inc., Pathfinder GM Core © 2023 Paizo Inc., and Pathfinder Monster Core © 2024 Paizo Inc. The research says III.b(i) requires crediting each licensor, including upstream licensors, in the manner those parties request. Copying the full designer and author line from each book's legal page is the research's safe choice.

The author lines in `LICENSES/ORC-NOTICE` are the full Designed by and Authors lines copied from the Archives of Nethys licenses page on 2026-10-23. The file has no author-line TODO. **UNVERIFIED** pending the legal pages of Player Core, GM Core, and Monster Core.

Check: the three book lines are present. The check fails if an author-line TODO is in the file.

## LIC-10. Genesis Reserved Material stays closed

Reserved Material is Evan's lore and the name "Project Genesis"; Genesis world and setting names, characters, places, factions, deities, plots, and dialogue; and Genesis art, music, and sound. The research quotes ORC I.h and the AxE: reserved material the publisher develops is exclusive property, and lore, names, story, art, and music stay closed. Art and audio in `docs/visual/` and `docs/audio/` are in that reserved set. `LICENSES/ORC-NOTICE` states that list in generic words.

**UNVERIFIED**: generic list pending named Genesis setting content and review by Evan or a lawyer. The research says Art. III.c asks for this product's own reserved material, and that Paizo's reserved-material paragraph does not have to be copied word for word.

Check: `LICENSES/ORC-NOTICE` contains the Genesis reserved-material statement. A rules entry that copies Evan's setting names into mechanics fails LIC-6 and this rule.

## LIC-11. New mechanics Genesis adds are ORC Licensed Material

Every new mechanic Genesis adds becomes ORC Licensed Material. The research quotes the AxE: using the ORC in a work licenses the relevant portions of the whole work, and new game mechanics in the published work are licensed out. That includes custom feats, spells, monster stat blocks, homebrew subsystems, and mechanics from Evan's campaign. Lore, names, story, art, and music stay Reserved Material under LIC-10.

Recipients of the adapted licensed material receive an offer of the licensed rights under the ORC (ORC II.b, as quoted in the research). Additional terms that restrict those rights are not allowed.

Check: a new mechanic is described as ORC Licensed Material in the spec that adds it, and it is not listed as Reserved Material.

## LIC-12. Rule text may be quoted; Archives of Nethys is not scraped

ORC Licensed Material may be quoted. The research quotes the AxE: verbatim use of Licensed Material is allowed and is not required. Quote only mechanical text, and only from the three books in LIC-3.

Do not scrape Archives of Nethys. Do not copy its database, its formatting, its summaries, its clarifications, or its structure. The research found no Archives of Nethys terms that grant downstream rights in that compilation. **UNVERIFIED**: no separate terms-of-use page was found. What may be used is the Paizo ORC text that the site displays, because of the ORC, with Paizo credited. Transcribe from the project's own copies of the books. Use the site only to cross-check.

Archives of Nethys pages also mix in reserved material and OGL legacy content. Copying a page wholesale fails LIC-5 and LIC-6.

Check: a rules entry that reproduces an Archives of Nethys table, page structure, or summary fails the check. A short quotation of a mechanic from one of the three books, with book and page under LIC-4, passes.

## LIC-13. Expressly Designated Licensed Material

`LICENSES/ORC-NOTICE` states: None. No Reserved Material in this product is designated as Licensed Material. The research's draft notice uses a none sentence, matching the none sentence it quotes from reproductions of Paizo's notices. A later decision can designate material. Until that decision is written down, the notice stays on none.

Check: the Expressly Designated Licensed Material section says there is none, or it names the elements a recorded decision designated.

## LIC-14. No technical measure may lock the mechanics

ORC II.b, as quoted in the research, forbids effective technological measures that restrict exercise of the licensed rights in the licensed material. Do not apply a measure that claims exclusivity over the mechanics.

**UNVERIFIED**: the research marks ordinary store DRM on the binary as ambiguous and as a lawyer question. It is untested whether that DRM restricts the licensed rights, because the mechanics can still be used elsewhere.

Check: a release process that encrypts or locks the mechanics so a recipient cannot use them fails the check.

## LIC-15. Distribution terms must not take the mechanics back

The research quotes an AxE warning: a distributor that requires assignment of all rights in the code would violate the ORC, because the mechanics still have to be licensed out. Publisher terms, platform terms, and terms for user-made levels must not demand assignment or exclusive rights in the Licensed Material.

Check: a store, platform, or editor terms document that requires assignment or exclusivity over the mechanics fails the check.

## Open questions for Evan and an IP lawyer

The research lists these as questions for an IP lawyer. They are not decided here.

1. Does a Credits or Legal screen plus a bundled file satisfy Art. III's "reasonable manner" for a digital game? Is a store-page notice advisable? **UNVERIFIED** (placement is ambiguous in the research).
2. Can marketing say the product is compatible with Pathfinder Second Edition on a nominative-fair-use theory, given that the Compatibility License excludes standalone products and video games? **UNVERIFIED** (lawyer item). LIC-7 does not use that wording.
3. Where is the practical line for mechanics that embed proper nouns (deity choices, sanctification, region-tied heritages, named spells)? **UNVERIFIED**.
4. Is quoting ORC rule text that was copied from Archives of Nethys clean, given that site's Community Use Policy status, the 2026 change in its arrangement with Paizo, and its own compilation rights? **UNVERIFIED**. LIC-12 forbids scraping and says to transcribe from the books.
5. Does platform DRM, or an encrypted data file, count as an effective technological measure that restricts licensed rights (ORC II.b)? **UNVERIFIED**. See LIC-14.
6. User-made levels and the Phase 2 map editor. The research says user levels that contain new mechanics are made by users, and that whether Genesis has to give an ORC notice or a grant-back for user-created mechanics it hosts or distributes is not addressed. The Phase 2 map editor is that editor. The research suggests the editor show the ORC notice and that user-content terms not conflict with the ORC. **UNVERIFIED**.
7. Do publisher or platform agreements that require IP assignment or exclusivity conflict with ORC II.b? See LIC-15.
8. Confirm that no pre-Remaster or OGL material slipped in through renamed-but-similar content, and check the risk from Paizo's name appearing in the attribution lines. **UNVERIFIED** (no per-item list was compiled).
9. Trademark search and clearance for the name "Project Genesis," outside the ORC.

The research also records these weak spots. Each is **UNVERIFIED**:

- Paizo's book notices were quoted from third-party reproductions, not from the books.
- The Pathfinder Infinite pages were blocked. Those quotes are search-index excerpts, and whether that program allows standalone executables is unverified. The research's conclusion that Infinite is not a route for this game is tied to that limit: content there is distributed on Pathfinder Infinite and Starfinder Infinite, the allowed files are PDFs, images, and import files for a virtual tabletop or character builder, and a publisher there may not also use the ORC. The agreement would turn proper nouns and characters into program IP. That would hand over Evan's campaign lore.
- No Archives of Nethys terms of use were found. The relationship with Paizo changed in July 2026.
- No per-item list of OGL-only spells and monsters removed in the Remaster was compiled.
