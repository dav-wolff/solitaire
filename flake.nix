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
	
	outputs = {self, nixpkgs, flake-utils, crane, fenix}: {
		overlays = {
			svg-playing-cards = final: prev: {
				svg-playing-cards = prev.callPackage ./nix/svg_playing_cards.nix {};
			};
			
			craneLib = final: prev: let
				fenixPackage = fenix.packages.${prev.system};
				fenixNative = fenixPackage.complete; # nightly
				fenixWasm = fenixPackage.targets.wasm32-unknown-unknown.latest; # nightly
				fenixToolchain = fenixPackage.combine [
					fenixNative.rustc
					fenixNative.rust-src
					fenixNative.cargo
					fenixNative.rust-docs
					fenixNative.clippy
					fenixWasm.rust-std
				];
				craneLib = (crane.mkLib final).overrideToolchain fenixToolchain;
			in {
				inherit craneLib;
			};
			
			solitaire = final: prev: let
				inherit (prev) callPackage;
			in {
				solitaire = {
					cards = callPackage ./nix/cards.nix {};
					native = callPackage ./nix/native.nix {};
					web = callPackage ./nix/web.nix {};
				};
			};
			
			default = nixpkgs.lib.composeManyExtensions (with self.overlays; [
				svg-playing-cards
				craneLib
				solitaire
			]);
		};
	} // flake-utils.lib.eachDefaultSystem (system:
			let
				pkgs = import nixpkgs {
					inherit system;
					overlays = [self.overlays.default];
				};
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
				
				devShells.default = pkgs.craneLib.devShell {
					packages = with pkgs; solitaire.native.libraries ++ [
						rust-analyzer
						pkg-config
						clang
						mold
					];
					
					LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath pkgs.solitaire.native.libraries;
					
					SOLITAIRE_CARDS_LOCATION = pkgs.solitaire.cards;
				};
			}
		);
}
