
# install rust nightly
curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly

# install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# build wasm
wasm-pack build

# install node modules
npm install

# build web
npm run build