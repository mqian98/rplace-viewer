# rplace-viewer
Views r/place data

### Prereqs: 
* Install rust: https://www.rust-lang.org/tools/install

### Setup Steps:
1. Download the git repo
2. Download dataset (named output_white) from: https://drive.google.com/drive/folders/1hZ0dHd0WYgBmOt4cztPFBzLrKa2hV3Ao?usp=share_link
3. Save file as `output_white` in `data/custom/`
4. Run `cargo run` in the base directory

### Commands:
* Scroll - zoom in/out
* Shift+scroll or J/L - fast forward/backward in time
* Ctrl+scroll or ,/. - move foward/backward 1 pixel edit at a time
* Plus/Minus - Control how much to fast forwards/backwards by
* Click+drag - move canvas around
* Shift+click - select area
* P - screenshot selected area or whole screen if nothing is selected 
* WASD - move canvas up/down/left/right
* Q/Esc - quit
* 1 - jump to start
* 2 - jump to start of first expansion
* 3 - jump to start of second expansion
* 4 - jump to end
