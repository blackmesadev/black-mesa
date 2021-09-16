FROM golang:1.17

WORKDIR /go/src/app
COPY . .

RUN go build -o black-mesa

CMD ["./black-mesa"]
