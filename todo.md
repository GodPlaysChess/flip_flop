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
+ shadow for cursor, to see exact position on the board.
+ initial cell distribution on the level.
+ rules - i.e next level, previous level.

Even when i do not draw anything it's around 23& utilisation.
- optimisations: (currently 23% for 120 fps/ 28% for 240 fps)  my baseline.
  + re-upload vertrex buffers for grid and panel, only when they change
  + limit fps
  + do not re-render static data.
  - window cursor + skip the loop


- performance metrics (fps, mem, cpu etc..), to check if the game is properly optimised
- juicyness - i.e. background, particle effects, colour changes
- game rules as a config. (Such as, number of shapes no the panel, score limit etc.)
- deploy to browser
- check if can't put any shape => gameover

- levels
- may be some image opens in the background
- debug mode - where I can see all shapes, grids, make levels, etc..
- configuring the text buffers
- nice textures
- sound

