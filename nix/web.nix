{ callPackage
, craneLib
, wasm-bindgen-cli
, binaryen
, solitaire
}:

let
	common = callPackage ./common.nix {
		inherit craneLib;
	};
	inherit (common) pname;
	
	commonArgs = common.args // {
		pname = "${pname}-web";
		cargoExtraArgs = "--locked --target wasm32-unknown-unknown --no-default-features --features web";
		CARGO_PROFILE = "wasm-release";
		
		# bevy already looks up relative paths in the "assets" folder
		SOLITAIRE_CARDS_LOCATION = ".";
	};
	
	cargoArtifacts = craneLib.buildDepsOnly commonArgs;
	
	tests = {
		clippy = craneLib.cargoClippy (commonArgs // {
			inherit cargoArtifacts;
			cargoClippyExtraArgs = "--all-targets -- --deny warnings";
		});
	};
in craneLib.buildPackage (commonArgs // {
	passthru = {
		inherit tests;
	};
	
	inherit cargoArtifacts;
	SOLITAIRE_CANVAS_ID = "canvas";
	
	nativeBuildInputs = [
		wasm-bindgen-cli
		binaryen
	];
	
	postBuild = ''
		wasm-bindgen --no-typescript --target web --out-dir wasm-bindgen --out-name "solitaire" target/wasm32-unknown-unknown/wasm-release/solitaire.wasm
		wasm-opt -Oz wasm-bindgen/${pname}_bg.wasm -o wasm-bindgen/${pname}_optimized.wasm
	'';
	
	installPhaseCommand = ''
		mv site $out
		mv wasm-bindgen/${pname}_optimized.wasm $out/${pname}_bg.wasm
		mv wasm-bindgen/${pname}.js $out/${pname}.js
		cp -r --no-preserve=all ${solitaire.cards} $out/assets
	'';
	
	doCheck = false; # can't run tests for wasm build
})
