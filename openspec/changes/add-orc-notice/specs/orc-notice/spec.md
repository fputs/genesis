# orc-notice

## Purpose

The repository ships an ORC License notice. The notice names the license, the Licensed Material, the Reserved Material, and the attribution. The README points at that notice.

## ADDED Requirements

### Requirement: The notice names the ORC License

`LICENSES/ORC-NOTICE` SHALL contain the ORC Notice part. That part SHALL state that this product is licensed under the ORC License and SHALL give the license text location in these words: This product is licensed under the ORC License located at the Library of Congress at TX 9-307-067 and available online at various locations. All warranties are disclaimed as set forth therein.

**Source**: ORC License Art. III.a (https://paizo.com/orclicense).

#### Scenario: The notice quotes the ORC License reference

- **GIVEN** `LICENSES/ORC-NOTICE`
- **WHEN** the ORC Notice part is read
- **THEN** it contains `This product is licensed under the ORC License located at the Library of Congress at TX 9-307-067 and available online at various locations. All warranties are disclaimed as set forth therein.`

### Requirement: New mechanics are Licensed Material

`LICENSES/ORC-NOTICE` SHALL contain a Licensed Material part that states that all new Genesis game mechanics are ORC Licensed Material.

**Source**: ORC License Art. I.e and Art. III, and the research checklist item that new mechanics are ORC Licensed Material (https://paizo.com/orclicense).

#### Scenario: New Genesis mechanics are Licensed Material

- **GIVEN** `LICENSES/ORC-NOTICE`
- **WHEN** the Licensed Material part is read
- **THEN** it contains `All new Genesis game mechanics are ORC Licensed Material.`

### Requirement: Reserved Material names this product

Designer review of pull request 85. `LICENSES/ORC-NOTICE` SHALL contain a Reserved Material part that lists the setting, characters, story, art, music, and the title and logo of Project Genesis, says the list may not be limited to those elements, and says Licensed Material from Paizo Inc. is not Reserved Material.

**Source**: ORC License Art. III.c (https://paizo.com/orclicense). Designer review of pull request 85.

#### Scenario: Reserved Material names the Genesis setting and title

- **GIVEN** `LICENSES/ORC-NOTICE`
- **WHEN** the Reserved Material part is read
- **THEN** it contains `Reserved Material elements in this product include, but may not be limited to: the setting, characters, story, art, music, and the title and logo of Project Genesis, and all elements designated as Reserved Material under the ORC License. Licensed Material from Paizo Inc. is not Reserved Material.`

### Requirement: Expressly Designated Licensed Material is none

Designer review of pull request 85. The section in `LICENSES/ORC-NOTICE` immediately after Reserved Material SHALL be Expressly Designated Licensed Material, and it SHALL state that no Reserved Material in this product is designated as Licensed Material.

**Source**: ORC License Art. III.d (https://paizo.com/orclicense). Designer review of pull request 85.

#### Scenario: Expressly Designated Licensed Material follows Reserved Material

- **GIVEN** `LICENSES/ORC-NOTICE`
- **WHEN** the section after Reserved Material is read
- **THEN** that section is `Expressly Designated Licensed Material` followed by `None. No Reserved Material in this product is designated as Licensed Material.`

### Requirement: Attribution copies the three book lines

`LICENSES/ORC-NOTICE` SHALL contain an Attribution part. That part SHALL include the Player Core, GM Core, and Monster Core attribution lines verbatim, each Designed by list and each Authors list included, and SHALL include the line `Project Genesis © 2026 Evan Burkey.` Designer review of pull request 85: the credit request SHALL say `Genesis Licensed Material`.

**Source**: Archives of Nethys licenses page, fetched 2026-10-23 (https://2e.aonprd.com/Licenses.aspx). ORC License Art. III.b (https://paizo.com/orclicense).

#### Scenario: Player Core attribution is the Archives of Nethys line

- **GIVEN** `LICENSES/ORC-NOTICE`
- **WHEN** the Attribution part is read
- **THEN** it contains this line: `Pathfinder Player Core © 2023 Paizo Inc., Designed by Logan Bonner, Jason Bulmahn, Stephen Radney-MacFarland, and Mark Seifter. Authors: Alexander Augunas, Kate Baker, Logan Bonner, Jason Bulmahn, Carlos Cabrera, Calder CaDavid, James Case, Eleanor Ferron, Steven Hammond, Joan Hong, Vanessa Hoskins, James Jacobs, Jenny Jarzabski, Erik Keith, Dustin Knight, Lyz Liddell, Luis Loza, Patchen Mortimer, Dennis Muldoon, Stephen Radney-MacFarland, Mikhail Rekun, David N. Ross, Michael Sayre, Mark Seifter, Kendra Leigh Speedling, Mark Thompson, Clark Valentine, Andrew White, Landon Winkler, and Linda Zayas-Palmer`

#### Scenario: GM Core attribution is the Archives of Nethys line

- **GIVEN** `LICENSES/ORC-NOTICE`
- **WHEN** the Attribution part is read
- **THEN** it contains this line: `Pathfinder GM Core © 2023 Paizo Inc., Designed by Logan Bonner and Mark Seifter. Authors: Amirali Attar Olyaee, Logan Bonner, Creighton Broadhurst, Jason Bulmahn, James Case, Jesse Decker, Eleanor Ferron, Fabby Garza Marroquín, Jaym Gates, Matthew Goetz, James Jacobs, Brian R. James, Jenny Jarzabski, Dustin Knight, Jason LeMaitre, Lyz Liddell, Luis Loza, Ron Lundeen, Stephen Radney-MacFarland, David N. Ross, Michael Sayre, Mark Seifter, Owen K.C. Stephens, Amber Stewart, Clark Valentine, Landon Winkler, and Linda Zayas-Palmer`

#### Scenario: Monster Core attribution is the Archives of Nethys line

- **GIVEN** `LICENSES/ORC-NOTICE`
- **WHEN** the Attribution part is read
- **THEN** it contains this line: `Pathfinder Monster Core © 2024 Paizo Inc., Authors: Alexander Augunas, Dennis Baker, Kate Baker, Joshua Birdsong, Joseph Blomquist, Logan Bonner, Jason Bulmahn, James Case, John Compton, Paris Crenshaw, Adam Daigle, Darrin Drader, Brian Duckwitz, Robert N. Emerson, Scott Fernandez, Eleanor Ferron, Leo Glass, Matthew Goodall, BJ Hensley, Thurston Hillman, Vanessa Hoskins, James Jacobs, Jenny Jarzabski, Miko Kallio, Jason Keeley, Jeff Lee, Lyz Liddell, Luis Loza, Ron Lundeen, Robert G. McCreary, Philippe-Antoine Menard, Jacob W. Michaels, Dave Nelson, Jason Nelson, Tim Nightengale, Stephen Radney-MacFarland, Mikhail Rekun, Patrick Renie, Alex Riggs, David N. Ross, Michael Sayre, Mark Seifter, Chris S. Sims, Amber Stewart, Jeffrey Swank, William Thompson, Jason Tondro, Clark Valentine, Landon Winkler, Tonya Woldridge, and Linda Zayas-Palmer`

#### Scenario: Genesis is credited on its own line

- **GIVEN** `LICENSES/ORC-NOTICE`
- **WHEN** the Attribution part is read
- **THEN** it contains `Project Genesis © 2026 Evan Burkey.`
- **AND** the credit request is `If you use Genesis Licensed Material in your own published work, please credit us as follows:`

### Requirement: The notice states the player-campaign note

`LICENSES/ORC-NOTICE` SHALL state that Genesis does not host player-made campaigns, that creators handle their own distribution and ORC attribution, and that the bundled campaign follows ORC rules.

**Source**: Project decision recorded in this change. Genesis does not host player-made campaigns.

#### Scenario: The notice states the player-campaign note

- **GIVEN** `LICENSES/ORC-NOTICE`
- **WHEN** the player-campaign note is read
- **THEN** it contains `Genesis does not host player-made campaigns. Creators handle their own distribution and ORC attribution. The bundled campaign follows ORC rules.`

### Requirement: The README points at the notice

`README.md` SHALL contain a short section that links to `LICENSES/ORC-NOTICE` and repeats the player-campaign note from the notice file.

**Source**: Same player-campaign note as the notice file.

#### Scenario: The README links the notice and repeats the note

- **GIVEN** `README.md`
- **WHEN** the ORC notice section is read
- **THEN** it links to `LICENSES/ORC-NOTICE`
- **AND** it contains `Genesis does not host player-made campaigns. Creators handle their own distribution and ORC attribution. The bundled campaign follows ORC rules.`

### Requirement: Attribution is factual only

`LICENSES/ORC-NOTICE` and the README section SHALL NOT claim compatibility, endorsement, sponsorship, or approval, and SHALL NOT add Pathfinder branding or a logo. The word Pathfinder SHALL appear only inside the three book attribution lines.

**Source**: ORC License Art. I.b and II.d.i (https://paizo.com/orclicense). The Compatibility License is not used.

#### Scenario: The notice and README make no compatibility claim

- **GIVEN** `LICENSES/ORC-NOTICE` and `README.md`
- **WHEN** they are read for branding
- **THEN** neither file contains a compatibility claim, an endorsement claim, a sponsorship claim, or an approval claim
- **AND** neither file contains a Pathfinder logo
- **AND** `README.md` does not contain the word Pathfinder
- **AND** `LICENSES/ORC-NOTICE` contains the word Pathfinder only inside the three book attribution lines

### Requirement: The notice does not include a lawyer review

`LICENSES/ORC-NOTICE` and the README section SHALL NOT include a lawyer review. That review happens at release and is out of this change.

**Source**: Release process. This change does not perform that review.

#### Scenario: The notice does not include a lawyer review

- **GIVEN** `LICENSES/ORC-NOTICE` and the new README section
- **WHEN** they are read
- **THEN** neither one includes a lawyer review

### Requirement: The repository has one notice file

Designer review of pull request 85. The file `NOTICE-ORC.md` SHALL be absent. `README.md` and `docs/architect/licensing.md` SHALL name `LICENSES/ORC-NOTICE` and SHALL NOT name `NOTICE-ORC.md`.

**Source**: Designer review of pull request 85.

#### Scenario: The old notice file is gone

- **GIVEN** the repository
- **WHEN** `NOTICE-ORC.md` is looked up
- **THEN** that file is absent
- **AND** `README.md` does not contain `NOTICE-ORC.md`
- **AND** `docs/architect/licensing.md` does not contain `NOTICE-ORC.md`
- **AND** both of those files name `LICENSES/ORC-NOTICE`

### Requirement: The changelog records the notice

This pull request SHALL add one new bullet under `## [Unreleased]` and `### Added` in `CHANGELOG.md`, and that bullet SHALL link this pull request. `LICENSES/ORC-NOTICE` is not an exempt docs path.

**Source**: Changelog spec, requirement "Mixed pull request category": a path outside the skip list is a behavior path.

#### Scenario: The changelog bullet links this pull request

- **GIVEN** the `CHANGELOG.md` diff for this pull request
- **WHEN** a reviewer reads `## [Unreleased]`
- **THEN** one new bullet is under `### Added`
- **AND** that bullet links this pull request
