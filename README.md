# dealrs: a Rust library for card games

dealrs is a Rust library for playing card games (like poker), providing a set of low-level utilities for efficiently working with cards, decks, evaluation, ranking, and combinatorial exploration thereof. On top of this, it provides a few higher level frameworks for evaluating simulations, strategies, game variants, optimal play, and building your own applications.

This is currently a work-in-progress, and the API is not yet stable.

* [List of poker hands](https://en.wikipedia.org/wiki/List_of_poker_hands)
* [Poker Project (CodeThrowdown)](http://www.codethrowdown.com/)
  * [5 Card Single Deck Hands](http://www.codethrowdown.com/5CardSingleDeckHands.txt)

## Usage

See the [examples](examples) for usage.

As a quick sanity check, you can run the [`examples/sim/pairs.rs`](examples/sim/pairs.rs) example to see if the library is working correctly:

```shell
$ cargo run --package dealrs --example sim-pairs
Simulation complete, final results:
Ran a total of 10000 deals, finding 589 pairs
On a random deal, this is a 5.89% chance of happening
On average, you will find a pair every 16.98 deals
```

Unless something is completely broken with your system (never this library 😉), you should expect around a 1/17 chance of finding a pair in a random deal.

## Benchmarks

There are lots of benchmarks, since performance is important for card games. You can run all of them with the following command:

```shell
$ cargo bench
# ...
```

This will take a very long time and produce a report. Performance is pretty good, but the benchmarking is not exhaustive or particularly organized. In the future, I hope to have exhaustive benchmarks that can be used to objectively inform decisions about algorithmic improvements.

## Development

### Regenerating Lookup Tables

If you need to regenerate the lookup tables, you can do so with the following command:

```shell
# regenerates the lookup tables for the best 5-card hands
# NOTE: --no-default-features is required if the files do not exist, since by default they are included and will cause a compilation error
$ cargo run --package dealrs --example gen-lutbest5 --no-default-features
```

You can double check the generated report ([`lutbest5.md`](./src/hand/lutbest5/lutbest5.md)) to see if the tables are correct, and when debugging.
