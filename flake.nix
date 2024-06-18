{
	inputs = {
		nixpkgs = {
			url = "github:nixos/nixpkgs/nixpkgs-unstable";
		};
		
		flake-utils = {
			url = "github:numtide/flake-utils";
		};
		
		crane = {
			url = "github:ipetkov/crane";
			inputs.nixpkgs.follows = "nixpkgs";
		};
		
		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};
	
	outputs = {self, nixpkgs, flake-utils, crane, fenix}:
		flake-utils.lib.eachDefaultSystem (system:
			let
				pkgs = import nixpkgs {
					inherit system;
				};
				
				inherit (pkgs) lib;
				
				fenixPackage = fenix.packages.${system};
				fenixToolchain = fenixPackage.default.toolchain; # nightly
				craneLib = (crane.mkLib pkgs).overrideToolchain fenixToolchain;
				
				src = with lib; cleanSourceWith {
					src = craneLib.path ./.;
					filter = craneLib.filterCargoSources;
				};
				
				nameVersion = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };
				pname = nameVersion.pname;
				version = nameVersion.version;
				
				svg-playing-cards = pkgs.callPackage ./svg_playing_cards.nix {};
				cards = pkgs.runCommand "${pname}-${version}-cards" {
					nativeBuildInputs = [svg-playing-cards];
				} ''
					mkdir $out
					makecards -d $out \
						--plain \
						--ace=plain \
						--ace1="" --ace2="" \
						--width="56mm" \
						--height="87mm" \
						--w=190 \
						--h=360 \
						--ph=58 \
						--corner=12
					rm $out/1B.svg
					rm $out/2B.svg
					rm $out/1J.svg
					rm $out/2J.svg
				'';
				
				libraries = with pkgs; [
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
				
				commonArgs = {
					inherit pname version src;
					strictDeps = true;
					
					nativeBuildInputs = with pkgs; [
						pkg-config
						clang
						mold
					];
					
					buildInputs = with pkgs; [
						alsa-lib.dev
						udev
					];
					
					cargoExtraArgs = "--locked --no-default-features";
				};
				
				cargoArtifacts = craneLib.buildDepsOnly commonArgs;
				
				solitaire = craneLib.buildPackage (commonArgs // {
					inherit cargoArtifacts;
					
					SOLITAIRE_CARDS_LOCATION = cards;
					
					postFixup = lib.optionalString pkgs.stdenv.isLinux ''
						patchelf $out/bin/solitaire --set-rpath ${lib.makeLibraryPath libraries}
					'';
				});
			in {
				packages = {
					default = solitaire;
					inherit svg-playing-cards cards;
				};
				
				checks = {
					test = craneLib.cargoTest (commonArgs // {
						inherit cargoArtifacts;
					});
					
					clippy = craneLib.cargoClippy (commonArgs // {
						inherit cargoArtifacts;
						cargoClippyExtraArgs = "--all-targets -- --deny warnings";
					});
				};
				
				devShells.default = craneLib.devShell {
					packages = libraries ++ (with pkgs; [
						rust-analyzer
						pkg-config
						clang
						mold
					]);
					
					LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
					
					SOLITAIRE_CARDS_LOCATION = cards;
				};
			}
		);
}
