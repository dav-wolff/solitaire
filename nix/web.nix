{ callPackage
, craneLib
, wasm-bindgen-cli
, binaryen
, cachebust
, cards
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
		CACHE_BUST_ASSETS_DIR = cards;
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
		cachebust
	];
	
	postBuild = ''
		wasm-bindgen --no-typescript --target web --out-dir wasm-bindgen --out-name "${pname}" target/wasm32-unknown-unknown/wasm-release/${pname}.wasm
		wasm-opt -Oz wasm-bindgen/${pname}_bg.wasm -o wasm-bindgen/${pname}_optimized.wasm
	'';
	
	installPhaseCommand = ''
		mv site $out
		mv wasm-bindgen/${pname}_optimized.wasm $out/${pname}_bg.wasm
		mv wasm-bindgen/${pname}.js $out/${pname}.js
		cp -r --no-preserve=all ${cards} $out/assets
	'';
	
	fixupPhase = ''
		runHook preFixup
		
		cachebust $out/assets
		wasm_hashed=$(cachebust $out --file ${pname}_bg.wasm --print file-name)
		substituteInPlace $out/${pname}.js --replace-fail ${pname}_bg.wasm $wasm_hashed
		js_hashed=$(cachebust $out --file ${pname}.js --print file-name)
		css_hashed=$(cachebust $out --file style.css --print file-name)
		substituteInPlace $out/index.html --replace-fail ${pname}.js $js_hashed
		substituteInPlace $out/index.html --replace-fail style.css $css_hashed
		
		runHook postFixup
	'';
	
	doCheck = false; # can't run tests for wasm build
})
