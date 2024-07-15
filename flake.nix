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
		
		cache_bust = {
			url = "github:dav-wolff/cache_bust";
			inputs.nixpkgs.follows = "nixpkgs";
			inputs.fenix.follows = "fenix";
		};
	};
	
	outputs = { self, nixpkgs, flake-utils, ... } @ inputs: let
		makeCraneLib = pkgs: let
				fenixNative = pkgs.fenix.complete; # nightly
				fenixWasm = pkgs.fenix.targets.wasm32-unknown-unknown.latest; # nightly
				fenixToolchain = pkgs.fenix.combine [
					fenixNative.rustc
					fenixNative.rust-src
					fenixNative.cargo
					fenixNative.rust-docs
					fenixNative.clippy
					fenixWasm.rust-std
				];
		in (inputs.crane.mkLib pkgs).overrideToolchain fenixToolchain;
	in {
		overlays = {
			svg-playing-cards = final: prev: {
				svg-playing-cards = prev.callPackage ./nix/svg_playing_cards.nix {};
			};
			
			cachebust = final: prev: {
				cachebust = inputs.cache_bust.packages.${prev.system}.cli;
			};
			
			fenix = final: prev: {
				fenix = inputs.fenix.packages.${prev.system};
			};
			
			solitaire = final: prev: let
				inherit (prev) callPackage;
				craneLib = makeCraneLib final;
			in {
				solitaire = {
					cards = callPackage ./nix/cards.nix {
						inherit craneLib;
					};
					native = callPackage ./nix/native.nix {
						inherit craneLib;
					};
					web = callPackage ./nix/web.nix {
						inherit craneLib;
					};
				};
			};
			
			default = nixpkgs.lib.composeManyExtensions (with self.overlays; [
				svg-playing-cards
				cachebust
				fenix
				solitaire
			]);
		};
	} // flake-utils.lib.eachDefaultSystem (system:
			let
				pkgs = import nixpkgs {
					inherit system;
					overlays = [self.overlays.default];
				};
				craneLib = makeCraneLib pkgs;
			in {
				packages = {
					inherit (pkgs) svg-playing-cards;
					inherit (pkgs.solitaire) native web cards;
					
					default = pkgs.solitaire.native;
					
					serve = pkgs.writeShellScriptBin "solitaire-serve" ''
						${pkgs.http-server}/bin/http-server ${pkgs.solitaire.web}
					'';
				};
				
				checks = let
					prefixChecks = prefix: with pkgs.lib; mapAttrs' (name: value:
						nameValuePair "${prefix}_${name}" value
					);
				in prefixChecks "native" pkgs.solitaire.native.tests // prefixChecks "web" pkgs.solitaire.web.tests;
				
				devShells.default = craneLib.devShell {
					packages = with pkgs; solitaire.native.libraries ++ [
						fenix.rust-analyzer
						pkg-config
						clang
						mold
					];
					
					LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath pkgs.solitaire.native.libraries;
					
					SOLITAIRE_CARDS_LOCATION = pkgs.solitaire.cards;
					CACHE_BUST_ASSETS_DIR = pkgs.solitaire.cards;
					CACHE_BUST_SKIP_HASHING = 1;
				};
			}
		);
}
