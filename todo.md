+ collisions, such as polygon selection
+ score
+ shape generation
- render on GPU instead of CPU
- moving shape feel (cursor sticks to the place where shape is located)
- graphics improvement
- further optimisations
- levels
- spicy objects




===================== WGPU
+ refactor to Game / UI / Rendering
+ click on the shape to select
+ cursor becomes the shape
+ clicking places the shape
+ draw the score
+ update panel when shapes are over
+ shape over the board checks if we can place it
+ bug (can place shape over the edge)


- juicyness - i.e. background, particle effects, colour changes
- performance metrics (fps, mem, cpu etc..), to check if the game is properly optimised
- game rules as a config. (Such as, number of shapes no the panel, score limit etc.)
- rules - i.e next level, previous level.
- deploy to browser
- check if can't put any shape => gameover
- initial cell distribution on the level.

- may be some image opens in the background
- debug mode - where I can see all shapes, grids, make levels, etc..
- configuring the text buffers
- nice textures
+ shadow for cursor, to see exact position on the board.
- levels
- sound

