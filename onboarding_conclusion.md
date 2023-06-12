### Notes on my code
* Much cleaner to move the a_star into a method on HexGrid
* My events systems would have been much better if I had all verification happen at the receiving system (like the `select_tile` system Felix has)
* The cursor system is perfect in the tutorial

### Notes on Felix's code
* why from_world() to load `HexGrid`?
* Could've used a better `match` for `handle_changed_tiles`
* Why not use the `bevy::Timer` for handling ticks?
