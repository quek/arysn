FROM rust:1.63.0-slim-bullseye

RUN apt update && apt install -y locales curl

RUN localedef -i ja_JP -c -f UTF-8 -A /usr/share/locale/locale.alias ja_JP.UTF-8
ENV LANG ja_JP.UTF-8

RUN cargo install cargo-watch
RUN rustup component add rustfmt

RUN curl -L https://github.com/rust-analyzer/rust-analyzer/releases/latest/download/rust-analyzer-linux -o /usr/local/bin/rust-analyzer
RUN chmod +x /usr/local/bin/rust-analyzer
RUN rustup component add rust-src

ADD wait-for-it.sh /
ADD start.sh /

WORKDIR /app

CMD ["/start.sh"]
