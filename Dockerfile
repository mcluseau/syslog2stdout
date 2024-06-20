from rust:1.79-alpine as build
run apk add --no-cache musl-dev

workdir /src
copy Cargo.* .
copy src src

run cargo build -r

# ------------------------------------------------------------------------
from alpine:3.19
copy --from=build /src/target/release/syslog2stdout /bin/
entrypoint ["/bin/syslog2stdout"]
