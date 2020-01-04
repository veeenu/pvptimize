# Synopsis of a PvP fight

## Individual Pokémon State machine

At every turn (0.5s interval) each player can be in one of four possible states:
- *Neutral*: warrants instantaneous transition to one of the other
  states. **Transitioning costs 0 turns**
- *Idle*: when a fast move was selected, awaiting for this turn to pass,
  can go into another idle state or a register fast state. **Transitioning costs 1 turn**
- *Register Fast*: register the effects of a fast move and transition to
  neutral.  **Transitioning costs 1 turn**
- *Register Charged*: register the effects of a charged move and
  transition to neutral. Accessible from neutral *iff* there is enough
  energy. **Transitioning costs 1 turn**

In case both players are in the *Register Charged* state, a
*charged move priority* calculation ensues to determine which of the two
Pokémon applies the effects first (and potentially faints the other one).

## Battle state machine

On top of this mechanism, we can compute what each turn in the battle
looks like by executing individual transitions up to the next turn, and
matching the two states to apply the relevant effect.
- In case both players are either in an *Idle* or *Register Fast* state,
  the two fast move effects (or lack thereof) get immediately applied.
- In case one player is in *Register Charged* state and the other is either
  in *Idle* or *Register Fast* state, apply both charged move effects
  and fast move effect (or lack thereof).
- In case both players are in the *Register Charged* state, compute the
  charged move priority to decide which of the two goes first. Apply the
  effects in the given order.