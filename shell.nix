{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell rec {
	nativeBuildInputs = with pkgs; [
		rustup
		cargo
		rustc
		trunk
		pkg-config
		wayland
		libxkbcommon
		xorg.libX11
		xorg.libXcursor
		xorg.libXrandr
		xorg.libXi
		libglvnd
	];
	LD_LIBRARY_PATH="${pkgs.libglvnd}/lib:${pkgs.libxkbcommon}/lib:${pkgs.xorg.libX11}/lib";
}
