# Changelogs

## Version 1.0.4

This release still fix sequencer bugs.
By this fix the sequencers seem to be nice.

- bugfix: sequencer clears internal events when looping ([ea0e796](https://github.com/t-sin/tapirus/commit/ea0e79663ee3ad48ae69c89efb13706fb21b0500))

## Version 1.0.3

The main change in this release is a bugfix for sequencers.
Before this release, when a long sequence pattern is set to sequencer, sequencers repeats the short part of the pattern.

- Fix event scheduling bugs in sequencers
- Avoid compiler warnings ([c1cbd34](https://github.com/t-sin/tapirus/commit/c1cbd3461cbac20e20560e28e9c444e5fada5c2c))

## Version 1.0.2

Minor release for some bugfixes and improvements.

### Improvements

- Sort unit generators by the hierarchial order when dumping ([03232cf](https://github.com/t-sin/tapirus/commit/03232cf3eb90b7e31235f208da907fad90907256))
- Comments are available in Tapir Lisp ([c816a43](https://github.com/t-sin/tapirus/commit/c816a432dc139dddc81aece9db8b770c93c615c9))

### Bugfix

- Fix error message for `rand` oscillator ([48c6ab1](https://github.com/t-sin/tapirus/commit/48c6ab1024ebb3051a3c3a8a71e9b001f67098e3))

## Version 1.0.1

Minor release for internal tiny changes.

## Version 1.0.0

Split the sound synthesis modules from [Koto music performer](http://github.com/t-sin/koto).
It includes features below:

- Simple musical time abstructions
- Unit generator system for sound synthesis
    - oscillators, sequencers, effects and some utility units
- Tapir Lisp; audio graph contruction language
