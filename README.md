# rplace-viewer
Views r/place data

### Prereqs: 
* Install rust: https://www.rust-lang.org/tools/install

### Setup Steps:
1. Download the git repo
2. Download dataset (named output_white) from: https://drive.google.com/drive/folders/1hZ0dHd0WYgBmOt4cztPFBzLrKa2hV3Ao?usp=share_link
3. Save file as `output_white` in `data/custom/`
4. In the terminal, run `cargo run` in the base directory
5. After the command, you should see a white square with black borders (i.e. the start state of the r/place canvas). Use the commands below to traverse the canvas. 

### Commands:
* Scroll - Zoom in/out
* Shift+Scroll or J/L - Fast forward/backward in time
* Ctrl+Scroll or ,/. - Move foward/backward 1 pixel edit at a time
* Plus/Minus - Control how much to fast forwards/backwards by
* Mouse Press->Drag - Move canvas around
* Shift+Mouse Press->Drag - Select area
* P - Screenshot selected area or whole screen if nothing is selected. Images are saved in the `screenshots` folder
* WASD - Move canvas up/down/left/right
* Q/Esc - Quit
* 1 - Jump to start
* 2 - Jump to start of first expansion
* 3 - Jump to start of second expansion
* 4 - Jump to end
