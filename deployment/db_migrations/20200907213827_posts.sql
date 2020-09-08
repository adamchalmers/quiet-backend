-- +goose Up
-- +goose StatementBegin
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
ALTER EXTENSION "uuid-ossp" SET SCHEMA public;

CREATE TABLE IF NOT EXISTS users (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ DEFAULT NULL,
    name            TEXT        NOT NULL
);

CREATE TABLE IF NOT EXISTS posts (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ DEFAULT NULL,
    text            TEXT        DEFAULT NULL,
    user_id         UUID        REFERENCES users (id)
);

CREATE TABLE IF NOT EXISTS follows (
    -- Account `reads` follows the posts of accounts `posts`
    posts           UUID        REFERENCES users (id),
    reads           UUID        REFERENCES users (id)
);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS follows;
DROP TABLE IF EXISTS users;
-- +goose StatementEnd
