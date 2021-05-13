FROM alpine:3.13

WORKDIR /app

ARG USERID="1000"
ARG USERNAME="chat"

RUN set -x && \
  addgroup -S $USERNAME -g $USERID && \
  adduser -S $USERNAME -G $USERNAME -u $USERID

COPY --chown=$USERNAME:$USERNAME target/x86_64-unknown-linux-musl/release/grpc_chat /usr/local/bin/grpc_chat

USER $USERNAME

CMD ["/usr/local/bin/grpc_chat"]
