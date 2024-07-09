{ callPackage
, runCommand
, craneLib
, svg-playing-cards
}:

let
	common = callPackage ./common.nix {
		inherit craneLib;
	};
	inherit (common) pname version;
in runCommand "${pname}-cards-${version}" {
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
		--corner=12 \
		--back=Plain \
		--back-colour=#03a32e \
		--card=1B
	mv $out/1B.svg $out/slot.svg
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
''
