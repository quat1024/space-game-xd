ludum dare 48 funy game
=======================

Welcome to my funny ludum dare 48 game!!

* LLD and the MSVC toolchain required to build on windows (install llvm, make sure `lld-link.exe` is on path somewhere)
	* I think
	* Maybe it doesnt even use lld?
	* If this is a problem edit `./cargo/config.toml` but don't expect good link times
* Sorry in advance about the hard tabs

## Assets

Half-stored in `asset_src/` and half-stored in `assets/`, because i can't be bothered to make a proper system for copying over assets rn

`asset_src` is for stuff that the buildscript will process, the idea is that everything will go in there and `assets/` will become a file that i can gitignore, the buildscript will automatically clear, etc

maybe a location like `target/` would be a better idea for that ephemeral assets folder. Look @ what gradle does in minecraft modding stuff

idk !

## Link Pile

orthographic projection matrices http://learnwebgl.brown37.net/08_projections/projections_ortho.html

https://learnopengl.com/Getting-started/Coordinate-Systems