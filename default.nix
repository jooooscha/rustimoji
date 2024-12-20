{ pkgs ? import <nixpkgs> {}, ...}:

pkgs.stdenv.mkDerivation rec {
  pname = "rustimoji";
  version = "v1.0.2";
  src = fetchTarball {
      url = "https://github.com/jooooscha/rustimoji/releases/download/${version}/rustimoji-linux.tar.gz";
      sha256 = "09jmfck5k98s5vxvr4cq65p1kxydpvl8247i840m5yipyys104hf";
  };
  buildInputs = with pkgs; [
    libgcc
  ];
  nativeBuildInputs = with pkgs; [
    autoPatchelfHook
  ];
  phases = [ "unpackPhase" "installPhase" "fixupPhase" ];
  installPhase = ''
     mkdir -p $out/bin
     cp rustimoji $out/bin/
  '';
}
