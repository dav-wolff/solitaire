{ lib
, stdenv
, fetchgit
, curl
, popt
}:

stdenv.mkDerivation {
	pname = "svg-playing-cards";
	version = "0-unstable-2023-06-22";
	
	buildInputs = [
		curl
		popt
	];
	
	src = fetchgit {
		url = "https://github.com/revk/SVG-playing-cards.git";
		rev = "08706399f146279c621a6515a2bcd87a8e12e646";
		fetchSubmodules = true;
		hash = "sha256-i5E7rpP0pw9qxO+Y6yUz8+bjy6oQ/U4bkksq6AMTKzM=";
	};
	
	installPhase = ''
		mkdir -p $out/bin
		cp makecards $out/bin/
	'';
	
	meta = with lib; {
		description = "Application to make a wide selection of different types of SVG playing cards with lots of options";
		mainProgram = "makecards";
		homepage = "https://github.com/revk/SVG-playing-cards.git";
		license = with licenses; [ gpl3Only ];
	};
}
