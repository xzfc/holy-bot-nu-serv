with import <unstable> {};
let sqlite-3-24 = sqlite.overrideAttrs (x: rec {
  name = "sqlite-${version}";
  version = "3.24.0";
    src = fetchurl {
      url = "http://sqlite.org/2018/sqlite-autoconf-3240000.tar.gz";
      sha256 = "0jmprv2vpggzhy7ma4ynmv1jzn3pfiwzkld0kkg6hvgvqs44xlfr";
    };
  });
in
stdenv.mkDerivation {
  name = "batch";
  nativeBuildInputs = [ cargo rustc sqlite-3-24 rustfmt ];
}
