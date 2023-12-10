FROM debian:bookworm

ARG GROUP_ID
ARG USER_ID

RUN dpkg --add-architecture arm64 \
  && apt update && apt install -y ca-certificates curl pkg-config crossbuild-essential-arm64 libssl-dev libssl-dev:arm64
ENV PKG_CONFIG_SYSROOT_DIR="/usr/lib/aarch64-linux-gnu/pkgconfig"

RUN groupadd -g ${GROUP_ID} builder \
  && useradd -rm -d /home/builder -s /bin/bash -u ${USER_ID} -g ${GROUP_ID} builder
USER builder

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --target aarch64-unknown-linux-gnu \
  && echo '[target.aarch64-unknown-linux-gnu]' > /home/builder/.cargo/config.toml \
  && echo 'linker = "/usr/bin/aarch64-linux-gnu-gcc"' >> /home/builder/.cargo/config.toml

ENV PATH="/home/builder/.cargo/bin:${PATH}"

WORKDIR /home/builder/rust-pkg
CMD ["cargo", "build", "-r", "--target", "aarch64-unknown-linux-gnu"]
