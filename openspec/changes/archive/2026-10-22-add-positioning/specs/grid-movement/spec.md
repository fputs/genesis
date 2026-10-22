# grid-movement

## MODIFIED Requirements

### Requirement: Step and Crawl share one counter

Step, Crawl, and Leap SHALL share the one diagonal counter for that creature's turn. A diagonal Step SHALL advance that counter. This capability SHALL NOT implement Crawl or Leap. Later movement actions use that same counter.

**Rule**: Player Core pg. 421 counts all movement in the turn. Crawl and Leap are movement. https://2e.aonprd.com/Rules.aspx?ID=2357 https://2e.aonprd.com/Actions.aspx?ID=2293 https://2e.aonprd.com/Actions.aspx?ID=2298
**Why**: Crawl and Leap belong to add-crawl-and-leap.
**Substitute**: Step advances the shared turn counter. Crawl and Leap are not functions of this change.
**Intent**: Later movement actions take the counter Step already updates.

#### Scenario: Step and Crawl share the diagonal counter

- **GIVEN** Speed 25 and one fresh turn counter
- **WHEN** the creature Steps one diagonal square
- **THEN** the counter counts one diagonal, a diagonal Stride on that same counter costs 10 feet, and no Crawl or Leap action is resolved
