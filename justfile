default: run

clean:
    rm -rf build/

build-wasm: clean
    mkdir -p build/wasm/
    cp -r wasm/* build/wasm/.
    cp -r assets/ build/wasm/assets
    cargo build --release --target wasm32-unknown-unknown
    cp $CARGO_TARGET_DIR/wasm32-unknown-unknown/release/flock-flow.wasm build/wasm/bevy_game_bg.wasm
    wasm-bindgen --no-typescript --out-name bevy_game --out-dir build/wasm --target web build/wasm/bevy_game_bg.wasm

package-wasm: clean build-wasm
    rm -f dist/flock-flow-wasm-v0.0.1-test.zip
    mkdir -p dist/
    cd build/wasm && zip -r ../../dist/flock-flow-wasm-v0.0.1-test.zip .

run: build-wasm
    sfz ./build/wasm

export-svg:
    inkscape --export-filename assets/boi.png --export-id boi resource/assets.svg
    inkscape --export-filename assets/player.png --export-id player resource/assets.svg
    inkscape --export-filename assets/calmboi.png --export-id calmboi resource/assets.svg
    inkscape --export-filename assets/angryboi.png --export-id angryboi resource/assets.svg
    inkscape --export-filename assets/collectible.png --export-id collectible resource/assets.svg
    inkscape --export-filename assets/smoke.png --export-id smoke resource/assets.svg
