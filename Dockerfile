FROM scratch


ARG SERVER_PORT=1337
ARG BUILD_DATE
ARG VCS_REF
ARG BUILD_VERSION


LABEL org.label-schema.version="1.0" \
      org.label-schema.build-date=${BUILD_DATE} \
      org.label-schema.name="meesign-server" \
      org.label-schema.description="Meesign server for threshold ECDSA signatures." \
      org.label-schema.url="https://github.com/crocs-muni/meesign-server" \
      org.label-schema.vcs-ref=${VCS_REF} \
      org.label-schema.version=${BUILD_VERSION} \
      org.label-schema.docker.cmd="docker run --detach --publish ${SERVER_PORT}:${SERVER_PORT} meesign-server:latest"


COPY ./target/x86_64-unknown-linux-musl/release/meesign-server /meesign-server

EXPOSE ${SERVER_PORT}

CMD ["/meesign-server"]
