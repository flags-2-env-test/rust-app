# Base pinned by digest: a floating tag would make a red fixture ambiguous
# between "the library changed" and "the toolchain changed".
FROM rust:1-bookworm@sha256:14bc9c5966e7b3a385794b3d5389a8765668342025fbcc7b2e3d2866ac4bd8c3

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
COPY Cargo.toml Cargo.lock ./
COPY src ./src

ENV CARGO_HOME=/tmp/cargo
ENV CARGO_TARGET_DIR=/tmp/target
ENV FLAGS2ENV_NATIVE_LIB=/app/.vendor/.zed/oresoftware/flags-2-env/build/libflags2env.so

# --locked: the committed Cargo.lock is the only accepted resolution. Without
# it cargo may silently pick newer transitive versions and a red fixture stops
# meaning what it is supposed to mean.
RUN cargo build --release --locked --quiet

RUN useradd --create-home --shell /bin/sh --uid 10001 fixture
USER fixture

CMD ["/tmp/target/release/demo"]
