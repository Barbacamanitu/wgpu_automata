# GPU Accelerated Cellular Automata Simulator

![Automata Screenshot](https://github.com/Barbacamanitu/wgpu_automata/raw/master/screenshots/gui.png)

Work in progress cellular automata simulator that runs on the gpu. Written in Rust.

Supports totalistic cellular automata via the SimParams::Totalistic enum. Supports Golly like rule strings, such as "B3/S23" for the game of life.

Currently working on adding neural cellular automata support with customizable filters and activation functions. The first example of this is in the Continuous simulator. Access this via the SimParams::Continuous enum variant.
