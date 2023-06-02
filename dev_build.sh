sudo docker build -f docker/Dockerfile.dev -t media-uploader:dev docker
sudo docker run -it --mount type=bind,source=$(pwd)/code/frontend,target=/build media-frontend:dev /bin/bash -c "cargo build --release --target wasm32-unknown-unknown && wasm-bindgen --out-name media-frontend --out-dir pkg --target web target/wasm32-unknown-unknown/release/media_frontend.wasm"
sudo docker run -it --rm --mount type=bind,source=$(pwd)/code/backend,target=/build media-uploader:dev /bin/bash -c "cargo build"
sudo docker build --no-cache -f docker/Dockerfile.serve -t media-uploader:serve code
sudo docker run --name media-uploader -e ENDPOINT=mongodb://127.0.0.1:27017/media -e TABLE=violin --rm --network=host media-uploader:serve