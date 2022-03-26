FROM rust:1.58.1

WORKDIR /raidhelpbot
COPY . .
RUN cargo install --path .

CMD ["raidhelpbot"]
