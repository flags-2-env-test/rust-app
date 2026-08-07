FROM rust:1-bookworm

WORKDIR /app

RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential make \
 && rm -rf /var/lib/apt/lists/*

# The dependency, exactly where .zpkg.toml's [install].dir puts it.
COPY .vendor/.zed/oresoftware/flags-2-env ./.vendor/.zed/oresoftware/flags-2-env

# Upstream tracks prebuilt macOS artifacts under build/ (a Mach-O parser.o and a
# .dylib). Clean before building or the Linux link step consumes them.
RUN make -C .vendor/.zed/oresoftware/flags-2-env clean && make -C .vendor/.zed/oresoftware/flags-2-env shared

COPY .cli-flags.toml ./
COPY Cargo.toml ./
COPY src ./src

ENV CARGO_HOME=/tmp/cargo
ENV CARGO_TARGET_DIR=/tmp/target
ENV FLAGS2ENV_NATIVE_LIB=/app/.vendor/.zed/oresoftware/flags-2-env/build/libflags2env.so

RUN cargo build --release --quiet

CMD ["/tmp/target/release/demo"]
