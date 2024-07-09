{ lib
, stdenv
, callPackage
, craneLib
, alsa-lib
, vulkan-loader
, xorg
, libxkbcommon
, wayland
, udev
, solitaire
}:

let
	common = callPackage ./common.nix {
		inherit craneLib;
	};
	
	libraries = [
		udev
		alsa-lib
		vulkan-loader
		xorg.libX11
		xorg.libXcursor
		xorg.libXi
		xorg.libXrandr
		libxkbcommon
		wayland
	];
	
	commonArgs = common.args // {
		buildInputs = [
			alsa-lib.dev
			wayland
			udev
		];
		
		cargoExtraArgs = "--locked --no-default-features --features native";
		
		SOLITAIRE_CARDS_LOCATION = solitaire.cards;
	};
	
	cargoArtifacts = craneLib.buildDepsOnly commonArgs;
	
	tests = {
		test = craneLib.cargoTest (commonArgs // {
			inherit cargoArtifacts;
		});
		
		clippy = craneLib.cargoClippy (commonArgs // {
			inherit cargoArtifacts;
			cargoClippyExtraArgs = "--all-targets -- --deny warnings";
		});
	};
in craneLib.buildPackage (commonArgs // {
	passthru = {
		inherit libraries tests;
	};
	
	inherit cargoArtifacts;
	
	postFixup = lib.optionalString stdenv.isLinux ''
		patchelf $out/bin/solitaire --set-rpath ${lib.makeLibraryPath libraries}
	'';
})
