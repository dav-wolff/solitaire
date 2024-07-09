{ lib
, runCommand
, craneLib
, svg-playing-cards
, pkg-config
, clang
, mold
}:

let
	src = with lib; cleanSourceWith {
		src = craneLib.path ../.;
		filter = path: type: 
			hasInfix "/site/" path
			|| (craneLib.filterCargoSources path type);
	};
	
	nameVersion = craneLib.crateNameFromCargoToml { cargoToml = ../Cargo.toml; };
	pname = nameVersion.pname;
	version = nameVersion.version;
in {
	inherit pname version;
	
	args = {
		inherit pname version src;
		strictDeps = true;
		
		nativeBuildInputs = [
			pkg-config
			clang
			mold
		];
	};
}
