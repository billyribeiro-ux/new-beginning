-- citext: case-insensitive email uniqueness.
-- pgcrypto: digest()/gen_salt() for operational queries and future in-migration
--           hashing. NOT used for UUID generation — PKs come from Rust now_v7().
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;
