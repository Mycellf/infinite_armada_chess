Todo: 

* Add pawn promotion
* Detect checkmate

Instructions: 

* Use arrow keys to pan and shift to pan faster
* Click pieces to select them and move them
* Hold `shift` while selecting an empty tile to select the first piece below it
* Type `:`, a rank number, then `enter` to jump to it
* Type `tile1 tile2` then `enter` to make a move from one tile to another (ex `c-10 c20`)
* Press `1`, `2`, and `3` to select a zoom level

## How does it work? 

The pieces are stored in an expanding array. When the engine attempts to read from an out of bounds rank, the data structure "lies" and returns a reference to a default rank full of the right color of queen pieces. When the engine attempts to write to an out of bounds rank, the data structure expands as one would expect. Each rank takes up 12 bytes of memory, so it's unlikely that anyone will ever run into memory consumption issues. It would take over two and a half years of moving once per second to fill more than a gigabyte. 

(1 000 000 000 bytes ÷ 12 bytes per rank × 1 second per move ÷ 31 557 600 seconds per year ≈ 2.6 years)
