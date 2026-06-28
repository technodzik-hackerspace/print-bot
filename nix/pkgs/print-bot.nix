{ lib
, rustPlatform
, fetchFromGitHub
, openssl
, pkg-config
}:

rustPlatform.buildRustPackage rec {
  pname = "print-bot";
  version = "dev";

  src = fetchFromGitHub {
    owner = "technodzik-hackerspace";
    repo = "print-bot";
    rev = "842d10feb7b860039bd04635a7be85b0f9dbe954";
    sha256 = "0l9ygbgk5w557958dpjv3cjri198c0954a9pmgxdmw74yl604425";
  };

  buildInputs = [ openssl ];
  nativeBuildInputs = [ pkg-config ];

  cargoHash = "sha256-vQMFA1LGAsWJgVnntqxcHNaC0kA2CDIWDYsYxHUyOfM=";

  meta = with lib; {
    description = "Telegram printer bot";
    homepage = "https://github.com/technodzik-hackerspace/print-bot";
    mainProgram = "rust_test_bot";
  };
}
