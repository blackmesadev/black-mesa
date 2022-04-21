FROM openjdk:11

RUN groupadd -g 322 lavalink && \
        useradd -r -u 322 -g lavalink lavalink

USER lavalink

WORKDIR /opt/Lavalink

COPY . .

RUN curl -LJO https://github.com/freyacodes/Lavalink/releases/download/3.4/Lavalink.jar

CMD ["java", "-Djdk.tls.client.protocols=TLSv1.1,TLSv1.2", "-Xmx2G", "-jar", "Lavalink.jar"]
