CREATE TABLE scale (
    id          BIGSERIAL PRIMARY KEY,
    serial      varchar(64) NOT NULL
);

CREATE TABLE measurement (
    id          BIGSERIAL PRIMARY KEY,
    scale       BIGSERIAL REFERENCES scale(id),
    raw_value   REAL
);
