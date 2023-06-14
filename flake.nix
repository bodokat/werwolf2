{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };

  };
  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:

    let
      pkgs = import nixpkgs {
        inherit system;
      };

      frontend = 
       pkgs.mkYarnPackage rec {
          pname = "werwolf-frontend";
          version = "0.1.0";
          src = ./web;
          packageJson = "${src}/package.json";
          yarnLock = "${src}/yarn.lock";


          buildPhase = ''
            export HOME=$TMP
            yarn --offline build
          '';

          installPhase = 
          let pkgName = (pkgs.lib.importJSON "${src}/package.json").name;
          in 
            ''
              cp -r deps/${pkgName}/dist/ $out
            '';

          # do not attempt to build distribution bundles
          distPhase = ''
            true
          '';
        };
      
      backend = (let
        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource (craneLib.path ./server);
        cargoArtifacts = craneLib.buildDepsOnly { inherit src; };
        bin = craneLib.buildPackage { inherit src cargoArtifacts;
          STATIC_PATH="${frontend}";
         };

        in 
          bin
        );

      
      in
        {
          packages = rec {
            default = docker;
            inherit backend frontend;

            docker = pkgs.dockerTools.streamLayeredImage {
              name = "werwolf";
              tag = "latest";
              config = {
                Cmd = [ "${backend}/bin/werwolf" ];
              };
          };
          };
          
        }
    );
      
}