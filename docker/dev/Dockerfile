FROM rust:latest

RUN apt update && apt install -y locales

RUN localedef -i ja_JP -c -f UTF-8 -A /usr/share/locale/locale.alias ja_JP.UTF-8
ENV LANG ja_JP.UTF-8

RUN cargo install cargo-watch

RUN rustup component add rls rust-analysis rust-src

ADD wait-for-it.sh /
ADD start.sh /

WORKDIR /app/server

CMD ["/start.sh"]
